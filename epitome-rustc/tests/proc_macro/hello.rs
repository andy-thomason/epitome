//! A small proc-macro crate for testing: built by rustc, run through the
//! toolchain's proc-macro server.

extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree};

/// `#[derive(Hello)]` on `struct S` gives `impl Hello for S { fn hello() ->
/// &'static str { "S" } }`.
#[proc_macro_derive(Hello)]
pub fn hello(input: TokenStream) -> TokenStream {
    let mut it = input.into_iter();
    let mut name = String::new();
    while let Some(t) = it.next() {
        if let TokenTree::Ident(i) = &t
            && (i.to_string() == "struct" || i.to_string() == "enum")
        {
            name = it.next().map(|n| n.to_string()).unwrap_or_default();
            break;
        }
    }
    format!("impl Hello for {name} {{ fn hello() -> &'static str {{ \"{name}\" }} }}").parse().unwrap()
}

/// `echo!(tokens)` gives the tokens back.
#[proc_macro]
pub fn echo(input: TokenStream) -> TokenStream {
    input
}

/// `#[twice] fn f() {}` gives the item twice, the second named `f_again`.
#[proc_macro_attribute]
pub fn twice(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let text = item.to_string();
    let again = text.replacen("fn ", "fn __again_", 1);
    format!("{text} {again}").parse().unwrap()
}

/// `show!(tokens)` gives a string literal of the tokens as the macro saw
/// them.
#[proc_macro]
pub fn show(input: TokenStream) -> TokenStream {
    format!("{:?}", input.to_string()).parse().unwrap()
}
