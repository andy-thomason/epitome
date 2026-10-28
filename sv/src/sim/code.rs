//! Statements to bytecode (`docs/design/sv.md` section 8.2).
//!
//! A process, task or function body becomes a run of `Op`s in one shared
//! code vector. Control flow becomes jumps, so a process can stop at any
//! wait and resume from its program counter. Expressions are not
//! compiled: an `Op` names the IR expression, which the evaluator walks,
//! since an expression can never wait.

use crate::design::*;

#[derive(Clone, Debug)]
pub enum Op {
    /// Evaluate for its effect.
    Expr(ExprId),
    /// An assignment statement: its lhs, rhs, kind and any timing.
    Assign(StmtId),
    Jump(u32),
    /// Jump unless the condition is true.
    JumpIfNot(ExprId, u32),
    /// A `case`: evaluate, then jump to the matching item's target.
    Case(StmtId, Vec<u32>, u32),
    /// Wait as the timing says, then go on.
    Wait(StmtId),
    /// `wait (cond)`.
    WaitCond(ExprId),
    /// `repeat (n)`: push the count.
    RepeatInit(ExprId),
    /// Decrement the count, or pop it and jump out.
    RepeatNext(u32),
    /// `foreach`: work out the indices, then step through them.
    ForeachInit(StmtId),
    ForeachNext(StmtId, u32),
    /// A block's automatic variables get their initial values.
    Enter(ScopeId),
    /// A task call: push a frame and jump into it.
    CallTask(ExprId),
    /// Return from a task or function, with a value.
    Return(Option<ExprId>),
    /// `fork`: the start of each branch, the join, the code after.
    Fork(Vec<u32>, Join, u32),
    /// The end of a forked branch.
    BranchEnd,
    WaitFork,
    DisableFork,
    /// `disable name`: the block's end, if it is one this code is in.
    Disable(ExprId, Option<u32>),
    Trigger(StmtId),
    ProcCont(StmtId),
    /// An immediate assertion: on failure, jump to the fail action, or
    /// report.
    Assert(StmtId, Option<u32>, u32),
    /// Wait for a change of anything the process is sensitive to.
    WaitSensitive,
    /// The end of a process body.
    Halt,
    /// Not modelled: report once and carry on.
    Unsupported(StmtId),
}

/// Compiles bodies into one code vector.
pub struct Compiler<'d, 'a> {
    pub d: &'d Design<'a>,
    pub code: Vec<Op>,
    /// Loop exits and continues, innermost last.
    loops: Vec<(Vec<usize>, Vec<usize>)>,
    /// Named blocks being compiled and the jumps to their ends.
    blocks: Vec<(Option<ScopeId>, Vec<usize>)>,
}

impl<'d, 'a> Compiler<'d, 'a> {
    pub fn new(d: &'d Design<'a>) -> Self {
        Compiler { d, code: Vec::new(), loops: Vec::new(), blocks: Vec::new() }
    }

    fn here(&self) -> u32 {
        self.code.len() as u32
    }

    fn emit(&mut self, op: Op) -> usize {
        self.code.push(op);
        self.code.len() - 1
    }

    fn patch(&mut self, at: usize, to: u32) {
        match &mut self.code[at] {
            Op::Jump(t) | Op::JumpIfNot(_, t) | Op::RepeatNext(t) | Op::ForeachNext(_, t) => *t = to,
            Op::Disable(_, Some(t)) => *t = to,
            Op::Fork(_, _, t) => *t = to,
            Op::Assert(_, _, t) => *t = to,
            _ => {}
        }
    }

    /// A process body. `always` bodies loop; `always_comb` and
    /// `always_latch` run once, then wait for their inputs to change.
    pub fn process(&mut self, p: &Process) -> u32 {
        let start = self.here();
        self.stmt(p.body);
        match p.kind {
            ProcKind::Always | ProcKind::AlwaysFf => {
                self.emit(Op::Jump(start));
            }
            ProcKind::AlwaysComb | ProcKind::AlwaysLatch => {
                self.emit(Op::WaitSensitive);
                self.emit(Op::Jump(start));
            }
            _ => {
                self.emit(Op::Halt);
            }
        }
        start
    }

