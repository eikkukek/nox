use core::fmt::Write;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Data, DeriveInput, Ident, LitStr, Expr, parse_macro_input, spanned::Spanned};
use quote::{quote};

use crate::{
    input::Input,
    display,
};

#[inline(always)]
fn handle_enum(input: &Input, e: &syn::DataEnum) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let generic_idents = &input.generic_idents;
    let where_clause = &generics.where_clause;
    let impl_display = display::handle_enum(input, e)?; 
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
                        if let Ok(expr) = attr.parse_args::<Expr>() {
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
                            let field_ty = match &field.ty {
                                syn::Type::Path(path) => {
                                    Some(path)
                                },
                                _ => None,
                            };
                            if let Some(field_ty) = field_ty {
                                if let Some(ident) = &field.ident {
                                    from.push(quote! {
                                        impl #generics From<#field_ty> for #name #generic_idents #where_clause {

                                            fn from(value: #field_ty) -> Self {
                                                Self::#var_name { #ident: value, }
                                            }
                                        }
                                    });
                                } else {
                                    from.push(quote! {
                                        impl #generics From<#field_ty> for #name #generic_idents #where_clause {

                                            fn from(value: #field_ty) -> Self {
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
        impl #generics core::error::Error for #name #generic_idents #where_clause {
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
    let expanded = quote! {
        #impl_display
        #impl_error
        #impl_from
    };
    Ok(TokenStream::from(expanded))
}

#[inline(always)]
fn handle_struct(input: &Input, s: &syn::DataStruct) -> syn::Result<TokenStream> {
    let mut impl_display = Default::default();
    let name = &input.ident;
    let generics = &input.generics;
    let generic_idents = &input.generic_idents;
    let where_clause = &generics.where_clause;
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
                let fmt =
                    if let Ok(s) = attr.parse_args::<LitStr>() {
                        quote! { #s }
                    } else {
                        let expr = attr.parse_args::<Expr>()?;
                        quote! { "{}", #expr }
                    };
                impl_display = quote! {
                    impl #generics core::fmt::Display for #name #generic_idents #where_clause
                    {
                        #[allow(unused_variables, unused_assignments)]
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
                impl_display = quote! {
                    impl #generics core::fmt::Display for #name #generic_idents #where_clause
                    {
                        #[allow(unused_variables, unused_assignments)]
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
                if let Ok(expr) = attr.parse_args::<Expr>() {
                    source = Some((i, &field.ident, Some(expr)));
                } else {
                    source = Some((i, &field.ident, None));
                }
            }
        }
        if let Some((idx, ident, expr)) = source {
            if let Some(expr) = expr {
                quote! {
                    impl #generics core::error::Error for #name #generic_idents #where_clause
                    {
                        #[allow(unused_variables)]
                        fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
                            #expr
                        }
                    }
                }
            } else {
                if let Some(ident) = ident {
                    quote! {
                        impl #generics core::error::Error for #name #generic_idents #where_clause
                        {
                            #[allow(unused_variables)]
                            fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
                                Some(&self.#ident)
                            }
                        }
                    }
                } else {
                    let idx = syn::Index::from(idx);
                    quote! {
                        impl #generics core::error::Error for #name #generic_idents #where_clause
                        {
                            #[allow(unused_variables)]
                            fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
                                Some(&self.#idx)
                            }
                        }
                    }
                }
            }
        } else {
            quote! { impl #generics core::error::Error for #name #generic_idents #where_clause {} }
        }
    } else {
        if let Some(attr) = input.attrs
            .iter()
            .find(|attr| attr.path().is_ident("display"))
        {
            let fmt = attr.parse_args::<LitStr>()?;
            impl_display = quote! {
                impl #generics core::fmt::Display for #name #generic_idents #where_clause
                {
                    #[allow(unused_variables, unused_assignments)]
                    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        write!(f, #fmt)
                    }
                }
            };
        }
        quote! { impl #generics core::error::Error for #name #generic_idents #where_clause {} }
    };
    let expanded = quote! {
        #impl_display
        #impl_error
    };
    Ok(TokenStream::from(expanded))
}

pub fn error(item: TokenStream) -> TokenStream {
    let input = &parse_macro_input!(item as DeriveInput);
    match &input.data {
        Data::Enum(e) => {
            match handle_enum(&input.into(), e) {
                Ok(ts) => ts,
                Err(err) => err.to_compile_error().into(),
            }
        },
        Data::Struct(s) => {
            match handle_struct(&input.into(), s) {
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
