mod nox_pod;

extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(Pod)]
pub fn nox_pod(item: TokenStream) -> TokenStream {
    nox_pod::nox_pod(item)
}
