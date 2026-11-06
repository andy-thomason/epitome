//! Control flow: labelled loops with values, `while let`, `match` with
//! guards, ranges, or-patterns and bindings, slice patterns, `let else`,
//! `if let` chains, `?` on `Option` and `Result`, and early returns.
#![no_std]
#![no_main]

unsafe extern "C" {
    fn putchar(c: i32) -> i32;
}

fn put_str(s: &str) {
    for &b in s.as_bytes() {
        unsafe {
            putchar(b as i32);
        }
    }
}

fn put_i64(n: i64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    let neg = n < 0;
    let mut m = if neg { 0u64.wrapping_sub(n as u64) } else { n as u64 };
    loop {
        buf[i] = b'0' + (m % 10) as u8;
        i += 1;
        m /= 10;
        if m == 0 {
            break;
        }
    }
    if neg {
        put_str("-");
    }
    while i > 0 {
        i -= 1;
        unsafe {
            putchar(buf[i] as i32);
        }
    }
    put_str("\n");
}

fn classify(n: i64) -> &'static str {
    match n {
        i64::MIN..=-1 => "negative",
        0 => "zero",
        1 | 2 | 3 => "small",
        x if x % 2 == 0 => "even",
        4..100 => "odd",
        _ => "big",
    }
}

fn find_pair(xs: &[i64], target: i64) -> Option<(usize, usize)> {
    let found = 'outer: {
        for i in 0..xs.len() {
            for j in i + 1..xs.len() {
                if xs[i] + xs[j] == target {
                    break 'outer Some((i, j));
                }
            }
        }
        None
    };
    found
}

fn first_square_over(limit: u64) -> u64 {
    let mut n = 0;
    loop {
        n += 1;
        if n * n > limit {
            break n * n;
        }
    }
}

fn digits(mut n: u64) -> u32 {
    let mut count = 0;
    'count: loop {
        count += 1;
        n /= 10;
        if n == 0 {
            break 'count;
        }
    }
    count
}

#[derive(Clone, Copy)]
enum Token {
    Num(i64),
    Plus,
    Minus,
    Times,
    End,
}

fn eval(tokens: &[Token]) -> Result<i64, &'static str> {
    let mut acc = match tokens.first() {
        Some(Token::Num(n)) => *n,
        _ => return Err("expected a number"),
    };
    let mut rest = &tokens[1..];
    loop {
        match rest {
            [Token::End, ..] | [] => return Ok(acc),
            [op @ (Token::Plus | Token::Minus | Token::Times), Token::Num(n), tail @ ..] => {
                acc = match op {
                    Token::Plus => acc + n,
                    Token::Minus => acc - n,
                    _ => acc * n,
                };
                rest = tail;
            }
            _ => return Err("bad expression"),
        }
    }
}

fn twice(tokens: &[Token]) -> Result<i64, &'static str> {
    let v = eval(tokens)?;
    Ok(v * 2)
}

fn add_firsts(a: &[i64], b: &[i64]) -> Option<i64> {
    let x = a.first()?;
    let y = b.first()?;
    Some(x + y)
}

fn describe(v: Option<Result<i64, i64>>) -> i64 {
    let Some(r) = v else { return -1 };
    if let Ok(x) = r
        && x > 10
    {
        return x * 100;
    }
    match r {
        Ok(x) => x,
        Err(e) => -e,
    }
}

fn countdown(mut stack: [u8; 4]) -> u32 {
    let mut len = 4;
    let mut total = 0;
    while let [.., last] = &stack[..len] {
        total += *last as u32;
        len -= 1;
        stack[0] = stack[0].wrapping_add(1);
    }
    total + stack[0] as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    for n in [-5, 0, 2, 8, 9, 101, 120] {
        put_str(classify(n));
        put_str("\n");
    }
    match find_pair(&[1, 4, 6, 9], 13) {
        Some((i, j)) => {
            put_i64(i as i64);
            put_i64(j as i64);
        }
        None => put_str("none\n"),
    }
    put_i64(find_pair(&[1, 2], 10).is_none() as i64);
    put_i64(first_square_over(50) as i64);
    put_i64(digits(0) as i64);
    put_i64(digits(123456) as i64);
    let prog = [Token::Num(3), Token::Plus, Token::Num(4), Token::Times, Token::Num(5), Token::End];
    match twice(&prog) {
        Ok(v) => put_i64(v),
        Err(e) => put_str(e),
    }
    match twice(&[Token::Plus]) {
        Ok(v) => put_i64(v),
        Err(e) => {
            put_str(e);
            put_str("\n");
        }
    }
    match eval(&[Token::Num(1), Token::Minus, Token::Minus]) {
        Ok(v) => put_i64(v),
        Err(e) => {
            put_str(e);
            put_str("\n");
        }
    }
    put_i64(add_firsts(&[5], &[6]).unwrap_or(0));
    put_i64(add_firsts(&[], &[6]).unwrap_or(-7));
    put_i64(describe(None));
    put_i64(describe(Some(Ok(4))));
    put_i64(describe(Some(Ok(40))));
    put_i64(describe(Some(Err(9))));
    put_i64(countdown([1, 2, 3, 4]) as i64);
    assert!(digits(7) == 1);
    assert!(first_square_over(10) == 16, "got {}", first_square_over(10));
    put_str("asserts held\n");
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
