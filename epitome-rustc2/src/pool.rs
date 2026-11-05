//! Running jobs on a fixed number of threads, where a job may add more
//! jobs (parsing a file finds the files of its modules).
//!
//! Plain `std`: scoped threads, one queue behind a mutex, and a condition
//! variable for idle workers. Jobs are coarse (a file, a function), so one
//! queue is not a bottleneck. Results must not depend on the order jobs
//! run in; callers sort what they collect.

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

/// The stack each worker gets: the parser and later passes recurse as
/// deeply as the source nests.
const STACK_SIZE: usize = 64 << 20;

/// The number of threads to use when none is given: one per core.
pub fn default_threads() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get())
}

/// Lets a running job add more jobs.
pub struct Queue<J> {
    state: Mutex<State<J>>,
    wake: Condvar,
}

struct State<J> {
    jobs: VecDeque<J>,
    /// Jobs queued or running. The pool is done when this reaches zero.
    pending: usize,
}

impl<J> Queue<J> {
    /// Adds a job.
    pub fn push(&self, job: J) {
        let mut s = self.state.lock().unwrap();
        s.jobs.push_back(job);
        s.pending += 1;
        self.wake.notify_one();
    }

    /// The next job, waiting while others may still add some; `None` when
    /// all are done.
    fn pop(&self) -> Option<J> {
        let mut s = self.state.lock().unwrap();
        loop {
            if let Some(j) = s.jobs.pop_front() {
                return Some(j);
            }
            if s.pending == 0 {
                return None;
            }
            s = self.wake.wait(s).unwrap();
        }
    }

    fn done_one(&self) {
        let mut s = self.state.lock().unwrap();
        s.pending -= 1;
        if s.pending == 0 {
            self.wake.notify_all();
        }
    }
}

/// Runs `initial` and every job they add on `threads` threads, calling
/// `work(job, queue)` for each. Returns when every job has finished.
pub fn run<J: Send, F>(threads: usize, initial: Vec<J>, work: F)
where
    F: Fn(J, &Queue<J>) + Sync,
{
    let pending = initial.len();
    let queue = Queue { state: Mutex::new(State { jobs: initial.into(), pending }), wake: Condvar::new() };
    let threads = threads.max(1);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            std::thread::Builder::new()
                .stack_size(STACK_SIZE)
                .spawn_scoped(scope, || {
                    while let Some(job) = queue.pop() {
                        work(job, &queue);
                        queue.done_one();
                    }
                })
                .expect("spawn a worker thread");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn jobs_add_jobs() {
        // Each job n > 0 adds two jobs n - 1: 2^(d+1) - 1 jobs in all.
        let count = AtomicUsize::new(0);
        run(4, vec![10u32], |n, q| {
            count.fetch_add(1, Ordering::Relaxed);
            if n > 0 {
                q.push(n - 1);
                q.push(n - 1);
            }
        });
        assert_eq!(count.load(Ordering::Relaxed), (1 << 11) - 1);
    }

    #[test]
    fn no_jobs() {
        run(4, Vec::<u32>::new(), |_, _| unreachable!());
    }
}
