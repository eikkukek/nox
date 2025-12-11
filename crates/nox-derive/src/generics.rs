use syn::{Token, punctuated::Punctuated};

pub enum GenericIdent<'a> {
    Lifetime(&'a syn::Lifetime),
    Type(&'a syn::Ident),
    Const(&'a syn::Ident),
}

impl<'a> quote::ToTokens for GenericIdent<'a> {

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Lifetime(lifetime) => lifetime.to_tokens(tokens),
            Self::Type(ty) => ty.to_tokens(tokens),
            Self::Const(c) => c.to_tokens(tokens),
        }
    }
}

impl<'a> From<&'a syn::GenericParam> for GenericIdent<'a> {

    fn from(value: &'a syn::GenericParam) -> Self {
        match value {
            syn::GenericParam::Lifetime(lifetime) => Self::Lifetime(&lifetime.lifetime),
            syn::GenericParam::Type(ty) => Self::Type(&ty.ident),
            syn::GenericParam::Const(c) => Self::Const(&c.ident),
        }
    }
}

pub struct GenericIdents<'a> {
    pub lt_token: Option<Token![<]>,
    pub idents: Punctuated<GenericIdent<'a>, Token![,]>,
    pub gt_token: Option<Token![>]>,
}

impl<'a> From<&'a syn::Generics> for GenericIdents<'a> {

    fn from(value: &'a syn::Generics) -> Self {
        Self {
            lt_token: value.lt_token,
            idents: value.params.iter().map(GenericIdent::from).collect(),
            gt_token: value.gt_token,
        }
    }
}

impl<'a> quote::ToTokens for GenericIdents<'a> {

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.lt_token.to_tokens(tokens);
        self.idents.to_tokens(tokens);
        self.gt_token.to_tokens(tokens);
    }
}
