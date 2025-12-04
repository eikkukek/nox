use core::fmt::Write;

use proc_macro::{TokenStream};
use proc_macro2::Span;
use syn::{Data, DeriveInput, Error, Ident, parse_macro_input, spanned::Spanned};
use quote::{quote};

fn handle_enum(input: &DeriveInput, e: &syn::DataEnum) -> syn::Result<TokenStream> {
    let vars: Vec<_> = e
        .variants
        .iter()
        .collect();
    let mut display = Vec::new();
    for (i, var) in vars.iter().enumerate() {
        if let Some(attr) = var
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("display"))
        {
            display.push((i, attr));
        }
    }
    let name = &input.ident;
    let mut impl_display = quote! {};
    if !display.is_empty() {
        let mut err = false;
        let fmts: Vec<_> = display
            .iter()
            .map(|&(idx, attr)| {
                let var = vars[idx];
                let name = &var.ident;
                if let Ok(fmt) = attr.parse_args::<syn::LitStr>() {
                    let fmt = fmt.value();
                    if var.fields.is_empty() {
                        quote! { Self::#name => write!(f, #fmt) }
                    } else {
                        if var.fields
                            .iter()
                            .find(|f| f.ident.is_some())
                            .is_some()
                        {
                            let names: Vec<_> = var.fields
                                .iter()
                                .map(|f| {
                                    &f.ident
                                })
                                .collect();
                            quote! {
                                Self::#name { #( #names ),* } => write!(f, #fmt)
                            }
                        } else {
                            let names: Vec<_> = var.fields
                                .iter()
                                .enumerate()
                                .map(|(i, _)| {
                                    let mut name = String::new();
                                    write!(&mut name, "_{i}").ok();
                                    syn::Ident::new(&name, Span::call_site())
                                })
                                .collect();
                            let mut args = Vec::new();
                            for (i, name) in names.iter().enumerate() {
                                let mut search_for = String::new();
                                write!(&mut search_for, "{{{i}}}").ok();
                                let mut substr = &fmt[..];
                                let len = search_for.len();
                                while let Some(idx) = substr.find(&search_for) {
                                    substr = &fmt[idx + len - 1..];
                                    args.push(name);
                                }
                            }
                            quote! { Self::#name(#( #names ),*) => write!(f, #fmt, #( #args ),*) }
                        }
                    }
                } else {
                    err = true;
                    Default::default()
                }
            })
            .collect();
        if err {
            return Err(syn::Error::new(Span::call_site(),
                "failed to parse display attribute"
            ))
        }
        impl_display = quote! {
            impl core::fmt::Display for #name {
                #[allow(unused_variables)]
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        #( #fmts, )*
                        _ => Ok(()),
                    }
                }
            }
        };
    }
    let mut err: Option<(Span, &str)> = None;
    let mut from = Vec::new();
    let err_sources: Vec<_> = e.variants
        .iter()
        .map(|var| {
            let var_name = &var.ident;
            let mut source = None;
            for (i, field) in var.fields.iter().enumerate() {
                for attr in &field.attrs {
                    if attr.path().is_ident("source") {
                        if let Ok(expr) = attr.parse_args::<syn::Expr>() {
                            source = Some((i, Some(expr)));
                        } else {
                            source = Some((i, None));
                        }
                    } else if attr.path().is_ident("from") {
                        if var.fields.len() > 1 {
                            err = Some((
                                attr.span(),
                                "'from' attribute can only be used with variants with one field"
                            ))
                        } else {
                            let field = var.fields.iter().next().unwrap();
                            let ty = match &field.ty {
                                syn::Type::Path(path) => {
                                    Some(path)
                                },
                                _ => None,
                            };
                            if let Some(ty) = ty {
                                if let Some(ident) = &field.ident {
                                    from.push(quote! {
                                        impl From<#ty> for #name {

                                            fn from(value: #ty) -> Self {
                                                Self::#var_name { #ident: value, }
                                            }
                                        }
                                    });
                                } else {
                                    from.push(quote! {
                                        impl From<#ty> for #name {

                                            fn from(value: #ty) -> Self {
                                                Self::#var_name(value)
                                            }
                                        }
                                    });
                                }
                            } else {
                                err = Some((attr.span(), "unsupported 'source' type"));
                            }
                        }
                    }
                }
            }
            if let Some((idx, expr)) = source {
                let mut named = false;
                let names: Vec<_> = var.fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        if let Some(ident) = &field.ident {
                            named = true;
                            ident.clone()
                        } else {
                            if i == idx {
                                Ident::new("err", field.span())
                            } else {
                                Ident::new("_", field.span())
                            }
                        }
                    })
                    .collect();
                if let Some(expr) = expr {
                    if named {
                        quote! {
                            Self::#var_name { #(#names),* } => #expr,
                        }
                    } else {
                        quote! {
                            Self::#var_name(#(#names),*) => #expr,
                        }
                    }
                } else {
                    let err_name = &names[idx];
                    if named {
                        quote! {
                            Self::#var_name { #(#names),* } => Some(#err_name),
                        }
                    } else {
                        quote! {
                            Self::#var_name(#(#names),*) => Some(#err_name),
                        }
                    }
                }
            } else {
                quote! {}
            }
        })
        .collect();
    if let Some((span, err)) = err {
        return Err(syn::Error::new(span, err))
    }
    let impl_error = quote! {
        impl core::error::Error for #name {
            fn source(&self) -> Option<&(dyn error::Error + 'static)> {
                match self {
                        #( #err_sources )*
                    _ => None,
                }
            }
        }
    };
    let impl_from = quote! {
        #(#from)*
    };
    let expanded = quote! {
        #impl_display
        #impl_error
        #impl_from
    };
    Ok(TokenStream::from(expanded))
}

pub fn nox_error(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    match &input.data {
        Data::Enum(e) => {
             match handle_enum(&input, e) {
                 Ok(ts) => ts,
                 Err(err) => err.to_compile_error().into(),
             }
        },
        _ => {
            let err = Error::new_spanned(&input, "Expected enum");
            return err.to_compile_error().into()
        }
    }
}
