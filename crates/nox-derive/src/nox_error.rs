use core::fmt::Write;

use find_crate::find_crate;
use proc_macro::{TokenStream};
use proc_macro2::Span;
use syn::{Data, DeriveInput, Ident, LitStr, parse_macro_input, spanned::Spanned};
use quote::{quote};

#[inline(always)]
fn impl_into_any(input: &DeriveInput) -> syn::Result<Option<proc_macro2::TokenStream>> {
    if let Some(attr) = input.attrs
        .iter()
        .find(|attr| {
            if let Some(ident) = attr.path().get_ident() {
                ident == "any"
            } else {
                false
            }
        })
    {
        let msg = attr.parse_args::<syn::LitStr>()?;
        let name = &input.ident;
        let crate_path = find_crate(|c| c == "nox-error" || c == "nox")
            .map_err(|err| syn::Error::new(attr.path().span(), format!("failed to find nox-error crate {err}")))?
            .name;
        let pkg_path = if crate_path == "nox" {
            Ident::new("nox::error", attr.path().span())
        } else {
            Ident::new("nox_error", attr.path().span())
        };
        Ok(Some(quote! {
            impl From<#name> for #pkg_path::any::AnyError {

                fn from(value: #name) -> Self {
                    Self::new(#msg, value)
                }
            }
            impl From<#name> for #pkg_path::any::SomeError<#name> {

                fn from(value: #name) -> Self {
                    Self::new(#msg, value)
                }
            }
        }))

    } else {
        Ok(None)
    }
}

