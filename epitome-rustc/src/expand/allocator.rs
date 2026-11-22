//! `#[global_allocator]`: the allocator shim. `alloc` reaches the global
//! allocator through functions it declares and the compiler supplies
//! (`__rust_alloc`, `__rust_dealloc`, `__rust_realloc`,
//! `__rust_alloc_zeroed`), with `__rust_no_alloc_shim_is_unstable_v2` and
//! `__rust_alloc_error_handler` beside them. Here they are written as Rust
//! after the static, each calling the static's `GlobalAlloc` method, and
//! `#[no_mangle]` so they are the functions `alloc` calls.

use crate::ast::{Ast, Kind, NodeId};

/// The shim for the static `item`, as source.
pub fn shim(ast: &Ast, item: NodeId) -> Result<String, String> {
    if ast.kind(item) != Kind::Static {
        return Err("`#[global_allocator]` applies only to statics".into());
    }
    let name = ast.child(item, Kind::Name).map(|n| ast.token(n).ident_name()).ok_or("a static without a name")?;
    let layout = "::core::alloc::Layout::from_size_align_unchecked";
    Ok(format!(
        "const _: () = {{
    #[unsafe(no_mangle)]
    unsafe fn __rust_alloc(size: usize, align: usize) -> *mut u8 {{
        unsafe {{ ::core::alloc::GlobalAlloc::alloc(&{name}, {layout}(size, align)) }}
    }}
    #[unsafe(no_mangle)]
    unsafe fn __rust_dealloc(ptr: *mut u8, size: usize, align: usize) {{
        unsafe {{ ::core::alloc::GlobalAlloc::dealloc(&{name}, ptr, {layout}(size, align)) }}
    }}
    #[unsafe(no_mangle)]
    unsafe fn __rust_realloc(ptr: *mut u8, size: usize, align: usize, new_size: usize) -> *mut u8 {{
        unsafe {{ ::core::alloc::GlobalAlloc::realloc(&{name}, ptr, {layout}(size, align), new_size) }}
    }}
    #[unsafe(no_mangle)]
    unsafe fn __rust_alloc_zeroed(size: usize, align: usize) -> *mut u8 {{
        unsafe {{ ::core::alloc::GlobalAlloc::alloc_zeroed(&{name}, {layout}(size, align)) }}
    }}
    #[unsafe(no_mangle)]
    fn __rust_no_alloc_shim_is_unstable_v2() {{}}
    #[unsafe(no_mangle)]
    fn __rust_alloc_error_handler(_size: usize, _align: usize) -> ! {{
        ::core::intrinsics::abort()
    }}
}};"
    ))
}
