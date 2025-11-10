use proc_macro::{TokenStream};
use syn::{parse_macro_input, Data, DeriveInput, Error};
use quote::{quote, ToTokens};

pub fn nox_vertex_input(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let mut repr_c = false;
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            if let Ok(ident) = attr.parse_args::<syn::Ident>() {
                if ident == "C" {
                    repr_c = true;
                    break
                }
            }
        }
    }
    if !repr_c {
        let err = Error::new_spanned(&input, "Struct must be repr(C)");
        return err.to_compile_error().into()
    }
    let fields = match &input.data {
        Data::Struct(data_struct) => &data_struct.fields,
        _ => {
            return Error::new_spanned(&input, "VertexInput must be a struct")
                .to_compile_error()
                .into()
        }
    };
    let name = &input.ident;
    let inputs: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let i = i as u32;
            let n = &f.ident; 
            let ty = f.ty.to_token_stream();
            let str = ty.to_string();
            let format = match str.as_str() {
                "u32" => quote!(VkFormat::R32_UINT),
                "[u32; 2]" => quote!(VkFormat::R32G32_UINT),
                "[u32; 3]" => quote!(VkFormat::R32G32B32_UINT),
                "[u32; 4]" => quote!(VkFormat::R32G32B32A32_UINT),
                "i32" => quote!(VkFormat::R32_SINT),
                "[i32; 2]" => quote!(VkFormat::R32G32_SINT),
                "[i32; 3]" => quote!(VkFormat::R32G32B32_SINT),
                "[i32; 4]" => quote!(VkFormat::R32G32B32A32_SINT),
                "f32" => quote!(VkFormat::R32_SFLOAT),
                "[f32; 2]" => quote!(VkFormat::R32G32_SFLOAT),
                "[f32; 3]" => quote!(VkFormat::R32G32B32_SFLOAT),
                "[f32; 4]" => quote!(VkFormat::R32G32B32A32_SFLOAT),
                _ if str.ends_with("Vec2") => quote!(VkFormat::R32G32_SFLOAT),
                _ if str.ends_with("Vec3") => quote!(VkFormat::R32G32B32_SFLOAT),
                _ if str.ends_with("Vec4") => quote!(VkFormat::R32G32B32A32_SFLOAT),
                _ => quote!(#ty::VK_FORMAT),
            };
            quote! {
                VertexInputAttribute {
                    location: FIRST_LOCATION + #i,
                    format: #format,
                    offset: core::mem::offset_of!(#name, #n) as u32,
                }
            }
        })
        .collect();
    let expanded = quote! {
        use nox::*;
        unsafe impl VertexInput for #name {

            fn get_attributes<const FIRST_LOCATION: u32>() -> &'static [VertexInputAttribute] {
                &[ #( #inputs ),*
                ]
            }
        }
    };
    TokenStream::from(expanded)
}
