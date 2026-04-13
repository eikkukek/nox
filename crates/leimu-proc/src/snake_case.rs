use proc_macro2::TokenStream;

use proc_macro2::{
    Group,
    Ident,
    TokenTree
};

use syn::spanned::Spanned;

fn parse(
    stream: TokenStream,
    in_group: bool,
) -> (TokenStream, bool)
{
    #[derive(PartialEq, Eq)]
    enum Progress {
        None,
        Left,
        Ident,
        Right,
    }
    let span = stream.span();
    let mut out = TokenStream::new();
    let mut progress = Progress::None;
    for token in stream {
        match token {
            TokenTree::Group(group) => {
                let (new_stream, replace) = parse(group.stream(), true);
                if replace {
                    let ident = Ident::new(
                        &new_stream.to_string(),
                        group.span(),
                    );
                    out.extend([ident]);
                } else {
                    let mut new_group = Group::new(group.delimiter(), new_stream);
                    new_group.set_span(group.span());
                    out.extend([new_group]);
                }
            },
            TokenTree::Punct(p) if in_group && p.as_char() == '!' =>
            {
                match progress {
                    Progress::None => progress = Progress::Left,
                    Progress::Ident => progress = Progress::Right,
                    _ => {}
                }
            },
            TokenTree::Ident(i) if progress == Progress::Left =>
            {
                let mut ident = i.to_string();
                let mut indices = vec![];
                let mut prev_char = ' ';
                let mut first_num = None;
                for (idx, ch) in ident.char_indices().skip(1) {
                    if ch.is_uppercase()
                    {
                        if let Some((first_num, add_idx)) = first_num.take()
                            && add_idx
                        {
                            indices.push(first_num);
                        }
                        indices.push(idx);
                    }
                    if ch.is_numeric() && first_num.is_none() {
                        first_num = Some((idx, prev_char.is_lowercase()));
                    }
                    prev_char = ch;
                }
                if let Some((first_num, add_idx)) = first_num.take()
                    && add_idx
                {
                    indices.push(first_num);
                }
                for &idx in indices.iter().rev() {
                    ident.insert(idx, '_');
                }
                ident = ident.to_uppercase();
                let ident = Ident::new(&ident, i.span());
                out.extend([ident]);
                progress = Progress::Ident;
            }
            other => out.extend([other]),
        }
    }
    if matches!(progress, Progress::Left | Progress::Ident) {
        return (syn::Error::new(
            span,
            "mismatched closing delimiter !"
        ).to_compile_error(), false);
    }
    (out, progress != Progress::None)
}

pub fn snake_case(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse(item.into(), false).0.into()
}
