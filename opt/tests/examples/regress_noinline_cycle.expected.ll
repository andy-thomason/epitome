; @a: recursion: 1 call(s) now push a frame on an explicit stack (values saved per frame: 1)
; @a: recursion: cycle not collapsed: @b is noinline
; @a: combine: sub_const x2
; @a: simplifycfg: 0 unreachable blocks removed, 1 empty blocks bypassed, 0 selects made, 0 blocks merged, 0 edges into unreachable dropped
; @a: implied: 0 comparisons folded, 0 selects resolved, 1 flags set
; @b: recursion: 1 call(s) now push a frame on an explicit stack (values saved per frame: 1)
; @b: recursion: cycle not collapsed: @c is noinline
; @b: combine: sub_const x2
; @b: simplifycfg: 0 unreachable blocks removed, 1 empty blocks bypassed, 0 selects made, 0 blocks merged, 0 edges into unreachable dropped
; @b: implied: 0 comparisons folded, 0 selects resolved, 1 flags set
; @c: sroa: 1 allocas split into 1 pieces
; @c: mem2reg: 1 allocas promoted (1 loads, 1 stores removed)
; @main: not inlined @a: noinline
; @main: not inlined @b: noinline


@fmt = private constant [4 x i8] c"%d\0A\00"

declare i32 @printf(ptr, ...)

define internal i32 @a(i32 %n.arg) noinline {
entry.rec:
  %frames = alloca { ptr, ptr, [64 x { i32 }] }
  store ptr null, ptr %frames
  br label %rec.header

rec.header:                                       ; preds = %rec, %entry.rec
  %n = phi i32 [ %n.arg, %entry.rec ], [ %m, %rec ]
  %chunk.top = phi ptr [ %frames, %entry.rec ], [ %chunk, %rec ]
  %index.top = phi i32 [ 0, %entry.rec ], [ %index.next, %rec ]
  %chunk.full = icmp eq i32 %index.top, 64
  br i1 %chunk.full, label %rec.grow, label %entry

rec.grow:                                         ; preds = %rec.header
  %sp = call ptr @llvm.stacksave.p0()
  %chunk.new = alloca { ptr, ptr, [64 x { i32 }] }
  store ptr %chunk.top, ptr %chunk.new
  %chunk.sp = getelementptr inbounds { ptr, ptr, [64 x { i32 }] }, ptr %chunk.new, i32 0, i32 1
  store ptr %sp, ptr %chunk.sp
  br label %entry

entry:                                            ; preds = %rec.grow, %rec.header
  %chunk = phi ptr [ %chunk.top, %rec.header ], [ %chunk.new, %rec.grow ]
  %index = phi i32 [ %index.top, %rec.header ], [ 0, %rec.grow ]
  %c = icmp sle i32 %n, 0
  br i1 %c, label %rec.return, label %rec

rec:                                              ; preds = %entry
  %m = add nsw i32 %n, -1
  %x = call i32 @b(i32 %m)
  %frame0 = getelementptr inbounds { ptr, ptr, [64 x { i32 }] }, ptr %chunk, i32 0, i32 2, i32 %index
  store i32 %x, ptr %frame0
  %index.next = add nuw i32 %index, 1
  br label %rec.header

rec.return:                                       ; preds = %rec.pop, %entry
  %chunk1 = phi ptr [ %chunk.pop, %rec.pop ], [ %chunk, %entry ]
  %index1 = phi i32 [ %index.prev, %rec.pop ], [ %index, %entry ]
  %result = phi i32 [ %s2, %rec.pop ], [ 1, %entry ]
  %chunk.empty = icmp eq i32 %index1, 0
  br i1 %chunk.empty, label %rec.underflow, label %rec.pop

rec.underflow:                                    ; preds = %rec.return
  %chunk.prev = load ptr, ptr %chunk1
  %stack.empty = icmp eq ptr %chunk.prev, null
  br i1 %stack.empty, label %return, label %rec.shrink

return:                                           ; preds = %rec.underflow
  ret i32 %result

rec.shrink:                                       ; preds = %rec.underflow
  %chunk.sp1 = getelementptr inbounds { ptr, ptr, [64 x { i32 }] }, ptr %chunk1, i32 0, i32 1
  %saved.sp = load ptr, ptr %chunk.sp1
  call void @llvm.stackrestore.p0(ptr %saved.sp)
  br label %rec.pop

rec.pop:                                          ; preds = %rec.shrink, %rec.return
  %chunk.pop = phi ptr [ %chunk1, %rec.return ], [ %chunk.prev, %rec.shrink ]
  %index.pop = phi i32 [ %index1, %rec.return ], [ 64, %rec.shrink ]
  %index.prev = add i32 %index.pop, -1
  %frame = getelementptr inbounds { ptr, ptr, [64 x { i32 }] }, ptr %chunk.pop, i32 0, i32 2, i32 %index.prev
  %x.saved = load i32, ptr %frame
  %s = add i32 %x.saved, %result
  %s2 = urem i32 %s, 100003
  br label %rec.return
}

