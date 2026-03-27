#[macro_use]
mod util;
mod r#dyn;
mod as_raw;
mod display;
mod error;
mod nox_ash_structure;
mod build_structure;
mod vertex_input;
mod vk_to_rust_enum;
mod snake_case;

extern crate proc_macro;

use proc_macro::TokenStream;

/// Derive macro for [Dyn]
#[proc_macro_derive(Dyn, attributes(bounds))]
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

/// Derive macro for [`ash`]-like structures that implement [`ash::vk::TaggedStructure`].
#[proc_macro_derive(Structure)]
pub fn nox_ash_structure(item: TokenStream) -> TokenStream {
    nox_ash_structure::nox_ash_structure(item)
}

/// Derive macro for [VertexInput]
#[proc_macro_derive(VertexInput)]
pub fn vertex_input(item: TokenStream) -> TokenStream {
    vertex_input::vertex_input(item)
}

#[proc_macro]
pub fn vk_to_rust_enum(item: TokenStream) -> TokenStream {
    vk_to_rust_enum::vk_to_rust_enum(item)
}

#[proc_macro]
pub fn snake_case(item: TokenStream) -> TokenStream {
    snake_case::snake_case(item)
}

#[proc_macro_derive(BuildStructure, attributes(skip, default))]
pub fn build_structure(item: TokenStream) -> TokenStream {
    build_structure::build_structure(item)
}
