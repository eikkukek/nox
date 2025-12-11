use proc_macro::TokenStream;

use quote::quote;

use syn::{parse_macro_input, spanned::Spanned};

fn handle_fn(f: syn::ItemFn) -> syn::Result<TokenStream> {
    let sig = f.sig;
    let block = f.block;
    let attr = f.attrs;
    let vis = f.vis;
    return Ok(quote! {
        #(#attr)*
        #vis #sig {
            #block
        }
    }.into())
}

fn find_item_attr<'a>(item: &'a syn::Item, ident: &str) -> Option<&'a syn::Attribute> {
    macro_rules! find_attr {
        ($ident:ident, $($path:path),+ $(,)?) => {
            match $ident {
                $($path(i) => { i.attrs.iter().find(|attr| attr.path().is_ident(ident)) }),+
                _ => None,
            }
        };
    }
    find_attr!(item,
        syn::Item::ExternCrate,
        syn::Item::Use,
        syn::Item::Static,
        syn::Item::Const,
        syn::Item::Fn,
        syn::Item::Mod,
        syn::Item::ForeignMod,
        syn::Item::Type,
        syn::Item::Struct,
        syn::Item::Enum,
        syn::Item::Union,
        syn::Item::Trait,
        syn::Item::Impl,
        syn::Item::Macro,
    )
}

pub fn track(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
    /*
    let input = parse_macro_input!(item as syn::Item);
    match input {
        syn::Item::Fn(f) => match handle_fn(f) {
            Ok(ts) => ts,
            Err(err) => err.to_compile_error().into(),
        },
        item => {
            syn::Error::new(find_item_attr(&item, "track").span(), "expected function")
            .to_compile_error().into()
        }
    }
    */
}
