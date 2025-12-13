#[macro_use]

mod generics;
mod input;
mod util;
mod r#dyn;
mod as_raw;
mod display;
mod error;
mod vertex_input;

extern crate proc_macro;

use proc_macro::TokenStream;

/// Derive macro for [Dyn]
#[proc_macro_derive(Dyn, attributes(wrapped, bounds))]
pub fn r#dyn(item: TokenStream) -> TokenStream {
    r#dyn::r#dyn(item)
}

/// Derive macro for [AsRaw]
#[proc_macro_derive(AsRaw)]
pub fn as_raw(item: TokenStream) -> TokenStream {
    as_raw::as_raw(item)
}

/// Derive macro for [Display]
#[proc_macro_derive(Display, attributes(display))]
pub fn display(item: TokenStream) -> TokenStream {
    display::display(item)
}

/// Derive macro for [Error]
#[proc_macro_derive(Error, attributes(display, source, from))]
pub fn error(item: TokenStream) -> TokenStream {
    error::error(item)
}

/// Derive macro for [VertexInput]
#[proc_macro_derive(VertexInput)]
pub fn vertex_input(item: TokenStream) -> TokenStream {
    vertex_input::vertex_input(item)
}
