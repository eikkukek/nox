mod nox_as_raw;
mod nox_vertex_input;
mod nox_error;

extern crate proc_macro;

use proc_macro::TokenStream;

/// A derive macro for [`AsRaw`]
#[proc_macro_derive(AsRaw)]
pub fn nox_as_raw(item: TokenStream) -> TokenStream {
    nox_as_raw::nox_as_raw(item)
}

/// A derive macro for [`VertexInput`]
#[proc_macro_derive(VertexInput)]
pub fn nox_vertex_input(item: TokenStream) -> TokenStream {
    nox_vertex_input::nox_vertex_input(item)
}

/// A derive macro for [`Error`]
#[proc_macro_derive(Error, attributes(display, source, from, any))]
pub fn nox_error(item: TokenStream) -> TokenStream {
    nox_error::nox_error(item)
}
