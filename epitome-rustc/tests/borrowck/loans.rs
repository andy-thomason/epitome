// Loans: borrows that conflict with what is done while they are in use.
// `//~ E0xxx` says the error that line has; there are no others.
fn use_ref<T>(_: &T) {}
fn use_mut<T>(_: &mut T) {}
fn take<T>(_: T) {}
fn longest<'a>(a: &'a u8, _b: &u8) -> &'a u8 { a }

struct Pair { a: u32, b: u32 }
struct Holder<'a> { r: &'a mut u32 }

fn two_mut() {
    let mut x = 1;
    let r = &mut x;
    let s = &mut x; //~ E0499
    use_mut(r);
    use_mut(s);
}

fn shared_then_mut() {
    let mut x = 1;
    let r = &x;
    let s = &mut x; //~ E0502
    use_ref(r);
    use_mut(s);
}

fn assign_while_borrowed() {
    let mut x = 1;
    let r = &x;
    x = 2; //~ E0506
    use_ref(r);
}

fn move_while_borrowed() {
    let s = Pair { a: 1, b: 2 };
    let r = &s;
    take(s); //~ E0505
    use_ref(r);
}

fn read_while_mut_borrowed() {
    let mut x = 1;
    let r = &mut x;
    let y = x; //~ E0503
    use_mut(r);
    take(y);
}

fn nll_ends_the_borrow() {
    let mut x = 1;
    let r = &mut x;
    use_mut(r);
    let s = &mut x;
    use_mut(s);
}

fn disjoint_fields() {
    let mut p = Pair { a: 1, b: 2 };
    let a = &mut p.a;
    let b = &mut p.b;
    use_mut(a);
    use_mut(b);
}

fn same_field() {
    let mut p = Pair { a: 1, b: 2 };
    let a = &mut p.a;
    let b = &p.a; //~ E0502
    use_mut(a);
    use_ref(b);
}

fn reborrow_is_fine() {
    let mut x = 1;
    let r = &mut x;
    let s = &mut *r;
    use_mut(s);
    use_mut(r);
}

fn reborrow_then_use_outer() {
    let mut x = 1;
    let r = &mut x;
    let s = &mut *r;
    use_mut(r); //~ E0499
    use_mut(s);
}

fn dangling_in_scope() {
    let r;
    {
        let x = 1;
        r = &x; //~ E0597
    }
    use_ref(r);
}

fn return_local() -> &'static u32 {
    let x = 1;
    &x //~ E0515
}

fn return_param_data(x: &u32) -> &u32 { &*x }

fn signature_ties(a: &u8, b: &u8) -> u8 {
    let r = longest(a, b);
    *r
}

fn result_ties_to_first_only() {
    let mut y = 1u8;
    let x = 2u8;
    let r = longest(&x, &y);
    y = 3;
    use_ref(r);
    take(y);
}

fn result_ties_to_the_first() {
    let mut x = 1u8;
    let y = 2u8;
    let r = longest(&x, &y);
    x = 3; //~ E0506
    use_ref(r);
}

fn struct_holds_the_borrow() {
    let mut x = 1;
    let h = Holder { r: &mut x };
    x = 2; //~ E0506
    *h.r = 3;
}

fn closure_borrows() {
    let mut x = 1;
    let a = || x + 1;
    x = 5; //~ E0506
    take(a());
}

fn two_closures_mutate() {
    let mut x = 1;
    let mut a = || x += 1;
    let mut b = || x += 2; //~ E0499
    a();
    b();
}

fn closures_in_turn() {
    let mut x = 1;
    let mut a = || x += 1;
    a();
    let mut b = || x += 2;
    b();
}

fn closure_fields_are_disjoint() {
    let mut p = Pair { a: 1, b: 2 };
    let mut f = || p.a += 1;
    p.b = 3;
    f();
}

fn loop_reborrow(mut cur: &mut Pair, n: u32) {
    let mut i = 0;
    while i < n {
        let next = &mut cur.a;
        *next += 1;
        i += 1;
    }
    cur.b = 1;
}