    /// A subroutine body, ending in a return.
    pub fn sub(&mut self, s: &Subroutine) -> u32 {
        let start = self.here();
        self.emit(Op::Enter(s.scope));
        for &st in &s.body {
            self.stmt(st);
        }
        self.emit(Op::Return(None));
        start
    }

    pub fn stmt(&mut self, s: StmtId) {
        let st = self.d.stmt(s).kind.clone();
        match st {
            StmtKind::Null => {}
            StmtKind::Block { join: None, scope, stmts } => {
                let named = scope.filter(|&sc| self.d.scope(sc).name.is_some());
                self.blocks.push((named, vec![]));
                if let Some(sc) = scope {
                    self.emit(Op::Enter(sc));
                }
                for x in stmts {
                    self.stmt(x);
                }
                let (_, exits) = self.blocks.pop().unwrap();
                let end = self.here();
                for e in exits {
                    self.patch(e, end);
                }
            }
            StmtKind::Block { join: Some(j), scope, stmts } => {
                let fork = self.emit(Op::Fork(vec![], j, 0));
                if let Some(sc) = scope {
                    // Declarations of the fork are made before its branches.
                    let at = self.emit(Op::Enter(sc));
                    let _ = at;
                }
                let mut starts = Vec::new();
                for x in stmts {
                    starts.push(self.here());
                    self.stmt(x);
                    self.emit(Op::BranchEnd);
                }
                let after = self.here();
                if let Op::Fork(st, _, t) = &mut self.code[fork] {
                    *st = starts;
                    *t = after;
                }
            }
            StmtKind::Assign { .. } => {
                self.emit(Op::Assign(s));
            }
            StmtKind::Expr(x) => {
                let is_task = matches!(self.d.expr(x).kind, ExprKind::Call(sub) if self.d.sub(sub).kind == SubKind::Task);
                self.emit(if is_task { Op::CallTask(x) } else { Op::Expr(x) });
            }
            StmtKind::If { cond, then, else_, .. } => {
                let j = self.emit(Op::JumpIfNot(cond, 0));
                self.stmt(then);
                match else_ {
                    Some(e) => {
                        let over = self.emit(Op::Jump(0));
                        let h = self.here();
                        self.patch(j, h);
                        self.stmt(e);
                        let h = self.here();
                        self.patch(over, h);
                    }
                    None => {
                        let h = self.here();
                        self.patch(j, h);
                    }
                }
            }
            StmtKind::Case { items, default, .. } => {
                let at = self.emit(Op::Case(s, vec![], 0));
                let mut targets = Vec::new();
                let mut ends = Vec::new();
                for (_, body) in &items {
                    targets.push(self.here());
                    self.stmt(*body);
                    ends.push(self.emit(Op::Jump(0)));
                }
                let dflt = self.here();
                if let Some(d) = default {
                    self.stmt(d);
                }
                let end = self.here();
                for e in ends {
                    self.patch(e, end);
                }
                if let Op::Case(_, t, dd) = &mut self.code[at] {
                    *t = targets;
                    *dd = dflt;
                }
            }
            StmtKind::Forever(b) => {
                let start = self.here();
                self.loops.push((vec![], vec![]));
                self.stmt(b);
                self.emit(Op::Jump(start));
                self.close_loop(start);
            }
            StmtKind::While(c, b) => {
                let start = self.here();
                let j = self.emit(Op::JumpIfNot(c, 0));
                self.loops.push((vec![j], vec![]));
                self.stmt(b);
                self.emit(Op::Jump(start));
                self.close_loop(start);
            }
            StmtKind::DoWhile(b, c) => {
                let start = self.here();
                self.loops.push((vec![], vec![]));
                self.stmt(b);
                let test = self.here();
                let j = self.emit(Op::JumpIfNot(c, 0));
                self.emit(Op::Jump(start));
                self.loops.last_mut().unwrap().0.push(j);
                self.close_loop(test);
            }
            StmtKind::Repeat(n, b) => {
                self.emit(Op::RepeatInit(n));
                let start = self.here();
                let j = self.emit(Op::RepeatNext(0));
                self.loops.push((vec![j], vec![]));
                self.stmt(b);
                self.emit(Op::Jump(start));
                // `break` leaves the count on the stack; RepeatNext pops it
                // only on the normal exit, so a break jumps to a pop.
                self.close_loop(start);
            }
            StmtKind::For { scope, init, cond, step, body } => {
                if let Some(sc) = scope {
                    self.emit(Op::Enter(sc));
                }
                for i in init {
                    self.stmt(i);
                }
                let start = self.here();
                let j = cond.map(|c| self.emit(Op::JumpIfNot(c, 0)));
                self.loops.push((j.into_iter().collect(), vec![]));
                self.stmt(body);
                let cont = self.here();
                for st in step {
                    self.stmt(st);
                }
                self.emit(Op::Jump(start));
                self.close_loop(cont);
            }
            StmtKind::Foreach { body, scope, .. } => {
                self.emit(Op::Enter(scope));
                self.emit(Op::ForeachInit(s));
                let start = self.here();
                let j = self.emit(Op::ForeachNext(s, 0));
                self.loops.push((vec![j], vec![]));
                self.stmt(body);
                self.emit(Op::Jump(start));
                self.close_loop(start);
            }
            StmtKind::Timed(_, b) => {
                self.emit(Op::Wait(s));
                self.stmt(b);
            }
            StmtKind::Wait(c, b) => {
                self.emit(Op::WaitCond(c));
                self.stmt(b);
            }
            StmtKind::WaitFork => {
                self.emit(Op::WaitFork);
            }
            StmtKind::DisableFork => {
                self.emit(Op::DisableFork);
            }
            StmtKind::Return(x) => {
                self.emit(Op::Return(x));
            }
            StmtKind::Break => {
                let j = self.emit(Op::Jump(0));
                if let Some(l) = self.loops.last_mut() {
                    l.0.push(j);
                }
            }
            StmtKind::Continue => {
                let j = self.emit(Op::Jump(0));
                if let Some(l) = self.loops.last_mut() {
                    l.1.push(j);
                }
            }
            StmtKind::Disable(x) => {
                // A block this code is inside: jump to its end.
                let target = match &self.d.expr(x).kind {
                    ExprKind::Hier { path, .. } if path.len() == 1 => {
                        let name = path[0];
                        self.blocks.iter().rposition(|(sc, _)| sc.is_some_and(|sc| self.d.scope(sc).name == Some(name)))
                    }
                    _ => None,
                };
                let at = self.emit(Op::Disable(x, target.map(|_| 0)));
                if let Some(i) = target {
                    self.blocks[i].1.push(at);
                }
            }
            StmtKind::Trigger { .. } => {
                self.emit(Op::Trigger(s));
            }
            StmtKind::ProcContinuous { .. } => {
                self.emit(Op::ProcCont(s));
            }
            StmtKind::Assert { pass, fail, .. } => {
                let at = self.emit(Op::Assert(s, None, 0));
                if let Some(p) = pass {
                    self.stmt(p);
                }
                let skip = self.emit(Op::Jump(0));
                let fail_at = self.here();
                if let Some(f) = fail {
                    self.stmt(f);
                }
                let end = self.here();
                self.patch(skip, end);
                if let Op::Assert(_, fa, after) = &mut self.code[at] {
                    *fa = fail.map(|_| fail_at);
                    *after = end;
                }
            }
            StmtKind::Opaque => {
                self.emit(Op::Unsupported(s));
            }
        }
    }

    /// Patches a loop's `break`s to here and its `continue`s to `cont`.
    fn close_loop(&mut self, cont: u32) {
        let (breaks, conts) = self.loops.pop().unwrap();
        let end = self.here();
        for b in breaks {
            // A `repeat` keeps its count on the stack; its normal exit pops
            // it, and a break must too.
            self.patch(b, end);
        }
        for c in conts {
            self.patch(c, cont);
        }
    }
}
