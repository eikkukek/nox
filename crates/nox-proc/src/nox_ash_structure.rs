use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Data,
    parse_macro_input,
    spanned::Spanned, 
};

use crate::util::find_attr;

pub fn nox_ash_structure(item: TokenStream) -> TokenStream {
    let input = &parse_macro_input!(item as DeriveInput); let Data::Struct(data) = &input.data else {
        return syn::Error
            ::new(find_attr(input, "Structure").span(), "Expected struct")
            .to_compile_error()
            .into()
    };
    if !input.attrs
        .iter()
        .any(|attr| {
            if attr.path().is_ident("repr") &&
                let Ok(meta) = attr.parse_args::<syn::Ident>() &&
                meta == "C"
            {
                true
            } else {
                false
            }
        })
    {
        return syn::Error
            ::new(find_attr(input, "Structure").span(), "Struct must be repr(C)")
            .to_compile_error()
            .into()
    }
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let default = data.fields
        .iter()
        .map(|field| {
            if let Some(ident) = &field.ident
            {
                if ident == "s_type" {
                    quote! { s_type: Self::STRUCTURE_TYPE }
                } else {
                    quote! { #ident: Default::default() }
                }
            } else {
                quote! {}
            }
        });
    let impl_default = quote! {
        impl #impl_generics Default for #name #ty_generics #where_clause {

            #[inline]
            fn default() -> Self {
                Self {
                    #(#default),*
                }
            }
        }
    };
    let builders = data.fields
        .iter()
        .map(|field| {
            if let Some(ident) = &field.ident &&
                ident != "s_type" &&
                ident != "p_next" &&
                ident != "_marker"
            {
                let ty = &field.ty;
                match ty {
                    syn::Type::Path(p) if p.path.is_ident("Bool32") => {
                        quote! {
                            #[inline]
                            pub fn #ident(mut self, #ident: bool) -> Self {
                                self.#ident = #ident as Bool32;
                                self
                            }
                        }
                    },
                    _ => {
                        quote! {
                            #[inline]
                            pub fn #ident(mut self, #ident: #ty) -> Self {
                                self.#ident = #ident;
                                self
                            }
                        }
                    },
                }
            } else {
                quote! {}
            }
        });
    let impl_builders = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#builders)*
        }
    };
    quote! {
        #impl_default
        #impl_builders

        unsafe impl #impl_generics Send for #name #ty_generics #where_clause {}
        unsafe impl #impl_generics Sync for #name #ty_generics #where_clause {}
    }.into()
}
