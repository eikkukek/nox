    use proc_macro::TokenStream;

    use quote::quote;
    use syn::{DeriveInput, parse_macro_input, spanned::Spanned, punctuated::Punctuated};
    use find_crate::find_crate;

    use crate::generics::GenericIdents;

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
                if let Err(_) =input.parse::<syn::Token![+]>() {
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
        let generics = &input.generics;
        let generic_idents: GenericIdents = generics.into();
        let where_clause = &generics.where_clause;
        let wrap =
        if let Some(attr) = input.attrs
            .iter()
            .find(|attr| {
                if let Some(ident) = attr.path().get_ident() {
                    ident == "wrapped"
                } else {
                    false
                }
            })
        {
            let bounds = attr.parse_args::<syn::Expr>()?;
            quote! { #bounds }
        } else {
            quote! { self }
        };
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
            let crate_path = find_crate(|c| c == "nox-mem" || c == "nox")
                .map_err(|err| syn::Error::new(attr.path().span(), format!("failed to find nox-mem crate {err}")))?
                .name;
            let pkg_path = if crate_path == "nox" {
                quote! { nox::mem }
            } else {
                quote! { nox_mem }
            };
            Ok(TokenStream::from(
                quote! {
                    unsafe impl #generics #pkg_path::dynamic::Dyn for #name #generic_idents #where_clause {

                        type Target = dyn #bounds;
                        
                        unsafe fn raw_parts(&self) -> #pkg_path::dynamic::DynRawParts<Self> {
                            let s: &Self::Target = #wrap;
                            let (data, vtable) = unsafe { core::mem::transmute::<*const Self::Target, (*const Self, *const ())>(s) };
                            #pkg_path::dynamic::DynRawParts {
                                data,
                                vtable,
                            }
                        }

                        unsafe fn from_raw_parts_mut<'a>(raw_parts: #pkg_path::dynamic::DynRawParts<Self>) -> &'a mut Self::Target {
                            unsafe { core::mem
                                ::transmute::<(*const Self, *const ()), *mut Self::Target>((raw_parts.data, raw_parts.vtable))
                                .as_mut()
                                .unwrap()
                            }
                        }

                        unsafe fn read_self(target: *mut Self::Target) -> Self {
                            unsafe { core::mem
                                ::transmute::<*mut Self::Target, (*const Self, *const ())>(target)
                                .0.read()
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
