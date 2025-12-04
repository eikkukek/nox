mod nox_as_raw;
mod nox_vertex_input;
mod nox_error;

extern crate proc_macro;

use proc_macro::TokenStream;

/// A derive macro for `AsRaw`
#[proc_macro_derive(AsRaw)]
pub fn nox_as_raw(item: TokenStream) -> TokenStream {
    nox_as_raw::nox_as_raw(item)
}

/// A derive macro for `VertexInput`
#[proc_macro_derive(VertexInput)]
pub fn nox_vertex_input(item: TokenStream) -> TokenStream {
    nox_vertex_input::nox_vertex_input(item)
}

#[proc_macro_attribute]
pub fn display_error(attr: TokenStream, _item: TokenStream) -> TokenStream {
    attr
}

#[proc_macro_attribute]
pub fn error_source(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// A derive macro for `Error`
#[proc_macro_derive(Error, attributes(display, source, from))]
pub fn nox_error(item: TokenStream) -> TokenStream {
    nox_error::nox_error(item)
}
