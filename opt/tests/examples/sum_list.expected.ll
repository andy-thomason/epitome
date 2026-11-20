; @sum: recursion: 1 accumulator call(s) now jump back to the start, folding with add
; @sum: combine: add_zero x1
; @sum: rotate: 1 loops rotated
; @sum: simplifycfg: 0 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped


%node = type { i32, ptr }

@n3 = global %node { i32 30, ptr null }
@n2 = global %node { i32 20, ptr @n3 }
@n1 = global %node { i32 10, ptr @n2 }

define i32 @sum(ptr %list.arg, i32 %depth) {
entry.rec:
  %end1 = icmp eq ptr %list.arg, null
  br i1 %end1, label %empty, label %more

more:                                             ; preds = %more, %entry.rec
  %acc = phi i32 [ 0, %entry.rec ], [ %total, %more ]
  %list = phi ptr [ %list.arg, %entry.rec ], [ %next, %more ]
  %value = load i32, ptr %list
  %next.addr = getelementptr inbounds %node, ptr %list, i32 0, i32 1
  %next = load ptr, ptr %next.addr
  %scaled = mul i32 %value, %depth
  %total = add i32 %acc, %scaled
  %end = icmp eq ptr %next, null
  br i1 %end, label %empty, label %more

empty:                                            ; preds = %more, %entry.rec
  %acc1 = phi i32 [ 0, %entry.rec ], [ %total, %more ]
  ret i32 %acc1
}

define i32 @main() {
entry:
  %x = tail call i32 @sum(ptr @n1, i32 2)
  ret i32 %x
}
