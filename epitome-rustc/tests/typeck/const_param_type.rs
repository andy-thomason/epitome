//! A const argument's suffix is the type its parameter says; checked
//! against `core` from rust-src.
#![no_std]
#![feature(adt_const_params, min_generic_const_args, generic_const_parameter_types, unsized_const_params)]
#![allow(incomplete_features)]

use core::marker::ConstParamTy_;

fn foo<T: ConstParamTy_, const N: T>() {}

fn uses() {
    // `const N: T` with `N` written `2_i32`: `T` is `i32`.
    foo::<_, 2_i32>();
}