#[inline(always)]
fn handle_enum(input: &DeriveInput, e: &syn::DataEnum) -> syn::Result<TokenStream> {
    let mut display = Vec::new();
    for (i, var) in e.variants.iter().enumerate() {
        if let Some(attr) = var
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("display"))
        {
            display.push((i, attr));
        }
    }
    let name = &input.ident;
    let mut impl_display = Default::default();
    if !display.is_empty() {
        let mut err = None;
        let fmts: Vec<_> = display
            .iter()
            .map(|&(idx, attr)| {
                let var = &e.variants[idx];
                let name = &var.ident;
                if let Ok(fmt) = attr.parse_args::<syn::LitStr>() {
                    let mut fmt = fmt.value();
                    if var.fields.is_empty() {
                        quote! { Self::#name => write!(f, #fmt) }
                    } else {
                        if var.fields
                            .iter()
                            .find(|f| f.ident.is_some())
                            .is_some()
                        {
                            let names = var.fields
                                .iter()
                                .map(|f| {
                                    &f.ident
                                });
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
                            let mut indices = Vec::new();
                            for (i, _) in names.iter().enumerate() {
                                let mut search_for = String::new();
                                write!(&mut search_for, "{{{i}}}").ok();
                                let mut substr = &fmt[..];
                                let len = search_for.len();
                                let mut off = 0;
                                while let Some(idx) = substr.find(&search_for) {
                                    substr = &substr[idx + len..];
                                    off += idx;
                                    indices.push(off + 1);
                                    off += len;
                                }
                            }
                            indices.sort();
                            for &idx in indices.iter().rev() {
                                fmt.insert(idx, '_');
                            }
                            quote! { Self::#name(#( #names ),*) => write!(f, #fmt,) }
                        }
                    }
                } else {
                    err = Some(attr.path().span());
                    Default::default()
                }
            })
            .collect();
        if let Some(err) = err {
            return Err(syn::Error::new(err,
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
                                attr.path().span(),
                                "'from' attribute can only be used with variants with one field",
                            ));
                            break
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
                                err = Some((attr.path().span(), "unsupported 'source' type"));
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
            #[allow(unused_variables)]
            fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
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
    let impl_into_any = impl_into_any(input)?.unwrap_or(Default::default());
    let expanded = quote! {
        #impl_display
        #impl_error
        #impl_from
        #impl_into_any
    };
    Ok(TokenStream::from(expanded))
}

#[inline(always)]
fn handle_struct(input: &DeriveInput, s: &syn::DataStruct) -> syn::Result<TokenStream> {
    let mut impl_display = Default::default();
    let name = &input.ident;
    let impl_error =
    if let Some(field) = s.fields.iter().next() {
        if field.ident.is_some() {
            if let Some(attr) = input.attrs
                .iter()
                .find(|attr| attr.path().is_ident("display"))
            {
                let names = s.fields
                    .iter()
                    .map(|f| &f.ident);
                let fmt = attr.parse_args::<LitStr>()?;
                impl_display = quote! {
                    impl core::fmt::Display for #name {
                        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                            let Self { #( #names ),* } = self;
                            write!(f, #fmt)
                        }
                    }
                };
            }
        } else {
            if let Some(attr) = input.attrs
                .iter()
                .find(|attr| attr.path().is_ident("display"))
            {
                let names: Vec<_> = s.fields
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let mut name = String::new();
                        write!(&mut name, "_{i}").ok();
                        syn::Ident::new(&name, Span::call_site())
                    })
                    .collect();
                
                let mut fmt = attr.parse_args::<LitStr>()?.value();
                let mut indices = Vec::new();
                for (i, _) in names.iter().enumerate() {
                    let mut search_for = String::new();
                    write!(&mut search_for, "{{{i}}}").ok();
                    let mut substr = &fmt[..];
                    let len = search_for.len();
                    let mut off = 0;
                    while let Some(idx) = substr.find(&search_for) {
                        substr = &substr[idx + len..];
                        off += idx;
                        indices.push(off + 1);
                        off += len;
                    }
                }
                indices.sort();
                for &idx in indices.iter().rev() {
                    fmt.insert(idx, '_');
                }
                impl_display = quote! {
                    impl core::fmt::Display for #name {
                        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                            let Self(#( #names ),*) = self;
                            write!(f, #fmt)
                        }
                    }
                };
            }
        }
        let mut source = None;
        for (i, field) in s.fields.iter().enumerate() {
            if let Some(attr) = field.attrs
                .iter()
                .find(|attr| attr.path().is_ident("source"))
            {
                if source.is_some() {
                    return Err(syn::Error::new(
                        attr.path().span(), "'source' attribute can only be used for one field"
                    ))
                }
                if let Ok(expr) = attr.parse_args::<syn::Expr>() {
                    source = Some((i, &field.ident, Some(expr)));
                } else {
                    source = Some((i, &field.ident, None));
                }
            }
        }
        if let Some((idx, ident, expr)) = source {
            if let Some(expr) = expr {
                quote! {
                    impl core::error::Error for #name {
                        fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
                            #expr
                        }
                    }
                }
            } else {
                if let Some(ident) = ident {
                    quote! {
                        impl core::error::Error for #name {
                            fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
                                Some(&self.#ident)
                            }
                        }
                    }
                } else {
                    let idx = syn::Index::from(idx);
                    quote! {
                        impl core::error::Error for #name {
                            fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
                                Some(&self.#idx)
                            }
                        }
                    }
                }
            }
        } else {
            quote! { impl core::error::Error for #name {} }
        }
    } else {
        if let Some(attr) = input.attrs
            .iter()
            .find(|attr| attr.path().is_ident("display"))
        {
            let fmt = attr.parse_args::<LitStr>()?;
            impl_display = quote! {
                impl core::fmt::Display for #name {
                    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        write!(f, #fmt)
                    }
                }
            };
        }
        quote! { impl core::error::Error for #name {} }
    };
    let impl_into_any = impl_into_any(input)?.unwrap_or(Default::default());
    let expanded = quote! {
        #impl_display
        #impl_error
        #impl_into_any
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
        Data::Struct(s) => {
            match handle_struct(&input, s) {
                Ok(ts) => ts,
                Err(err) => err.to_compile_error().into(),
            }
        },
        Data::Union(_) => {
            syn::Error
                ::new(input.attrs.first().span(), "unions not supported")
                .to_compile_error()
                .into()
        },
    }
}
