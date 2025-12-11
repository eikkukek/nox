use core::fmt::Write;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    DeriveInput, Data,
    parse_macro_input,
    LitStr, Expr,
    spanned::Spanned, 
};

use crate::input::Input;
use crate::util::find_attr;

pub fn handle_enum(input: &Input, e: &syn::DataEnum) -> syn::Result<Option<proc_macro2::TokenStream>> {
    let name = &input.ident;
    let generics = &input.generics;
    let generic_idents = &input.generic_idents;
    let where_clause = &generics.where_clause;
    let mut display = Vec::new();
    for (i, var) in e.variants.iter().enumerate() {
        if let Some(attr) = var
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("display") )
        {
            display.push((i, attr));
        }
    }
    if !display.is_empty() {
        let mut err = Ok(());
        let fmts = display
            .iter()
            .map(|&(idx, attr)| {
                let var = &e.variants[idx];
                let name = &var.ident;
                match attr.parse_args::<syn::LitStr>()
                {
                    Ok(fmt) => {
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
                    },
                    Err(e) => {
                        err = Err(e);
                        Default::default()
                    },
                }
            });
        let expanded = quote! {
            impl #generics core::fmt::Display for #name #generic_idents #where_clause {
                #[allow(unused_variables)]
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        #( #fmts, )*
                        _ => Ok(()),
                    }
                }
            }
        };
        err?;
        Ok(Some(expanded))
    } else {
        Ok(None)
    }
}

pub fn handle_struct(input: &Input, s: &syn::DataStruct) -> syn::Result<Option<proc_macro2::TokenStream>> {
    let name = &input.ident;
    let generics = &input.generics;
    let generic_idents = &input.generic_idents;
    let where_clause = &generics.where_clause;
    let impl_dislay =
    if let Some(field) = s.fields.iter().next() {
        if field.ident.is_some() {
            if let Some(attr) = input.attrs
                .iter()
                .find(|attr| attr.path().is_ident("display"))
            {
                let names = s.fields
                    .iter()
                    .map(|f| &f.ident);
                let fmt =
                    if let Ok(s) = attr.parse_args::<LitStr>() {
                        quote! { #s }
                    } else {
                        let expr = attr.parse_args::<Expr>()?;
                        quote! { "{}", #expr }
                    };
                Some(quote! {
                    impl #generics core::fmt::Display for #name #generic_idents #where_clause
                    {
                        #[allow(unused_variables)]
                        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                            let Self { #( #names ),* } = self;
                            write!(f, #fmt)
                        }
                    }
                })
            } else {
                None
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
                    write!(&mut search_for, "{{{i}").ok();
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
                Some(quote! {
                    impl #generics #generics core::fmt::Display for #name #generic_idents #where_clause
                    {
                        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                            let Self(#( #names ),*) = self;
                            write!(f, #fmt)
                        }
                    }
                })
            } else {
                None
            }
        }
    } else {
        if let Some(attr) = input.attrs
            .iter()
            .find(|attr| attr.path().is_ident("display"))
        {
            let fmt = attr.parse_args::<LitStr>()?;
            Some(quote! {
                impl #generics core::fmt::Display for #name #generic_idents #where_clause
                {
                    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        write!(f, #fmt)
                    }
                }
            })
        } else {
            None
        }
    };
    Ok(impl_dislay)
}

pub fn display(item: TokenStream) -> TokenStream {
    let input = &parse_macro_input!(item as DeriveInput);
    match &input.data {
        Data::Enum(e) => {
            match handle_enum(&input.into(), e) {
                Ok(ts) => ts
                    .unwrap_or_else(|| syn::Error::new(
                        find_attr(input, "Display").span(),
                        "failed to find 'display' attribute"
                    ).to_compile_error()).into(),
                Err(err) => err.to_compile_error().into(),
            }
        },
        Data::Struct(s) => {
            match handle_struct(&input.into(), s) {
                Ok(ts) => ts
                    .unwrap_or_else(|| syn::Error::new(
                        find_attr(input, "Display").span(),
                        "failed to find 'display' attribute"
                    ).to_compile_error()).into(),
                Err(err) => err.to_compile_error().into(),
            }
        },
        Data::Union(_) => {
            syn::Error
                ::new(find_attr(input, "Display").span(), "unions not supported")
                .to_compile_error()
                .into()
        },
    }
}
