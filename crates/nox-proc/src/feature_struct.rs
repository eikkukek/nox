use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Data,
    parse_macro_input,
    spanned::Spanned,
};

use crate::util::find_attr;

pub fn feature_struct(item: TokenStream) -> TokenStream {
    let input = &parse_macro_input!(item as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let Data::Struct(data) = &input.data else {
        return syn::Error
            ::new(find_attr(input, "Structure").span(), "Expected struct")
            .to_compile_error()
            .into()
    };
    let builders = data.fields
        .iter()
        .map(|field| {
            let syn::Type::Path(ty) = &field.ty else {
                return quote! {}
            };
            if !ty.path.is_ident("bool") {
                return quote! {}
            }
            let Some(ident) = &field.ident else {
                return quote! {}
            };
            let fn_name = syn::Ident::new(&format!(
                "with_{}",
                ident,
            ), field.span());
            quote! {
                pub fn #fn_name(mut self, value: bool) -> Self {
                    self.#ident = value;
                    self
                }
            }
        });
    let name = &input.ident;
    let defaults = data.fields
        .iter()
        .map(|field| {
            let Some(ident) = &field.ident else {
                return quote! {}
            };
            let fallback = quote! { #ident: Default::default() };
            let syn::Type::Path(ty) = &field.ty else {
                return fallback
            };
            if !ty.path.is_ident("bool") {
                return fallback
            }
            if !field.attrs
                .iter()
                .any(|attr| attr.path().is_ident("on"))
            {
                return fallback
            }
            quote! { #ident: true }
        });
    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#builders)*
        }
        impl #impl_generics Default for #name #ty_generics #where_clause {

            #[inline(always)]
            fn default() -> Self {
                Self {
                    #(#defaults),*
                }
            }
        }
    }.into()
}
