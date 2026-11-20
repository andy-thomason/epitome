; cycle: @fail -> @write -> @unwrap -> @hint -> @fail   (SCC of 4: @fail, @hint, @unwrap, @write)
; @fail: recursion: the cycle splits into pieces of 1, 1, 1, 1
; @fail: inlined @write: cost 1, bonus 9, threshold 100
; @fail: combine: (folded) x1
; @hint: inlined @checks: cost 0, bonus 10, threshold 100
; @hint: combine: (folded) x1, br_false x1
; @hint: simplifycfg: 1 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 0 edges into unreachable dropped
; @unwrap: inlined @hint: cost 0, bonus 10, threshold 0
; @unwrap: simplifycfg: 2 unreachable blocks removed, 0 empty blocks bypassed, 0 selects made, 1 blocks merged, 1 edges into unreachable dropped
; @write: inlined @unwrap: cost 0, bonus 7, threshold 100
; @write: inlined @unwrap: cost 0, bonus 7, threshold 100
; @write: combine: add_self x1
; @main: inlined @write: cost 1, bonus 9, threshold 100
; @main: combine: (folded) x1
; @checks: deleted, no longer used
; @fail: deleted, no longer used
; @hint: deleted, no longer used
; @unwrap: deleted, no longer used


@out = global i32 0

define i32 @write(i32 %x, i1 %s) {
  %r = shl i32 %x, 1
  ret i32 %r
}

define i32 @main() {
  ret i32 10
}
