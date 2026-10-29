//! The built-in classes of the std package: semaphore (15.3), mailbox
//! (15.4) and process (9.7).
//!
//! Their objects are empty on the heap; their state is kept here by
//! handle. A task that would block (a mailbox `get` on an empty mailbox,
//! say) suspends its process, which tries the call again whenever the
//! object changes.

use super::*;

pub(super) enum Builtin {
    Semaphore { keys: i64 },
    /// Messages with the type each was put with, and the bound (0 for
    /// none).
    Mailbox { bound: usize, queue: VecDeque<(Const, TypeId)> },
    Process { proc: u32 },
}

/// The process states, in the order of `process::state` (9.7).
const FINISHED: i64 = 0;
const RUNNING: i64 = 1;
const WAITING: i64 = 2;
const SUSPENDED: i64 = 3;
const KILLED: i64 = 4;

/// Which built-in class, if any, a class is.
pub(super) fn builtin_class<'d>(d: &'d Design, c: ClassId, root: crate::ast::NodeId) -> Option<&'d str> {
    let cl = d.class(c);
    (cl.node == root).then(|| d.name(cl.name))
}

/// Whether a call of method `m` on a built-in class may block.
pub(super) fn blocking(class: &str, m: &str) -> bool {
    matches!((class, m), ("mailbox", "put" | "get" | "peek") | ("semaphore", "get") | ("process", "await"))
}

