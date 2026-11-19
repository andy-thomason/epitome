; @fib: recursion: 2 call(s) now push a frame on an explicit stack (values saved per frame: 1, 1)
; @fib: combine: add_add_const x1, add_zero x1, sub_const x3
; @fib: implied: 0 comparisons folded, 0 selects resolved, 1 flags set
; @fib: cse: 2 merged, 0 loads reused, 0 stores forwarded


define i32 @fib(i32 %n.arg) {
entry.rec:
  %frames = alloca { ptr, ptr, [64 x { i32, i32 }] }
  store ptr null, ptr %frames
  br label %rec.header

rec.header:                                       ; preds = %rec.pop0, %recur, %entry.rec
  %n = phi i32 [ %n.arg, %entry.rec ], [ %n1, %recur ], [ %n2, %rec.pop0 ]
  %chunk.top = phi ptr [ %frames, %entry.rec ], [ %chunk, %recur ], [ %chunk.pop, %rec.pop0 ]
  %index.top = phi i32 [ 0, %entry.rec ], [ %index.next, %recur ], [ %index.pop, %rec.pop0 ]
  %chunk.full = icmp eq i32 %index.top, 64
  br i1 %chunk.full, label %rec.grow, label %entry

rec.grow:                                         ; preds = %rec.header
  %sp = call ptr @llvm.stacksave.p0()
  %chunk.new = alloca { ptr, ptr, [64 x { i32, i32 }] }
  store ptr %chunk.top, ptr %chunk.new
  %chunk.sp = getelementptr inbounds { ptr, ptr, [64 x { i32, i32 }] }, ptr %chunk.new, i32 0, i32 1
  store ptr %sp, ptr %chunk.sp
  br label %entry

entry:                                            ; preds = %rec.grow, %rec.header
  %chunk = phi ptr [ %chunk.top, %rec.header ], [ %chunk.new, %rec.grow ]
  %index = phi i32 [ %index.top, %rec.header ], [ 0, %rec.grow ]
  %small = icmp slt i32 %n, 2
  br i1 %small, label %done, label %recur

recur:                                            ; preds = %entry
  %n1 = add nsw i32 %n, -1
  %frame0 = getelementptr inbounds { ptr, ptr, [64 x { i32, i32 }] }, ptr %chunk, i32 0, i32 2, i32 %index
  store i32 0, ptr %frame0
  %frame.slot = getelementptr inbounds { i32, i32 }, ptr %frame0, i32 0, i32 1
  store i32 %n, ptr %frame.slot
  %index.next = add nuw i32 %index, 1
  br label %rec.header

done:                                             ; preds = %rec.pop1, %entry
  %chunk1 = phi ptr [ %chunk, %entry ], [ %chunk.pop, %rec.pop1 ]
  %index1 = phi i32 [ %index, %entry ], [ %index.prev, %rec.pop1 ]
  %r = phi i32 [ %n, %entry ], [ %sum, %rec.pop1 ]
  %chunk.empty = icmp eq i32 %index1, 0
  br i1 %chunk.empty, label %rec.underflow, label %rec.pop

rec.underflow:                                    ; preds = %done
  %chunk.prev = load ptr, ptr %chunk1
  %stack.empty = icmp eq ptr %chunk.prev, null
  br i1 %stack.empty, label %return, label %rec.shrink

return:                                           ; preds = %rec.underflow
  ret i32 %r

rec.shrink:                                       ; preds = %rec.underflow
  %chunk.sp1 = getelementptr inbounds { ptr, ptr, [64 x { i32, i32 }] }, ptr %chunk1, i32 0, i32 1
  %saved.sp = load ptr, ptr %chunk.sp1
  call void @llvm.stackrestore.p0(ptr %saved.sp)
  br label %rec.pop

rec.pop:                                          ; preds = %rec.shrink, %done
  %chunk.pop = phi ptr [ %chunk1, %done ], [ %chunk.prev, %rec.shrink ]
  %index.pop = phi i32 [ %index1, %done ], [ 64, %rec.shrink ]
  %index.prev = add i32 %index.pop, -1
  %frame = getelementptr inbounds { ptr, ptr, [64 x { i32, i32 }] }, ptr %chunk.pop, i32 0, i32 2, i32 %index.prev
  %site = load i32, ptr %frame
  switch i32 %site, label %rec.pop1 [
    i32 0, label %rec.pop0
  ]

rec.pop0:                                         ; preds = %rec.pop
  %frame.slot2 = getelementptr inbounds { i32, i32 }, ptr %frame, i32 0, i32 1
  %n.saved = load i32, ptr %frame.slot2
  %n2 = add i32 %n.saved, -2
  store i32 1, ptr %frame
  store i32 %r, ptr %frame.slot2
  br label %rec.header

rec.pop1:                                         ; preds = %rec.pop
  %frame.slot3 = getelementptr inbounds { i32, i32 }, ptr %frame, i32 0, i32 1
  %a.saved = load i32, ptr %frame.slot3
  %sum = add i32 %a.saved, %r
  br label %done
}

define i32 @main() {
entry:
  %x = tail call i32 @fib(i32 20)
  ret i32 %x
}

declare ptr @llvm.stacksave.p0()

declare void @llvm.stackrestore.p0(ptr)
