; @walk: sroa: 1 allocas split into 1 pieces
; @walk: mem2reg: 1 allocas promoted (1 loads, 2 stores removed)
; @walk: recursion: 1 accumulator call(s) now jump back to the start, folding with add
; @walk: combine: add_zero x1, sub_const x1
; @walk: rotate: 1 loops rotated
; @walk: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @walk: indvars: loop %recur runs %n.arg times when entered
; @walk: indvars: 1 exact exit tests, 0 exit values, 0 counters widened
; @walk_marked: recursion: 1 call(s) now push a frame on an explicit stack (values saved per frame: 1)
; @walk_marked: combine: sub_const x1
; @walk_marked: rotate: 1 loops rotated
; @walk_marked: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @main: constcall: 1 calls worked out
; @main: combine: (constant to the right) x1


@seed = global i32 1

declare void @llvm.lifetime.start.p0(i64, ptr)

declare void @llvm.lifetime.end.p0(i64, ptr)

define void @fill(ptr %p) noinline {
  %v = load volatile i32, ptr @seed
  %at = getelementptr i32, ptr %p, i64 1
  store i32 %v, ptr %at
  ret void
}

define i32 @walk(i32 %n.arg) {
entry.rec:
  %stop1 = icmp eq i32 %n.arg, 0
  br i1 %stop1, label %done, label %recur

recur:                                            ; preds = %recur, %entry.rec
  %acc = phi i32 [ 0, %entry.rec ], [ %sum, %recur ]
  %n = phi i32 [ %n.arg, %entry.rec ], [ %m, %recur ]
  %m = add i32 %n, -1
  %sum = add i32 %acc, %n
  %stop = icmp eq i32 %m, 0
  br i1 %stop, label %done, label %recur

done:                                             ; preds = %recur, %entry.rec
  %acc1 = phi i32 [ 0, %entry.rec ], [ %sum, %recur ]
  ret i32 %acc1
}

define i32 @walk_marked(i32 %n.arg) {
entry.rec:
  br label %entry

entry:                                            ; preds = %recur, %entry.rec
  %n = phi i32 [ %n.arg, %entry.rec ], [ %m, %recur ]
  %frame.top = phi ptr [ null, %entry.rec ], [ %frame0, %recur ]
  %buf = alloca [4 x i32]
  call void @llvm.lifetime.start.p0(i64 16, ptr %buf)
  store i32 %n, ptr %buf
  call void @fill(ptr %buf)
  %stop = icmp eq i32 %n, 0
  br i1 %stop, label %done, label %recur

recur:                                            ; preds = %entry
  %m = add i32 %n, -1
  %sp = call ptr @llvm.stacksave.p0()
  %frame0 = alloca { ptr, ptr, ptr }
  store ptr %frame.top, ptr %frame0
  %frame.sp = getelementptr inbounds { ptr, ptr, ptr }, ptr %frame0, i32 0, i32 1
  store ptr %sp, ptr %frame.sp
  %frame.slot = getelementptr inbounds { ptr, ptr, ptr }, ptr %frame0, i32 0, i32 2
  store ptr %buf, ptr %frame.slot
  br label %entry

done:                                             ; preds = %entry
  call void @llvm.lifetime.end.p0(i64 16, ptr %buf)
  %stack.empty1 = icmp eq ptr %frame.top, null
  br i1 %stack.empty1, label %return, label %rec.pop

return:                                           ; preds = %rec.pop, %done
  %result = phi i32 [ 0, %done ], [ %sum, %rec.pop ]
  ret i32 %result

rec.pop:                                          ; preds = %rec.pop, %done
  %result1 = phi i32 [ 0, %done ], [ %sum, %rec.pop ]
  %frame = phi ptr [ %frame.top, %done ], [ %frame.prev, %rec.pop ]
  %frame.prev = load ptr, ptr %frame
  %frame.sp1 = getelementptr inbounds { ptr, ptr }, ptr %frame, i32 0, i32 1
  %saved.sp = load ptr, ptr %frame.sp1
  %frame.slot1 = getelementptr inbounds { ptr, ptr, ptr }, ptr %frame, i32 0, i32 2
  %buf.saved = load ptr, ptr %frame.slot1
  call void @llvm.stackrestore.p0(ptr %saved.sp)
  %mine = load i32, ptr %buf.saved
  %sum = add i32 %result1, %mine
  %stack.empty = icmp eq ptr %frame.prev, null
  br i1 %stack.empty, label %return, label %rec.pop
}

define i32 @main() {
entry:
  %y = tail call i32 @walk_marked(i32 10)
  %r = add i32 %y, 55
  ret i32 %r
}

declare ptr @llvm.stacksave.p0()

declare void @llvm.stackrestore.p0(ptr)