define internal i32 @b(i32 %n.arg) noinline {
entry.rec:
  %frames = alloca { ptr, ptr, [64 x { i32 }] }
  store ptr null, ptr %frames
  br label %rec.header

rec.header:                                       ; preds = %rec, %entry.rec
  %n = phi i32 [ %n.arg, %entry.rec ], [ %m, %rec ]
  %chunk.top = phi ptr [ %frames, %entry.rec ], [ %chunk, %rec ]
  %index.top = phi i32 [ 0, %entry.rec ], [ %index.next, %rec ]
  %chunk.full = icmp eq i32 %index.top, 64
  br i1 %chunk.full, label %rec.grow, label %entry

rec.grow:                                         ; preds = %rec.header
  %sp = call ptr @llvm.stacksave.p0()
  %chunk.new = alloca { ptr, ptr, [64 x { i32 }] }
  store ptr %chunk.top, ptr %chunk.new
  %chunk.sp = getelementptr inbounds { ptr, ptr, [64 x { i32 }] }, ptr %chunk.new, i32 0, i32 1
  store ptr %sp, ptr %chunk.sp
  br label %entry

entry:                                            ; preds = %rec.grow, %rec.header
  %chunk = phi ptr [ %chunk.top, %rec.header ], [ %chunk.new, %rec.grow ]
  %index = phi i32 [ %index.top, %rec.header ], [ 0, %rec.grow ]
  %c = icmp sle i32 %n, 0
  br i1 %c, label %rec.return, label %rec

rec:                                              ; preds = %entry
  %m = add nsw i32 %n, -1
  %x = call i32 @c(i32 %m)
  %y = mul i32 %x, 3
  %frame0 = getelementptr inbounds { ptr, ptr, [64 x { i32 }] }, ptr %chunk, i32 0, i32 2, i32 %index
  store i32 %y, ptr %frame0
  %index.next = add nuw i32 %index, 1
  br label %rec.header

rec.return:                                       ; preds = %rec.pop, %entry
  %chunk1 = phi ptr [ %chunk.pop, %rec.pop ], [ %chunk, %entry ]
  %index1 = phi i32 [ %index.prev, %rec.pop ], [ %index, %entry ]
  %result = phi i32 [ %s2, %rec.pop ], [ 2, %entry ]
  %chunk.empty = icmp eq i32 %index1, 0
  br i1 %chunk.empty, label %rec.underflow, label %rec.pop

rec.underflow:                                    ; preds = %rec.return
  %chunk.prev = load ptr, ptr %chunk1
  %stack.empty = icmp eq ptr %chunk.prev, null
  br i1 %stack.empty, label %return, label %rec.shrink

return:                                           ; preds = %rec.underflow
  ret i32 %result

rec.shrink:                                       ; preds = %rec.underflow
  %chunk.sp1 = getelementptr inbounds { ptr, ptr, [64 x { i32 }] }, ptr %chunk1, i32 0, i32 1
  %saved.sp = load ptr, ptr %chunk.sp1
  call void @llvm.stackrestore.p0(ptr %saved.sp)
  br label %rec.pop

rec.pop:                                          ; preds = %rec.shrink, %rec.return
  %chunk.pop = phi ptr [ %chunk1, %rec.return ], [ %chunk.prev, %rec.shrink ]
  %index.pop = phi i32 [ %index1, %rec.return ], [ 64, %rec.shrink ]
  %index.prev = add i32 %index.pop, -1
  %frame = getelementptr inbounds { ptr, ptr, [64 x { i32 }] }, ptr %chunk.pop, i32 0, i32 2, i32 %index.prev
  %y.saved = load i32, ptr %frame
  %s = add i32 %y.saved, %result
  %s2 = urem i32 %s, 100003
  br label %rec.return
}

define internal i32 @c(i32 %n) noinline {
entry:
  %x = tail call i32 @a(i32 %n)
  %s = add i32 %x, %n
  ret i32 %s
}

define i32 @main() {
  %v = tail call i32 @a(i32 15)
  %1 = tail call i32 (ptr, ...) @printf(ptr @fmt, i32 %v)
  %w = tail call i32 @b(i32 12)
  %2 = tail call i32 (ptr, ...) @printf(ptr @fmt, i32 %w)
  ret i32 0
}

declare ptr @llvm.stacksave.p0()

declare void @llvm.stackrestore.p0(ptr)
