# SystemVerilog simulation on many threads: a plan

`epitome-sv` compiles a design into a netlist simulator (`sv/src/netlist.rs`)
that runs on one thread. This document plans running one large design
on many threads: how the design is cut into tasks, how the state is laid
out so that tasks never write the same bytes, and how the tasks are run.
For the compiler itself, see [SystemVerilog](sv.md); for sources and
status labels, see the [index](README.md).

**Status:** Planned.

## 1. Goals

- **Use every core on one design.** A design large enough to need it
  (a whole chip tile) runs on all the cores of one machine. Many
  independent runs of a design already use lanes and states; this is
  about one run.
- **At least twice as many tasks as threads** in each phase, and more
  where the design allows, so that no thread waits long on another.
- **No locks.** Tasks share nothing they write. No `Mutex` and no
  atomics except in the dispatcher that orders the tasks.
- **The same answers on any number of threads**, bit for bit. One
  thread running the tasks in order is the simulator as it is now, and
  the reference the tests compare against.
- **A state that can be a mapped file.** The state stays one block of
  memory, cut into regions, so that a large design's state can live in
  a memory-mapped file.

## 2. Where the time goes

Each `_step` of the top settles the combinational logic, runs the
clocked processes (`_ff`), and where an edge came commits the registers
and settles again (see `Phase`). A clock cycle is two steps, so three
settles: one per step, and one after the commit.

On the large datapath designs measured so far (single thread, x86-64):

- **Settling is 80–95% of the time**, the clocked processes 5–15%, and
  committing under 5%. Settling is the phase to make parallel.
- **One block can dominate.** In one design, a single array of
  instances took over half of the settle. Unless the cut goes inside
  such a block, it bounds the speed-up (here, to under 2×).
- **A third of the settles may do nothing.** The settle after the clock
  falls sees only the clock change. Where no combinational logic reads
  a clock, that settle can be skipped. This is a single-thread gain, and
  is step 1 below.
- **Threads multiply the single-thread speed.** The netlist is still
  several times slower than single-threaded Verilator on datapath
  designs, so code quality and threads both have to improve.

## 3. Tasks

The compiler emits a graph of tasks for each step, and a small runtime
runs it. A task is a generated function `extern "C" fn(state: *mut u8)`
with its instances' offsets in the state as constants. Within a task,
code runs as it does now, calls to children included.

### 3.1 Cutting the hierarchy

The instance tree is walked from the top with a cost budget
`C = total / (k * threads)`, where `k` is at least 2 to 4. A task must
also be worth dispatching: a few microseconds of work, against a
dispatch of about 0.1 to 0.3 µs.

1. An instance whose subtree costs at most `C` is one task.
2. A dearer instance is opened. Each child becomes an item, recursively,
   and the parent's own code between the calls to its children becomes
   segments, cut along the order `order` already finds.
3. The edges between items come from the bit spans each node reads and
   writes, which `order` already computes within a specialisation. Ports
   connected in place (`Arg::Alias`, `Arg::Elems`) carry them from a
   child to its parent.
4. An array of instances called as one, an instance a lane, is cut into
   blocks of lanes, each block a task.
5. A leaf specialisation still dearer than `C` has its own code cut into
   `k` parts. This generalises the two-part split of a child that its
   parent calls twice (`Split`, `_a` and `_b`).
6. Small items are merged along chains, to save edges.
7. A list scheduler, simulated on the measured costs, chooses among the
   cuts and reports the speed-up it expects for a number of threads.

Costs come from profiles where there are any (section 6, step 1), and
otherwise from the size of each function's code.

### 3.2 The phases as tasks

- **Clocked processes:** a task per instance, running its own processes
  only. They have no edges between them: they read settled values, and a
  port below the top already keeps its next value in a slot of its own,
  so the order among them does not matter.
- **Commit:** a task per instance, after every clocked process. Later,
  each instance's commit can move into the first settle task that
  touches it.
- **Settle:** the graph of section 3.1.

Later still, an instance's clocked processes can start as soon as the
settle tasks they read are done, with no wait for the whole phase.

## 4. The state

Each byte of the state has exactly one task that writes it in each phase.

- **Checked when compiling.** Take any two tasks with no path between
  them in the graph. The writes of each must miss the reads and writes
  of the other. A write counts as the whole store the code makes (a
  64-bit word or a vector register), not just the bits it changes. A
  pair that fails is merged into one task, or compilation fails.
- **Laid out by writer.** The objects a task writes are placed together,
  and each task's region is padded to 128 bytes, so that no two threads
  write the same cache line (or its neighbour, which processors often
  fetch with it).
- **Mostly as now.** A child's state already lies inside its parent's.
  The exceptions are values a child writes in place through its ports,
  which today are kept in the parent. They move to the child's region.
- **Flags of their own.** Whether an edge came (`fired`) becomes a byte
  per task, which the dispatcher combines.

The regions are also the pages of a mapped file, when the state is one.

## 5. The runtime

The runtime lives in `epitome-sv`'s `model`, in Rust.

- **Workers:** `N` threads that last as long as the model, each pinned
  to a core.
- **A static schedule:** the compiler gives each worker a list of tasks
  in order. An instance then runs on the same core every cycle, and its
  state stays in that core's cache.
- **Epochs, not counters:** each task has an `AtomicU32`, the step it
  last finished. Before a task runs, its worker waits for each
  predecessor's epoch to reach the current step (`Acquire`, spinning
  with a pause). When it ends, the worker stores the step (`Release`).
  Nothing is reset between steps, and there are no queues and no locks.
- **One thread** runs the same lists in order, with no waits.

If the measured loads are uneven, the static lists can give way to work
stealing with a count of unfinished predecessors per task. The atomics
stay in the dispatcher.

## 6. Steps

1. **Measure.** Cost per instance for each phase, its own and with its
   children, as a tree (today's counters are per call). Skip settles
   where only clocks changed. Run Verilator with `--threads` on the same
   designs, as a program, for comparison.
2. **The graph, without a runtime.** Build the task graph of section 3
   and report, for each design, the total work, the longest path, and
   the expected speed-up on 8, 16, 32 and 64 threads. If the numbers are
   poor, we learn it here, before writing a runtime.
3. **The state.** The check and the layout of section 4.
4. **The runtime.** Clocked processes and commits first, as they have no
   edges; then settling.
5. **Cutting big blocks.** Specialisations cut in `k` parts, and arrays
   cut in blocks of lanes.
6. **Overlap and skipping.** Clocked processes started early, commits
   moved into settle tasks, and tasks skipped where none of their inputs
   changed since they last ran. A graph of tasks makes that last one
   cheap, and it may matter as much as the threads.

## 7. Tests

Generic designs written for the tests: arrays from generate loops,
pipelines, a mesh of small blocks, and a block much dearer than the rest.
Each is run on one thread and on several, and the states and outputs
must match bit for bit every cycle. The check of section 4 is tested on
designs built to break it: two children writing neighbouring bits of
one word, and an array whose lanes share a register.
