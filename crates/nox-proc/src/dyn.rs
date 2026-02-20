use proc_macro::TokenStream;

use quote::quote;
use syn::{DeriveInput, parse_macro_input, spanned::Spanned, punctuated::Punctuated};

struct Bounds {
    bounds: Punctuated<syn::TypeParamBound, syn::Token![+]>,
}

impl quote::ToTokens for Bounds {

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.bounds.to_tokens(tokens);
    }
}

impl syn::parse::Parse for Bounds {

    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut bounds = Punctuated::new();
        while let Ok(arg) = input.parse::<syn::TypeParamBound>() {
            bounds.push(arg);
            if input.parse::<syn::Token![+]>().is_err() {
                break;
            }
        }
        Ok(Self {
            bounds,
        })
    }
}

pub fn handle_input(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    if let Some(attr) = input.attrs
        .iter()
        .find(|attr| {
            if let Some(ident) = attr.path().get_ident() {
                ident == "bounds"
            } else {
                false
            }
        })
    {
        let bounds = attr.parse_args::<Bounds>()?;
        Ok(TokenStream::from(
            quote! {
                unsafe impl #impl_generics nox_mem::dynamic::Dyn<dyn #bounds> for
                    #name #ty_generics #where_clause
                {
                    type Target = Self;

                    unsafe fn raw_parts(ptr: *const Self) -> nox_mem::dynamic::DynRawParts<Self::Target> {
                        let s: *const (dyn #bounds) = ptr;
                        unsafe {
                            core::mem::transmute::<
                                *const (dyn #bounds),
                                nox_mem::dynamic::DynRawParts<Self::Target>
                            >(s)
                        }
                    }

                    unsafe fn from_raw_parts(raw_parts: nox_mem::dynamic::DynRawParts<Self>) -> *const (dyn #bounds) {
                        unsafe { core::mem
                            ::transmute::<
                                nox_mem::dynamic::DynRawParts<Self::Target>,
                                *mut (dyn #bounds)
                            >(raw_parts)
                        }
                    }

                    unsafe fn get_self(target: *const (dyn #bounds)) -> *const Self {
                        unsafe { core::mem
                            ::transmute::<*const (dyn #bounds), (*const Self, *const ())>(target).0
                        }
                    }
                }
            }
        ))
    } else {
        Err(syn::Error::new(input.attrs.first().span(), "failed to find 'bounds' attribute"))
    }
}

pub fn r#dyn(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    match handle_input(&input) {
        Ok(ts) => ts,
        Err(err) => err.to_compile_error().into(),
    }
}
