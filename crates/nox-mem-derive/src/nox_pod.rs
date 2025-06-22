use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

pub fn nox_pod(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let expanded = quote! {
        impl MaybePod for #name {

            fn is_pod() -> bool { true }

            fn as_pod(&self) -> &Pod<Self> {
                unsafe {
                    & *(self as *const Self as *const Pod<Self>)
                }
            }

            fn as_mut_pod(&mut self) -> &mut Pod<Self> {
                unsafe {
                    &mut *(self as *mut Self as *mut Pod<Self>)
                }
            }
        }
    };
    TokenStream::from(expanded)
}
