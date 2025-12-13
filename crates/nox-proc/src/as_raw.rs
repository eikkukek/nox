use proc_macro::{TokenStream};
use syn::{parse_macro_input, Ident, Data, DeriveInput, Error};
use quote::quote;

pub fn as_raw(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    match &input.data {
        Data::Enum(_) => (),
        _ => {
            let err = Error::new_spanned(&input, "Expected enum");
            return err.to_compile_error().into()
        }
    };
    let mut meta: Option<Ident> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            meta = match attr.parse_args::<syn::Ident>() {
                Ok(r) => Some(r),
                Err(_) => {
                    None
                },
            };
        }
    }
    if let Some(meta) = meta.take() {
        let name = &input.ident;
        let expanded = quote! {
            impl AsRaw for #name {

                type Repr = #meta;

                fn as_raw(self) -> Self::Repr {
                    self as Self::Repr
                }
            }

            impl From<#name> for #meta {

                fn from(value: #name) -> Self {
                    value as Self
                }
            }
        };
        TokenStream::from(expanded)
    }
    else {
        let err = Error::new_spanned(&input, "Expected repr(type)");
        err.to_compile_error().into()
    }
}
