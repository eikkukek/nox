use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, parse_macro_input, spanned::Spanned
};

pub fn handle_struct(
    input: &DeriveInput,
    data: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream>
{
    let by_mut = input.attrs
        .iter()
        .any(|attr| attr.path().is_ident("by_mut"));
    let mut any_default = false;
    let mut fields = vec![];
    fields.extend(data.fields
        .iter()
        .map(|field| {
            let default = field.attrs
                .iter()
                .find(|attr| attr.path().is_ident("default"));
            any_default |= default.is_some();
            (field, default)
        })
    );
    let fns =
        fields
        .iter()
        .filter(|(field, _)|
            !field.attrs.iter().any(|attr| attr.path().is_ident("skip")) &&
            matches!(field.vis, syn::Visibility::Public(_)) &&
            field.ident
            .as_ref()
            .is_some_and(|ident|
                !ident.to_string().starts_with("_")
            )
        ).map(|(field, _)| {
            let docs = field.attrs
                .iter()
                .filter(|attr| attr.path().is_ident("doc"));
            let ident = &field.ident;
            let ty = &field.ty;
            if !by_mut {
                quote! {
                    #(#docs)*
                    #[must_use]
                    #[inline]
                    pub fn #ident(mut self, #ident: #ty) -> Self {
                        self.#ident = #ident;
                        self
                    }
                }
            } else {
                quote! {
                    #(#docs)*
                    #[must_use]
                    #[inline]
                    pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                        self.#ident = #ident;
                        self
                    }
                }
            }
        });
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let getters = quote! {
        impl #impl_generics #name #ty_generics #where_clause {

            #(#fns)*
        }
    };
    let defaults =
        if any_default {
            let mut failed = None;
            let exprs = fields
                .iter()
                .map(|(field, attr)| {
                    let name = &field.ident;
                    if let Some(attr) = attr {
                        match attr.parse_args::<syn::Expr>() {
                            Ok(expr) => quote! {
                                #name: #expr
                            },
                            Err(err) => {
                                failed = Some(err);
                                quote! {}
                            },
                        }
                    } else {
                        quote! { #name: Default::default() }
                    }
                });
            let i = quote! {

                impl #impl_generics Default for #name #ty_generics #where_clause {

                    fn default() -> Self {
                        Self {
                            #(#exprs),*
                        }
                    }
                }
            };
            if let Some(err) = failed {
                return Err(err)
            }
            i
        } else {
            quote! {}
        };
    Ok(quote! {
        #getters
        #defaults
    })
}

pub fn build_structure(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    match &input.data {
        syn::Data::Struct(data) =>
            match handle_struct(&input, data) {
                Ok(s) => s.into(),
                Err(err) => err.to_compile_error().into(),
            },
        _ => syn::Error::new(input.span(), "expected struct")
            .to_compile_error().into(),
    }
}
