use std::collections::HashMap;

use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::{
    Token,
    token,
    Attribute,
    Visibility,
    Ident,
    Expr,
    Lit,
    Type,
    ext::IdentExt,
    punctuated::Punctuated,
    parse::{Parse, ParseStream},
    braced,
    parse_macro_input,
    parenthesized,
};

struct Variant {
    attr: Vec<Attribute>,
    name: Ident,
    eq_token: Token![=],
    value_token: Lit,
}

impl ToTokens for Variant {

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for attr in self.attr.iter()
            .filter(|attr| !attr.path().is_ident("group"))
        {
            attr.to_tokens(tokens);
        }
        self.name.to_tokens(tokens);
        self.eq_token.to_tokens(tokens);
        self.value_token.to_tokens(tokens);
    }
}

impl Parse for Variant {

    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Variant {
            attr: input.call(Attribute::parse_outer)?,
            name: input.parse()?,
            eq_token: input.parse()?,
            value_token: input.parse()?
        })
    }
}

struct GroupBound {
    ident: Ident,
    _colon_token: Token![=],
    ty: Type,
}

impl Parse for GroupBound {

    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            _colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

struct WhereClause {
    _where_token: Token![where],
    bounds: Vec<GroupBound>,
}

impl Parse for WhereClause {

    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _where_token = input.parse()?;
        let mut bounds: Vec<GroupBound> = vec![];
        while input.peek(Ident::peek_any)  {
            bounds.push(input.parse()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self {
            _where_token,
            bounds,
        })
    }
}

struct Enum {
    attr: Vec<Attribute>,
    vis: Visibility,
    _enum_token: Token![enum],
    ident: Ident,
    where_clause: Option<WhereClause>,
    _brace_token: token::Brace,
    variants: Punctuated<Variant, Token![,]>,
}

impl Parse for Enum {

    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Enum {
            attr: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            _enum_token: input.parse()?,
            ident: input.parse()?,
            where_clause:
                if input.peek(Token![where]) {
                    Some(input.parse()?)
                } else {
                    None
                },
            _brace_token: braced!(content in input),
            variants: content.parse_terminated(Variant::parse, Token![,])?,
        })
    }
}

struct Group {
    ident: Ident,
    expr: Option<Expr>,
}

impl Parse for Group {

    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Group {
            ident: input.parse()?,
            expr:
                if input.peek(token::Paren) {
                    parenthesized!(content in input);
                    Some(content.parse()?)
                } else {
                    None
                },
        })
    }
}

pub fn vk_to_rust_enum(item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as Enum);
    if input.variants.len() < 2 {
        return syn::Error::new(
            input.ident.span(),
            "input has to have at least two variants"
        ).to_compile_error().into()
    }
    let mut iter = input.variants.iter();
    let first = iter.next().unwrap();
    let mut common_ident = first.name.to_string();
    for variant in iter {
        let name = variant.name.to_string();
        let mut char_iter = name.chars();
        let mut byte_idx = 0;
        for (idx, a) in common_ident.char_indices() {
            let Some(b) = char_iter.next() else {
                break;
            };
            if a != b {
                break
            }
            byte_idx = idx;
        }
        common_ident = common_ident[0..byte_idx+1].to_string();
    }
    let mut groups: HashMap<Ident, (Vec<_>, Option<Type>)>
        = HashMap::default();
    if let Some(clause) = input.where_clause {
        for bound in clause.bounds {
            groups.insert(bound.ident, (vec![], Some(bound.ty)));
        }
    }
    for variant in &mut input.variants {
        let mut name = variant.name
            .to_string()
            .replace(&common_ident, "")
            .to_lowercase();
        let mut to_upper_case = vec![];
        for (idx, ch) in name.char_indices()
        {
            if ch == '_' {
                to_upper_case.push(idx);
            }
        }
        for &idx in to_upper_case.iter().rev() {
            name.remove(idx);
            let ch = name.remove(idx).to_ascii_uppercase();
            name.insert(idx, ch);
        }
        let ch = name.remove(0).to_ascii_uppercase();
        name.insert(0, ch);
        variant.name = Ident::new(&name, variant.name.span());
        if let Some(attr) = variant.attr
            .iter()
            .find(|attr| {
                attr.path().is_ident("group")
            }) &&
            let Ok(inner) = attr.parse_args_with(
                Punctuated::<Group, Token![,]>::parse_terminated
            )
        {
            for group in inner {
                groups.entry(group.ident)
                .and_modify(|variants| {
                    variants.0.push((&*variant, group.expr.clone()));
                }).or_insert_with(|| (vec![(&*variant, group.expr)], None));
            }
        }
    }
    let mut group_fns = vec![];
    for (group, (variants, ty)) in groups {
        if let Some(ty) = ty {
            let matches = variants
                .iter()
                .filter_map(|(var, expr)| {
                    let var = &var.name;
                    expr.as_ref().map(|expr| {
                        quote! {
                            if matches!(self, Self::#var) {
                                return Some(#expr)
                            }
                        }
                    })
                });
            let group_fn = syn::Ident::new(&format!(
                "group_{}_value",
                group,
            ), group.span());
            group_fns.push(quote! {
                fn #group_fn(self) -> Option<#ty> {
                    #(#matches)*
                    None
                }
            });
        }
        let idents = variants
            .iter()
            .map(|(var, _)| {
                let var = &var.name;
                quote! { Self::#var }
            });
        let is_in_fn_name = syn::Ident::new(&format!(
            "is_in_group_{}",
            group,
        ), group.span());
        group_fns.push(quote! {
            const fn #is_in_fn_name(self) -> bool {
                matches!(self, #(#idents)|*)
            }
        });
    }
    let attr = &input.attr;
    let vis = &input.vis;
    let name = &input.ident;
    let variants = &input.variants;
    let def = quote! {
        #(#attr)*
        #vis enum #name {
            #variants
        }
    };
    let group_impl =
        if group_fns.is_empty() {
            quote! {}
        } else {
            quote! {
                impl #name {
                    #(#group_fns)*
                }
            }
        };
    quote! {
        #def
        #group_impl
    }.into()
}