impl Sim<'_, '_, '_> {
    /// State for a new semaphore or mailbox, from `new`'s arguments.
    pub(super) fn builtin_new(&mut self, cx: &Cx, c: ClassId, h: u32, args: &[ExprId]) {
        let arg = |me: &mut Self, i: usize| args.get(i).and_then(|&a| me.eval(cx, a).to_i64()).unwrap_or(0);
        match builtin_class(self.d, c, self.ast.root) {
            Some("semaphore") => {
                let keys = arg(self, 0);
                self.builtins.insert(h, Builtin::Semaphore { keys });
            }
            Some("mailbox") => {
                let bound = arg(self, 0).max(0) as usize;
                self.builtins.insert(h, Builtin::Mailbox { bound, queue: VecDeque::new() });
            }
            Some("process") => {
                let _ = writeln!(self.out, "error: a process object cannot be created with new (9.7)");
                self.outcome.errors += 1;
            }
            _ => {}
        }
    }

    /// The handle of the process object for process `p`.
    pub(super) fn process_handle(&mut self, p: u32) -> u32 {
        if let Some(&h) = self.proc_obj.get(&p) {
            return h;
        }
        let c = self.d.classes.iter().position(|cl| cl.node == self.ast.root && self.d.name(cl.name) == "process");
        let h = self.heap.len() as u32;
        self.heap.push(Object { class: ClassId(c.unwrap_or(0) as u32), fields: vec![] });
        self.builtins.insert(h, Builtin::Process { proc: p });
        self.proc_obj.insert(p, h);
        h
    }

    /// A method of a built-in object. `None` means a task that would
    /// block: the caller suspends and tries again.
    pub(super) fn builtin_method(&mut self, cx: &Cx, m: &str, args: &[ExprId]) -> Option<Const> {
        let int = |v: i64| Some(Const::Bits(Bits::from_i64(32, true, v)));
        if m == "self" && args.is_empty() {
            let h = self.process_handle(self.cur_proc);
            return Some(Const::Handle(h));
        }
        let Const::Handle(h) = self.eval(cx, args[0]) else {
            let _ = writeln!(self.out, "error: method `{m}` called through a null handle");
            self.outcome.errors += 1;
            return Some(Const::Null);
        };
        let rest = &args[1..];
        let arg = |me: &mut Self, i: usize, dflt: i64| rest.get(i).and_then(|&a| me.eval(cx, a).to_i64()).unwrap_or(dflt);
        match self.builtins.get_mut(&h) {
            Some(Builtin::Semaphore { .. }) => {
                let n = arg(self, 0, 1);
                let Some(Builtin::Semaphore { keys }) = self.builtins.get_mut(&h) else { unreachable!() };
                match m {
                    "put" => {
                        *keys += n;
                        self.wake_blocked(h);
                        Some(Const::Null)
                    }
                    "get" | "try_get" => {
                        if *keys >= n {
                            *keys -= n;
                            int(1)
                        } else if m == "get" {
                            None
                        } else {
                            int(0)
                        }
                    }
                    _ => Some(Const::Null),
                }
            }
            Some(Builtin::Mailbox { bound, queue }) => {
                let (bound, len) = (*bound, queue.len());
                match m {
                    "num" => int(len as i64),
                    "put" | "try_put" => {
                        if bound > 0 && len >= bound {
                            return if m == "put" { None } else { int(0) };
                        }
                        let x = rest[0];
                        let v = self.eval(cx, x);
                        // A string literal is sent as a string.
                        let t = if matches!(v, Const::Str(_)) { Types::STRING } else { self.d.expr(x).ty };
                        if let Some(Builtin::Mailbox { queue, .. }) = self.builtins.get_mut(&h) {
                            queue.push_back((v, t));
                        }
                        self.wake_blocked(h);
                        if m == "put" { Some(Const::Null) } else { int(1) }
                    }
                    "get" | "try_get" | "peek" | "try_peek" => {
                        let tries = m.starts_with("try");
                        let Some(Builtin::Mailbox { queue, .. }) = self.builtins.get(&h) else { unreachable!() };
                        let Some((v, vt)) = queue.front().cloned() else {
                            return if tries { int(0) } else { None };
                        };
                        let x = rest[0];
                        let t = self.target(cx, x);
                        if !self.message_fits(vt, t.ty) {
                            // A message of another type (15.4.9).
                            if tries {
                                return int(-1);
                            }
                            let _ = writeln!(self.out, "error: the mailbox message's type does not match the variable it is read into (15.4)");
                            self.outcome.errors += 1;
                            return Some(Const::Null);
                        }
                        if m.ends_with("get")
                            && let Some(Builtin::Mailbox { queue, .. }) = self.builtins.get_mut(&h)
                        {
                            queue.pop_front();
                        }
                        self.write_cx(cx, &t, v, false);
                        self.wake_blocked(h);
                        if tries { int(1) } else { Some(Const::Null) }
                    }
                    _ => Some(Const::Null),
                }
            }
            Some(Builtin::Process { proc }) => {
                let q = *proc;
                match m {
                    "status" => int(self.status(q)),
                    "kill" => {
                        self.kill(q);
                        self.killed.insert(q);
                        self.wake_blocked(h);
                        Some(Const::Null)
                    }
                    "await" => {
                        if q == self.cur_proc {
                            let _ = writeln!(self.out, "error: a process cannot await itself (9.7)");
                            self.outcome.errors += 1;
                            return Some(Const::Null);
                        }
                        if self.procs[q as usize].state == State::Done { Some(Const::Null) } else { None }
                    }
                    "suspend" => {
                        if self.procs[q as usize].state != State::Done {
                            self.suspended.insert(q);
                        }
                        Some(Const::Null)
                    }
                    "resume" => {
                        if self.suspended.remove(&q) && self.held.remove(&q) {
                            self.active.push_back(q);
                        }
                        Some(Const::Null)
                    }
                    _ => Some(Const::Null),
                }
            }
            None => {
                self.unsupported(format!("method `{m}` of a built-in class"));
                Some(Const::Null)
            }
        }
    }

    /// `process::state` of process `q` (9.7).
    fn status(&self, q: u32) -> i64 {
        let pr = &self.procs[q as usize];
        if pr.state == State::Done {
            if self.killed.contains(&q) { KILLED } else { FINISHED }
        } else if self.suspended.contains(&q) {
            SUSPENDED
        } else if q == self.cur_proc {
            RUNNING
        } else if pr.state == State::Waiting {
            WAITING
        } else {
            RUNNING
        }
    }

    /// Whether a mailbox message of type `from` can be read into `to`: the
    /// types must match (15.4.9), integral ones by width and sign.
    fn message_fits(&self, from: TypeId, to: TypeId) -> bool {
        let t = &self.d.types;
        from == to
            || (t.is_integral(from) && t.is_integral(to) && t.width(from) == t.width(to) && t.is_signed(from) == t.is_signed(to))
            || (from == Types::STRING && to == Types::STRING)
            || (t.is_real(from) && t.is_real(to))
            || matches!((t.get(from), t.get(to)), (Type::Class(_) | Type::Null, Type::Class(_)))
    }

    /// Lets the processes blocked on an object try again.
    pub(super) fn wake_blocked(&mut self, h: u32) {
        if let Some(ps) = self.blocked.remove(&h) {
            for p in ps {
                self.wake(p);
            }
        }
    }

    /// A process ended: whoever awaits it can go on.
    pub(super) fn ended(&mut self, p: u32) {
        if let Some(&h) = self.proc_obj.get(&p) {
            self.wake_blocked(h);
        }
    }
}
