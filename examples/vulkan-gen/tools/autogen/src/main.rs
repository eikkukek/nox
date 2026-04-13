use std::fs::{File, DirBuilder};
use std::io::{BufReader, Write};
use std::collections::HashSet;
use std::process;
use core::{
    hash::{self, Hash},
    num::NonZeroU8,
};
use indexmap::{IndexMap, IndexSet};
use std::time::Duration;

use reqwest::blocking::Client;

use quote::quote;
use proc_macro2::Span;

use syn::Ident;
use xml::reader::{EventReader, XmlEvent};

#[derive(Default)]
struct Bitmask {
    requires: Option<String>,
    parsing_type: bool,
    ty: String,
    parsing_name: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum FlagType {
    U32,
    U64,
}

fn url_exists(url: &str) -> bool {
    /*
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    match client.head(url).send() {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }*/
    true
}

fn upper_snake_case(string: &str) -> String
{
    let mut res = string.to_string();
    for (idx, ch) in string.char_indices().rev() {
        if idx == 0 { break }
        if ch.is_uppercase() {
            res.insert(idx, '_');
        }
    }
    res.to_uppercase()
}

fn lower_snake_case(string: &str) -> String {
    let mut res = string.to_string();
    let mut iter = string.char_indices().rev();
    let mut next_ch = iter.next().unwrap().1;
    for (idx, ch) in iter {
        if !ch.is_uppercase() &&
            ch != '_' &&
            next_ch.is_uppercase()
        {
            let idx = idx + 1;
            res.remove(idx);
            res.insert(idx, next_ch.to_lowercase().next().unwrap());
            res.insert(idx, '_');
        }
        next_ch = ch;
    }
    res.to_lowercase()
}

struct Bit {
    name: String,
    bit: u64,
    doc: Option<String>,
    alias: Option<String>,
    deprecated: Option<String>,
}

impl Hash for Bit {

    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Bit {

    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Bit {}

struct BitmaskBits {
    bits: IndexSet<Bit>,
    flag_type: FlagType,
}

struct Variant {
    name: String,
    value: syn::LitInt,
    doc: Option<String>,
    alias: Option<String>,
    deprecated: Option<String>
}

impl PartialEq for Variant {

    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Variant {}

impl Hash for Variant {

    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

struct EnumVariants {
    variants: IndexSet<Variant>,
}

impl EnumVariants {

    fn add(
        &mut self,
        name: String,
        value: syn::LitInt,
        doc: Option<String>
    ) {
        self.variants.insert(Variant {
            name, value, doc, alias: None,
            deprecated: None,
        });
    }

    fn add_ext(
        &mut self,
        name: String,
        offset: u32,
        extnumber: u32,
        dir: Option<&String>,
        doc: Option<String>,
    ) {
        let mut value = format!("1{:06}{:03}", extnumber - 1, offset);
        if let Some(dir) = dir {
            value.insert_str(0, dir);
        }
        self.variants.insert(Variant {
            name,
            value: syn::LitInt::new(
                &value,
                Span::call_site()
            ),
            doc,
            alias: None,
            deprecated: None,
        });
    }
    
    fn add_alias(
        &mut self,
        name: String,
        alias: String,
        deprecated: Option<String>,
    ) {
        self.variants.insert(Variant {
            name,
            value: syn::LitInt::new("0", Span::call_site()),
            doc: None,
            alias: Some(alias),
            deprecated,
        });
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum StructParsing {
    None,
    #[default]
    Const,
    Type,
    Pointer,
    ArrayLen,
    Name,
    Comment,
}

#[derive(Default, Clone)]
struct StructMember {
    value: Option<String>,
    parsing: StructParsing,
    ty: String,
    is_const_ptr: bool,
    ptr_type: PtrType,
    array_len: Option<String>,
    name: String,
    comment: Option<String>,
    deprecated: Option<String>,
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
enum packed_u9_u9_u6_u4_u4 {
    A(String),
    B(String),
    C(String),
    D(String),
    E(String),
}

#[derive(Default, Clone)]
struct Struct {
    name: String,
    extends_unparsed: Option<String>,
    extends: Vec<Ident>,
    members: Vec<StructMember>,
    packed_u24_u8: Option<String>,
    packed_u9_u9_u6_u4_u4: Option<packed_u9_u9_u6_u4_u4>,
    parsing_member: Option<StructMember>,
    needs_lifetime: bool,
    is_union: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HandleType {
    NonDispatchable,
    Dispatchable,
}

#[derive(Clone)]
struct Handle {
    type_enum: String,
    parsing_ty: bool,
    ty: HandleType,
    parsing_name: bool,
}

#[derive(Default, Clone)]
enum FuncParsing {
    #[default]
    Const,
    Ty,
    Pointer,
    Name,
    Array,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum PtrType {
    #[default]
    None,
    ConstPtr,
    MutPtr,
    ConstPtrConstPtr,
    MutPtrMutPtr,
}

impl quote::ToTokens for PtrType {

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match *self {
            Self::None => {},
            Self::ConstPtr => quote! { *const }.to_tokens(tokens),
            Self::MutPtr => quote! { *mut }.to_tokens(tokens),
            Self::ConstPtrConstPtr => quote! { *const *const }.to_tokens(tokens),
            Self::MutPtrMutPtr => quote! { *mut *mut }.to_tokens(tokens),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FuncParamPtrType {
    PtrType(PtrType),
    Array(usize),
}

impl Default for FuncParamPtrType {

    fn default() -> Self {
        Self::PtrType(Default::default())
    }
}

#[derive(Default, Clone)]
enum FuncpointerParsing {
    #[default]
    None,
    ImplicitParams,
    Proto(FuncParsing),
    Param {
        parsing: FuncParsing,
        is_const: bool,
        ty: String,
        ptr: FuncParamPtrType,
        name: String,
    },
}

#[derive(Default, Clone)]
struct Funcparam {
    ty: String,
    ptr: FuncParamPtrType,
    name: String,
}

struct Func {
    ret_ty: String,
    ret_ptr: PtrType,
    params: Vec<Funcparam>,
}

#[derive(Default, Clone)]
struct Funcpointer {
    parsing: FuncpointerParsing,
    ret_is_const: bool,
    ret_ty: String,
    ret_ptr: PtrType,
    params: Vec<Funcparam>,
}

enum Type {
    Handle(Handle),
    Bitmask(Bitmask),
    Struct(Struct),
    Funcpointer(Funcpointer),
}

enum Enums {
    Constants,
    Enum(String),
}

enum CommandBody {
    Alias(String),
    Func(Func)
}

struct Command {
    body: CommandBody,
    version: Option<String>,
    extension: Option<String>,
}

#[derive(Clone)]
struct Feature {
    commands: Vec<String>,
}

#[derive(Clone)]
enum ExtConstant {
    U32(u32),
    String(String),
    Alias(String),
}

#[derive(Clone)]
struct Extension {
    author: String,
    new_constants: Vec<(String, ExtConstant)>,
    commands: Vec<String>,
    number: u32,
}

struct ExtParsing {
    name: String,
    ext: Extension,
    parsing_requires: Option<(u32, bool)>,
}

enum ReaderElement {
    Inactive,
    Type(Type),
    Enums(Enums),
    Feature {
        version: String,
        feature: Feature,
        parsing_requires: Option<u32>,
        is_sc: bool,
        parsing_remove: bool,
    },
    Extensions {
        parsing_extension: Option<ExtParsing>,
    },
}

struct Reader {
    depth: u32,
    element: ReaderElement,
    element_name: Option<String>,
}

#[derive(Default, Clone, Copy)]
enum DepthType {
    #[default]
    Unknown,
    Types,
    Enums,
    Commands,
    Feature,
    Extensions,
}

#[derive(Default, Clone, Copy)]
struct Depth {
    depth: u32,
    vulkan_sc: Option<u32>,
    ty: DepthType,
}

fn main() -> std::io::Result<()> {
    let xml = BufReader::new(File::open("vk.xml")?);
    let parser = EventReader::new(xml);
    let mut reader = Reader {
        depth: 0,
        element: ReaderElement::Inactive,
        element_name: None,
    };
    let mut depth = Depth::default();
    let mut unique_categories = HashSet::new();
    let mut handles = File::create("../../src/vk/handles.rs")?;
    write!(handles, "use super::*; use crate::define_non_dispatchable_handle; use crate::define_handle;")?;
    let mut enums = File::create("../../src/vk/enums.rs")?;
    write!(enums, "use super::*; use crate::bitflags;")?;
    let mut structs = File::create("../../src/vk/structs.rs")?;
    write!(structs, "use super::*; use vk_video::*;")?;
    let mut unions = File::create("../../src/vk/unions.rs")?;
    write!(unions, "use super::*;")?;
    let mut type_defs = File::create("../../src/vk/type_defs.rs")?;
    write!(type_defs, "use super::*;")?;
    let mut type_aliases = IndexMap::<String, String>::new();
    let mut bitmask_bits = IndexMap::<String, BitmaskBits>::new();
    let mut enum_variants = IndexMap::<String, EnumVariants>::new();
    let mut struct_extends = IndexSet::<String>::new();
    let mut struct_defs = vec![];
    let mut funcpointers = vec![];
    let mut commands = IndexMap::<String, Command>::new();
    let mut features = IndexMap::<String, Feature>::new();
    let mut extensions = IndexMap::<String, Extension>::new();
    let mut sc_exclude_list = HashSet::new();
    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                let find_attribute = |name: &str| -> Option<&xml::attribute::OwnedAttribute> {
                    attributes
                        .iter()
                        .find(|attr|
                            attr.name.local_name == name
                        )
                };
                let current_depth = reader.depth;
                reader.depth += 1;
                if let Some(api) = find_attribute("api") &&
                    api.value == "vulkansc"
                {
                    depth.vulkan_sc = Some(current_depth);
                }
                if depth.vulkan_sc.is_some() &&
                    (!matches!(depth.ty, DepthType::Feature | DepthType::Extensions)) &&
                    name.local_name != "feature"
                {
                    continue
                }
                match depth.ty {
                    DepthType::Unknown => {
                        if name.local_name == "types" {
                            depth.ty = DepthType::Types;
                            depth.depth = current_depth;
                        } else if name.local_name == "enums" {
                            depth.ty = DepthType::Enums;
                            depth.depth = current_depth;
                            let ty = find_attribute("type").unwrap();
                            if ty.value == "constants" {
                                reader.element = ReaderElement::Enums(Enums::Constants);
                            } else if ty.value == "enum" || ty.value == "bitmask" {
                                let name = find_attribute("name").unwrap();
                                reader.element = ReaderElement::Enums(Enums::Enum(name.value.clone()));
                            }
                        } else if name.local_name == "commands" {
                            depth.ty = DepthType::Commands;
                            depth.depth = current_depth;
                        } else if name.local_name == "feature" {
                            depth.ty = DepthType::Feature;
                            depth.depth = current_depth;
                            let version = find_attribute("number")
                                .unwrap().value.clone();
                            reader.element = ReaderElement::Feature {
                                version,
                                feature: Feature { commands: vec![] },
                                parsing_requires: None,
                                parsing_remove: false,
                                is_sc: depth.vulkan_sc.is_some(),
                            }
                        } else if name.local_name == "extensions" {
                            depth.ty = DepthType::Extensions;
                            depth.depth = current_depth;
                            reader.element = ReaderElement::Extensions {
                                parsing_extension: None,
                            };
                        }
                    },
                    DepthType::Types => {
                        if current_depth == depth.depth + 1 &&
                            name.local_name == "type"
                        {
                            let Some(category) = find_attribute("category")
                            else {
                                continue
                            };
                            unique_categories.insert(category.value.clone());
                            let name = find_attribute("name");
                            if let Some(alias) = find_attribute("alias")
                            {
                                type_aliases.insert(
                                    name
                                        .map(|name|
                                            name.value.clone()
                                    ).unwrap(),
                                    alias.value.clone()
                                );
                                continue
                            }
                            if category.value == "bitmask" {
                                let bitmask = Bitmask {
                                    requires : find_attribute("requires")
                                        .or_else(|| find_attribute("bitvalues"))
                                        .map(|attr| attr.value.clone()),
                                    ..Default::default()
                                };
                                reader.element = ReaderElement::Type(Type::Bitmask(bitmask));
                                reader.element_name = name.map(|name| name.value.clone());
                            } else if category.value == "enum" {
                                enum_variants.insert(name.as_ref().unwrap().value.clone(), EnumVariants {
                                    variants: IndexSet::new(),
                                });
                            } else if category.value == "struct" || category.value == "union" {
                                let s = Struct {
                                    extends_unparsed: find_attribute("structextends")
                                        .map(|attr| attr.value.clone()),
                                    is_union: category.value == "union",
                                    ..Default::default()
                                };
                                reader.element = ReaderElement::Type(Type::Struct(s));
                                reader.element_name = Some(name.map(|name| name.value.clone()).unwrap());
                            } else if category.value == "handle" {
                                let handle = Handle {
                                    type_enum: find_attribute("objtypeenum")
                                        .unwrap().value.clone(),
                                    ty: HandleType::Dispatchable,
                                    parsing_ty: false,
                                    parsing_name: false,
                                };
                                reader.element = ReaderElement::Type(Type::Handle(handle));
                                reader.element_name = None;
                            } else if category.value == "funcpointer" ||
                                category.value == "command"
                            {
                                let funcpointer = Funcpointer::default();
                                reader.element = ReaderElement::Type(Type::Funcpointer(funcpointer));
                                reader.element_name = None;
                            }
                        } else {
                            let ReaderElement::Type(ty) = &mut reader.element else {
                                continue
                            };
                            match ty {
                                Type::Bitmask(bitmask) => {
                                    if name.local_name == "type" {
                                        bitmask.parsing_type = true;
                                    } else if name.local_name == "name" {
                                        bitmask.parsing_name = true;
                                    } 
                                },
                                Type::Struct(structure) => {
                                    if let Some(parsing_member) = &mut structure.parsing_member {
                                        if name.local_name == "type" {
                                            parsing_member.parsing = StructParsing::Type;
                                        } else if name.local_name == "name" {
                                            parsing_member.parsing = StructParsing::Name;
                                        } else if name.local_name == "enum" {
                                            assert!(parsing_member.parsing == StructParsing::ArrayLen);
                                        } else if name.local_name == "comment" {
                                            parsing_member.parsing = StructParsing::Comment;
                                        }
                                    } else if name.local_name == "member" {
                                        let member = StructMember {
                                            value: find_attribute("values")
                                                .map(|attr| attr.value.clone()),
                                            deprecated: find_attribute("deprecated")
                                                .map(|attr| attr.value.clone()),
                                            ..Default::default()
                                        };
                                        structure.parsing_member = Some(member);
                                    }
                                },
                                Type::Handle(handle) => {
                                    if name.local_name == "type" {
                                        handle.parsing_ty = true;
                                    } else if name.local_name == "name" {
                                        handle.parsing_name = true;
                                    }
                                },
                                Type::Funcpointer(funcpointer) => {
                                    if let FuncpointerParsing::Proto (parsing) =
                                    &mut funcpointer.parsing {
                                        if name.local_name == "type" {
                                            *parsing = FuncParsing::Ty;
                                        } else if name.local_name == "name" {
                                            *parsing = FuncParsing::Name;
                                        }
                                    } else if let FuncpointerParsing::Param {
                                        parsing, ..
                                    } = &mut funcpointer.parsing {
                                        if name.local_name == "type" {
                                            *parsing = FuncParsing::Ty;
                                        } else if name.local_name == "name" {
                                            *parsing = FuncParsing::Name;
                                        }
                                    } else if name.local_name == "proto" {
                                        funcpointer.parsing = FuncpointerParsing::Proto(
                                            FuncParsing::Const
                                        );
                                    } else if name.local_name == "param" {
                                        funcpointer.parsing = FuncpointerParsing::Param {
                                            parsing: FuncParsing::Const,
                                            is_const: false,
                                            ptr: Default::default(),
                                            ty: Default::default(),
                                            name: Default::default(),
                                        };
                                    } else if name.local_name == "implicitexternsyncparams" {
                                        funcpointer.parsing = FuncpointerParsing::ImplicitParams;
                                    }
                                },
                            };
                        }
                    },
                    DepthType::Enums => {
                        let ReaderElement::Enums(e) = &reader.element else {
                            unreachable!()
                        };
                        match e {
                            Enums::Constants => {
                                let (mut ty, mut value, mut name) = Default::default();
                                for attr in &attributes {
                                    if attr.name.local_name == "type" {
                                        ty = Some(attr.value.as_str());
                                    } else if attr.name.local_name == "value" {
                                        value = Some(&attr.value);
                                    } else if attr.name.local_name == "name" {
                                        name = Some(&attr.value);
                                    }
                                }
                                let (ty, value, name) = (ty.unwrap(), value.unwrap(), name.unwrap());
                                let (ty, value) = match ty {
                                    "uint32_t" | "uint64_t" => {
                                        let mut value = value.to_string();
                                        let mut not = None;
                                        if let Some(a) = value.find("~") {
                                            let number = value.split_at(a + 1).1;
                                            let number = number.split_at(1).0;
                                            value = number.to_string();
                                            not = Some(quote! {!});
                                        }
                                        let number = syn::LitInt::new(&value, Span::call_site());
                                        (
                                            syn::Ident::new(
                                                if ty == "uint32_t" { "u32" } else {
                                                    "u64"
                                                },
                                                Span::call_site()
                                            ),
                                            quote! {
                                                #not #number
                                            }
                                        )
                                    },
                                    "float" => {
                                        let number = syn::LitFloat::new(
                                            value
                                                .trim_end_matches("f")
                                                .trim_end_matches("F"),
                                            Span::call_site()
                                        );
                                        (
                                            syn::Ident::new("f32", Span::call_site()),
                                            quote! {#number},
                                        )
                                    },
                                    _ => unreachable!(),
                                };
                                let name = Ident::new(name.trim_start_matches("VK_"), Span::call_site());
                                let def = quote! {
                                    pub const #name: #ty = #value;
                                };
                                write!(enums, "{}", def)?;
                            },
                            Enums::Enum(n) => {
                                if name.local_name != "enum" {
                                    continue
                                }
                                let name = n;
                                let variant_name = find_attribute("name")
                                    .unwrap().value.clone();
                                let doc = find_attribute("comment")
                                    .map(|attr| attr.value.clone());
                                if let Some(alias) = find_attribute("alias")
                                {
                                    if name.find("FlagBits").is_some() {
                                        let bits = &mut bitmask_bits.get_mut(name).unwrap();
                                        bits.bits.insert(Bit {
                                            name: variant_name,
                                            bit: 0,
                                            doc,
                                            alias: Some(alias.value.clone()),
                                            deprecated: find_attribute("deprecated")
                                                .map(|attr| attr.value.clone()),
                                        });
                                    } else {
                                        let variants = &mut enum_variants.get_mut(name).unwrap();
                                        variants.add_alias(
                                            variant_name,
                                            alias.value.clone(),
                                            find_attribute("deprecated")
                                                .map(|attr| attr.value.clone())
                                        );
                                    }
                                    continue
                                }
                                if name.find("FlagBits").is_some()
                                {
                                    let bits = &mut bitmask_bits.get_mut(name).unwrap();
                                    if let Some(bitpos) = find_attribute("bitpos")
                                    {
                                        let bitpos: u32 = str::parse(&bitpos.value).unwrap();
                                        bits.bits.insert(Bit {
                                            name: variant_name,
                                            bit: 1 << bitpos,
                                            doc,
                                            alias: None,
                                            deprecated: None,
                                        });
                                    } else {
                                        let value = find_attribute("value")
                                            .unwrap().value
                                            .clone();
                                        let value =
                                            if value.starts_with("0x") {
                                                u64::from_str_radix(value.trim_start_matches("0x"), 16).unwrap()
                                            } else {
                                                str::parse(&value).unwrap()
                                            };
                                        bits.bits.insert(Bit {
                                            name: variant_name,
                                            bit: value,
                                            doc,
                                            alias: None,
                                            deprecated: None,
                                        });
                                    }
                                } else {
                                    let variants = &mut enum_variants.get_mut(name).unwrap();
                                    let value = find_attribute("value")
                                        .unwrap().value.clone();
                                    variants.add(variant_name, syn::LitInt::new(&value, Span::call_site()), doc);
                                }
                            },
                        }
                    },
                    DepthType::Commands => {
                        if name.local_name == "command" {
                            if let Some(alias) = find_attribute("alias")
                            {
                                let name = find_attribute("name")
                                    .unwrap().value.clone();
                                let alias = alias.value.clone();
                                let pfn_name = format!("PFN_{name}");
                                let pfn_alias = format!("PFN_{alias}");
                                commands.insert(name, Command {
                                    body: CommandBody::Alias(alias),
                                    version: None,
                                    extension: None,
                                });
                                type_aliases.insert(
                                    pfn_name,
                                    pfn_alias,
                                );
                                continue
                            }
                            let funcpointer = Funcpointer::default();
                            reader.element = ReaderElement::Type(Type::Funcpointer(funcpointer));
                            reader.element_name = None;
                        } else {
                            let ReaderElement::Type(ty) = &mut reader.element else {
                                continue
                            };
                            if let Type::Funcpointer(funcpointer) = ty
                            {
                                if let FuncpointerParsing::Proto(parsing)
                                = &mut funcpointer.parsing {
                                    if name.local_name == "type" {
                                        *parsing = FuncParsing::Ty;
                                    } else if name.local_name == "name" {
                                        *parsing = FuncParsing::Name;
                                    }
                                } else if let FuncpointerParsing::Param {
                                    parsing, ..
                                } = &mut funcpointer.parsing {
                                    if name.local_name == "type" {
                                        *parsing = FuncParsing::Ty;
                                    } else if name.local_name == "name" {
                                        *parsing = FuncParsing::Name;
                                    }
                                } else if let FuncpointerParsing::ImplicitParams = funcpointer.parsing {
                                } else if name.local_name == "proto" {
                                    funcpointer.parsing = FuncpointerParsing::Proto(
                                        FuncParsing::Const
                                    );
                                } else if name.local_name == "param" {
                                    funcpointer.parsing = FuncpointerParsing::Param {
                                        parsing: FuncParsing::Const,
                                        is_const: false,
                                        ptr: Default::default(),
                                        ty: Default::default(),
                                        name: Default::default(),
                                    };
                                } else if name.local_name == "implicitexternsyncparams" {
                                    funcpointer.parsing = FuncpointerParsing::ImplicitParams;
                                }
                            }
                        }
                    },
                    DepthType::Feature => {
                        let ReaderElement::Feature {
                            version, feature, parsing_requires,
                            parsing_remove,
                            ..
                        } = &mut reader.element else { unreachable!() };
                        if depth.vulkan_sc.is_none() {
                            if name.local_name == "require" {
                                *parsing_requires = Some(reader.depth);
                            } else if parsing_requires.is_some() &&
                                name.local_name == "command"
                            {
                                let name = find_attribute("name")
                                    .unwrap().value.clone();
                                let cmd = commands.get_mut(&name).unwrap();
                                cmd.version = Some(version.clone());
                                feature.commands.push(name);
                            } else if name.local_name == "enum" &&
                                let Some(extends) = find_attribute("extends")
                                    .map(|extends| &extends.value)
                            {
                                let name = find_attribute("name")
                                    .unwrap().value.clone();
                                if let Some(offset) = find_attribute("offset")
                                {
                                    let offset = str::parse(&offset.value).unwrap();
                                    let extnumber = find_attribute("extnumber").unwrap();
                                    let extnumber = str::parse(&extnumber.value).unwrap();
                                    let doc = find_attribute("comment")
                                        .map(|attr| attr.value.clone());
                                    let variants = enum_variants
                                        .get_mut(extends)
                                        .unwrap();
                                    let dir = find_attribute("dir")
                                        .map(|attr| &attr.value);
                                    variants.add_ext(name, offset, extnumber, dir, doc);
                                } else if let Some(bitpos) = find_attribute("bitpos")
                                {
                                    let bitpos: u32 = str::parse(&bitpos.value).unwrap();
                                    let bits = bitmask_bits
                                        .get_mut(extends)
                                        .unwrap();
                                    let doc = find_attribute("comment")
                                        .map(|attr| attr.value.clone());
                                    bits.bits.insert(Bit {
                                        name,
                                        bit: 1 << bitpos,
                                        doc,
                                        alias: None,
                                        deprecated: None,
                                    });
                                } else if let Some(value) = find_attribute("value")
                                {
                                    let variants = enum_variants
                                        .get_mut(extends)
                                        .unwrap();
                                    let doc = find_attribute("comment")
                                        .map(|attr| attr.value.clone());
                                    variants.add(
                                        name,
                                        syn::LitInt::new(&value.value, Span::call_site()),
                                        doc
                                    );
                                }
                            }
                        } else {
                            if name.local_name == "remove" {
                                *parsing_remove = true;
                            } else if !*parsing_remove &&
                                let Some(name) = find_attribute("name")
                            {
                                sc_exclude_list.insert(name.value.trim_start_matches("Vk").to_string());
                            }
                        }
                    },
                    DepthType::Extensions => {
                        let ReaderElement::Extensions {
                            parsing_extension,
                        } = &mut reader.element else {
                            unreachable!()
                        };
                        if let Some(ext) = parsing_extension {
                            if let Some((_, is_sc)) = ext.parsing_requires {
                                if !is_sc {
                                    if name.local_name == "enum" {
                                        let name = find_attribute("name")
                                            .unwrap().value.clone();
                                        if let Some(value) = find_attribute("value")
                                            .map(|attr| &attr.value)
                                        {
                                            if let Some(extends) = &find_attribute("extends") {
                                                let variant = enum_variants
                                                    .get_mut(&extends.value)
                                                    .unwrap();
                                                let doc = find_attribute("comment")
                                                    .map(|attr| attr.value.clone());
                                                variant.add(
                                                    name,
                                                    syn::LitInt::new(value, Span::call_site()),
                                                    doc,
                                                );
                                            } else {
                                                let constant = if value.starts_with("\"") {
                                                    ExtConstant::String(value
                                                        .trim_start_matches("\"")
                                                        .trim_end_matches("\"")
                                                        .to_string()
                                                    )
                                                } else {
                                                    ExtConstant::U32(str::parse(value).unwrap())
                                                };
                                                ext.ext.new_constants.push((name, constant));
                                            }
                                        } else if let Some(offset) = find_attribute("offset")
                                        {
                                            let offset = str::parse(&offset.value).unwrap();
                                            let extends = &find_attribute("extends")
                                                .unwrap().value;
                                            let extnumber = find_attribute("extnumber")
                                                .map(|attr| str::parse(&attr.value).unwrap())
                                                .unwrap_or(ext.ext.number);
                                            let variant = enum_variants
                                                .get_mut(extends)
                                                .unwrap();
                                            let dir = find_attribute("dir")
                                                .map(|attr| &attr.value);
                                            let doc = find_attribute("comment")
                                                .map(|attr| attr.value.clone());
                                            variant.add_ext(
                                                name,
                                                offset,
                                                extnumber,
                                                dir,
                                                doc,
                                            );
                                        } else if let Some(bitpos) = find_attribute("bitpos")
                                        {
                                            let bitpos: u32 = str::parse(&bitpos.value).unwrap();
                                            let extends = &find_attribute("extends")
                                                .unwrap().value;
                                            let bits = bitmask_bits
                                                .get_mut(extends)
                                                .unwrap();
                                            let doc = find_attribute("comment")
                                                .map(|attr| attr.value.clone());
                                            bits.bits.insert(Bit {
                                                name,
                                                bit: 1 << bitpos,
                                                doc,
                                                alias: None,
                                                deprecated: None,
                                            });
                                        } else if let Some(alias) = find_attribute("alias") 
                                        {
                                            if let Some(extends) = find_attribute("extends")
                                                .map(|attr| &attr.value) {
                                                let deprecated = find_attribute("deprecated")
                                                    .map(|attr| attr.value.clone());
                                                if let Some(variants) = enum_variants.get_mut(extends) {
                                                    variants.add_alias(name, alias.value.clone(), deprecated);
                                                } else if let Some(bits) = bitmask_bits.get_mut(extends)
                                                {
                                                    bits.bits.insert(Bit {
                                                        name,
                                                        bit: 0,
                                                        doc: None,
                                                        alias: Some(alias.value.clone()),
                                                        deprecated: find_attribute("deprecated")
                                                            .map(|attr| attr.value.clone()),
                                                    });
                                                }
                                            } else {
                                                ext.ext.new_constants.push((
                                                    name,
                                                    ExtConstant::Alias(alias.value.clone()),
                                                ));
                                            }
                                        }
                                    }
                                } else {
                                    if let Some(name) = find_attribute("name") {
                                        sc_exclude_list.insert(
                                            name.value
                                                .trim_start_matches("Vk")
                                                .to_string()
                                        );
                                    }
                                }
                            } else if name.local_name == "require" {
                                ext.parsing_requires = Some((current_depth, depth.vulkan_sc.is_some()));
                            }
                        } else if name.local_name == "extension" && 
                            let Some(author) = find_attribute("author")
                        {
                            *parsing_extension = Some(ExtParsing {
                                name: find_attribute("name")
                                    .unwrap()
                                    .value.clone(),
                                ext: Extension {
                                    author: author.value.clone(),
                                    number: str::parse(&find_attribute("number")
                                        .unwrap()
                                        .value
                                    ).unwrap(),
                                    new_constants: vec![],
                                    commands: vec![],
                                },
                                parsing_requires: None,
                            })
                        }
                    },
                };
            },
            Ok(XmlEvent::Characters(string)) => {
                if depth.vulkan_sc.is_some() { continue }
                if let ReaderElement::Type(ty) = &mut reader.element {
                    match ty {
                        Type::Bitmask(bitmask) => {
                            if bitmask.parsing_type {
                                bitmask.ty = string;
                                bitmask.parsing_type = false;
                            } else if bitmask.parsing_name {
                                reader.element_name = Some(string);
                                bitmask.parsing_name = false;
                            }
                        },
                        Type::Struct(structure) => {
                            if let Some(parsing_member) = &mut structure.parsing_member {
                                match parsing_member.parsing {
                                    StructParsing::Const => {
                                        if string.starts_with("const") {
                                            parsing_member.is_const_ptr = true;
                                        }
                                        parsing_member.parsing = StructParsing::None;
                                    },
                                    StructParsing::Type => {
                                        parsing_member.ty = string
                                            .trim_start_matches("Vk")
                                            .to_string();
                                        parsing_member.parsing = StructParsing::Pointer;
                                    },
                                    StructParsing::Pointer => {
                                        if string.starts_with("**") {
                                            assert!(!parsing_member.is_const_ptr);
                                            parsing_member.ptr_type = PtrType::MutPtrMutPtr;
                                        } else if string.starts_with("* const*") ||
                                            string.starts_with("* const *")
                                        {
                                            assert!(parsing_member.is_const_ptr);
                                            parsing_member.ptr_type = PtrType::ConstPtrConstPtr;
                                        } else if string.starts_with("*  ") ||
                                            string == "*" || string == "* "
                                        {
                                            if parsing_member.is_const_ptr {
                                                parsing_member.ptr_type = PtrType::ConstPtr
                                            } else { parsing_member.ptr_type = PtrType::MutPtr }
                                        } else {
                                            assert!(!string.starts_with("*"), "{string}")
                                        };
                                        parsing_member.parsing = StructParsing::None;
                                    },
                                    StructParsing::Name => {
                                        let mut name = lower_snake_case(&string);
                                        if name == "p_next" {
                                            structure.needs_lifetime = true;
                                        }
                                        parsing_member.name = name;
                                        parsing_member.parsing = StructParsing::ArrayLen;
                                    },
                                    StructParsing::ArrayLen => {
                                        if string.contains("[") {
                                            continue
                                        } else if string.contains(":24") {
                                            structure.packed_u24_u8 = Some(parsing_member.name.clone());
                                        } else if string.contains(":8") {
                                            let packed_u24_u8 = structure.packed_u24_u8.take().unwrap();
                                            parsing_member.name = format!("{packed_u24_u8}_and_{}", parsing_member.name);
                                            parsing_member.ty = "packed_u24_u8".to_string();
                                        } else if let Some(packed) = structure.packed_u9_u9_u6_u4_u4.take() {
                                            use packed_u9_u9_u6_u4_u4 as E;
                                            structure.packed_u9_u9_u6_u4_u4 = match packed {
                                                E::A(name) => Some(E::B(format!("{name}_and_{}_", structure.name))),
                                                E::B(name) => Some(E::C(format!("{name}_and_{}_", structure.name))),
                                                E::C(name) => Some(E::D(format!("{name}_and_{}_", structure.name))),
                                                E::D(name) => Some(E::E(format!("{name}_and_{}_", structure.name))),
                                                E::E(name) => {
                                                    parsing_member.name = format!("{name}_and_{}", parsing_member.name);
                                                    parsing_member.ty = "packed_u9_u9_u6_u4_u4".to_string();
                                                    None
                                                },
                                            }
                                        } else if string.contains(":9") {
                                            structure.packed_u9_u9_u6_u4_u4 = Some(
                                                packed_u9_u9_u6_u4_u4::A(parsing_member.name.clone())
                                            );
                                        } else {
                                            parsing_member.array_len = Some(string
                                                .trim_start_matches("VK_")
                                                .to_string()
                                            );
                                            parsing_member.parsing = StructParsing::None;
                                        }
                                    },
                                    StructParsing::Comment => {
                                        parsing_member.comment = Some(string);
                                        parsing_member.parsing = StructParsing::None;
                                    },
                                    StructParsing::None => {},
                                }
                            }
                        },
                        Type::Handle(handle) => {
                            if handle.parsing_ty {
                                if string == "VK_DEFINE_HANDLE" {
                                    handle.ty = HandleType::Dispatchable;
                                } else if string == "VK_DEFINE_NON_DISPATCHABLE_HANDLE" {
                                    handle.ty = HandleType::NonDispatchable;
                                } else { unreachable!() }
                                handle.parsing_ty = false;
                            } else if handle.parsing_name {
                                reader.element_name = Some(string.trim_start_matches("Vk").to_string());
                                handle.parsing_name = false;
                            }
                        },
                        Type::Funcpointer(funcpointer) => {
                            if let FuncpointerParsing::Proto(parsing)
                                = &mut funcpointer.parsing 
                            {
                                match parsing {
                                    FuncParsing::Const => {
                                        assert!(string == "const", "{string}");
                                        funcpointer.ret_is_const = true;
                                        *parsing = FuncParsing::Ty;
                                    },
                                    FuncParsing::Ty => {
                                        funcpointer.ret_ty = string;
                                        *parsing = FuncParsing::Pointer;
                                    },
                                    FuncParsing::Pointer => {
                                        if string.starts_with("**") {
                                            assert!(!funcpointer.ret_is_const);
                                            funcpointer.ret_ptr = PtrType::MutPtrMutPtr;
                                        } else if string.starts_with("* const*") {
                                            assert!(funcpointer.ret_is_const);
                                            funcpointer.ret_ptr = PtrType::ConstPtrConstPtr;
                                        } else if string.starts_with("*  ") ||
                                            string == "*" || string == "* "
                                        {
                                            if funcpointer.ret_is_const {
                                                funcpointer.ret_ptr = PtrType::ConstPtr
                                            } else { funcpointer.ret_ptr = PtrType::MutPtr }
                                        } else {
                                            assert!(!string.starts_with("*"), "{string}");
                                        };
                                        *parsing = FuncParsing::Name;
                                    },
                                    FuncParsing::Name => {
                                        reader.element_name = Some(string);
                                        funcpointer.parsing = FuncpointerParsing::None;
                                    },
                                    FuncParsing::Array => unreachable!(),
                                }
                            } else if let FuncpointerParsing::Param {
                                parsing,
                                is_const,
                                ty,
                                ptr,
                                name,
                            } = &mut funcpointer.parsing {
                                match parsing {
                                    FuncParsing::Const => {
                                        *parsing = FuncParsing::Ty;
                                        if string.starts_with("const") {
                                            *is_const = true;
                                        }
                                    },
                                    FuncParsing::Ty => {
                                        *ty = string;
                                        *parsing = FuncParsing::Pointer;
                                    },
                                    FuncParsing::Pointer => {
                                        if string.starts_with("**") {
                                            assert!(!*is_const);
                                            *ptr = FuncParamPtrType::PtrType(PtrType::MutPtrMutPtr);
                                        } else if string.starts_with("* const*") {
                                            assert!(*is_const);
                                            *ptr = FuncParamPtrType::PtrType(PtrType::ConstPtrConstPtr);
                                        } else if string.starts_with("*  ") ||
                                            string == "*" || string == "* "
                                        {
                                            if *is_const {
                                                *ptr = FuncParamPtrType::PtrType(PtrType::ConstPtr);
                                            } else {
                                                *ptr = FuncParamPtrType::PtrType(PtrType::MutPtr);
                                            }
                                        } else {
                                            assert!(!string.starts_with("*"), "{string}");
                                        };
                                        *parsing = FuncParsing::Name;
                                    },
                                    FuncParsing::Name => {
                                        *name = string;
                                        *parsing = FuncParsing::Array;
                                    },
                                    FuncParsing::Array => {
                                        assert!(matches!(*ptr, FuncParamPtrType::PtrType(PtrType::None)));
                                        if let Some(idx) = string.find("[") {
                                            let str = string.split_at(idx + 1).1;
                                            let idx = str.find("]").unwrap();
                                            let str = str.split_at(idx).0;
                                            *ptr = FuncParamPtrType::Array(str::parse(str).unwrap());
                                        }
                                    },
                                }
                            }
                        },
                    }
                }
            },
            Ok(XmlEvent::EndElement { name }) => {
                reader.depth -= 1;
                if let Some(d) = depth.vulkan_sc 
                {
                    if d == reader.depth {
                        depth.vulkan_sc = None;
                    }
                    if !matches!(depth.ty, DepthType::Feature | DepthType::Extensions) {
                        continue
                    }
                }
                match depth.ty {
                    DepthType::Types => {
                        if name.local_name == "types" {
                            depth.ty = DepthType::Unknown;
                            reader.element = ReaderElement::Inactive;
                            reader.element_name = None;
                        } else if depth.depth + 1 == reader.depth {
                            if let ReaderElement::Type(ty) = &mut reader.element
                            {
                                match ty {
                                    Type::Bitmask(bitmask) => {
                                        bitmask_bits
                                            .insert(
                                                bitmask.requires
                                                    .clone()
                                                    .unwrap_or_else(||
                                                        reader.element_name
                                                            .as_ref()
                                                            .unwrap()
                                                            .replace("Flags", "FlagBits")
                                                    ),
                                                BitmaskBits {
                                                    bits: IndexSet::new(),
                                                    flag_type:
                                                        if bitmask.ty == "VkFlags" {
                                                            FlagType::U32
                                                        } else if bitmask.ty == "VkFlags64" {
                                                            FlagType::U64
                                                        } else { panic!("{:?}", reader.element_name) }
                                                }
                                            );
                                    },
                                    Type::Struct(structure) => {
                                        structure.name = reader.element_name
                                            .as_ref()
                                            .unwrap()
                                            .trim_start_matches("Vk")
                                            .to_string();
                                        if let Some(extends) = &structure.extends_unparsed {
                                            let idx = extends.find(",").unwrap_or(extends.len() - 1);
                                            let (mut a, mut b) = extends.split_at(idx + 1);
                                            while let Some(idx) = b.find(",") {
                                                let s = a
                                                    .trim_end_matches(",")
                                                    .trim_start_matches("Vk")
                                                    .to_string();
                                                structure.extends.push(Ident::new(
                                                    &format!("Extends{s}"),
                                                    Span::call_site())
                                                );
                                                struct_extends.insert(s);
                                                (a, b) = b.split_at(idx + 1);
                                            }
                                            let s = a
                                                .trim_end_matches(",")
                                                .trim_start_matches("Vk")
                                                .to_string();
                                            structure.extends.push(Ident::new(
                                                &format!("Extends{s}"),
                                                Span::call_site())
                                            );
                                            struct_extends.insert(s);
                                        }
                                        struct_defs.push(structure.clone());
                                    },
                                    Type::Handle(handle) => {
                                        let type_enum = Ident::new(
                                            handle.type_enum.trim_start_matches("VK_OBJECT_TYPE_"),
                                            Span::call_site(),
                                        );
                                        let name = reader.element_name.take().unwrap();
                                        let link = format!(
                                            "https://docs.vulkan.org/refpages/latest/refpages/source/Vk{name}.html",
                                        );
                                        let doc =
                                            if url_exists(&link) {
                                                let link = format!("<{link}>");
                                                quote! {
                                                    #link
                                                }
                                            } else {
                                                quote! {""}
                                            };
                                        let name = Ident::new(&name, Span::call_site());
                                        let def = match handle.ty {
                                            HandleType::NonDispatchable => {
                                                quote! {
                                                    define_non_dispatchable_handle!(
                                                        #name, #type_enum, #doc,
                                                    );
                                                }
                                            },
                                            HandleType::Dispatchable => {
                                                quote! {
                                                    define_handle!(
                                                        #name, #type_enum, #doc,
                                                    );
                                                }
                                            },
                                        };
                                        write!(handles, "{def}")?;
                                    },
                                    Type::Funcpointer(funcpointer) => {
                                        funcpointers.push((reader.element_name.take().unwrap(), Func {
                                            ret_ty: funcpointer.ret_ty.clone(),
                                            ret_ptr: funcpointer.ret_ptr,
                                            params: funcpointer.params.clone(),
                                        }));
                                    },
                                }
                            }
                            reader.element = ReaderElement::Inactive;
                            reader.element_name = None;
                        } else if 
                            let ReaderElement::Type(ty) = &mut reader.element &&
                            let Type::Struct(structure) = ty &&
                            name.local_name == "member"
                        {
                            let member = structure.parsing_member.take().unwrap();
                            if structure.packed_u24_u8.is_none() && structure.packed_u9_u9_u6_u4_u4.is_none() {
                                structure.members.push(member);
                            }
                        } else if
                            let ReaderElement::Type(ty) = &mut reader.element &&
                            let Type::Funcpointer(funcpointer) = ty
                        {
                            if name.local_name == "proto" {
                                funcpointer.parsing = FuncpointerParsing::None;
                            } else if name.local_name == "param" {
                                let FuncpointerParsing::Param {
                                    ty, ptr, name, ..
                                } = funcpointer.parsing.clone() else { unreachable!() };
                                funcpointer.params.push(Funcparam { ty, ptr, name });
                                funcpointer.parsing = FuncpointerParsing::None;
                            }
                        }
                    },
                    DepthType::Enums => {
                        if name.local_name == "enums" {
                            depth.ty = DepthType::Unknown;
                            reader.element = ReaderElement::Inactive;
                            reader.element_name = None;
                        }
                    },
                    DepthType::Commands => {
                        if name.local_name == "commands" {
                            depth.ty = DepthType::Unknown;
                            reader.element = ReaderElement::Inactive;
                            reader.element_name = None;
                        } else if name.local_name == "command" &&
                            let ReaderElement::Type(ty) = &mut reader.element &&
                            let Type::Funcpointer(funcpointer) = ty
                        {
                            commands.insert(
                                reader.element_name.clone().unwrap(),
                                Command {
                                    body: CommandBody::Func(Func {
                                        ret_ty: funcpointer.ret_ty.clone(),
                                        ret_ptr: funcpointer.ret_ptr,
                                        params: funcpointer.params.clone(),
                                    }),
                                    version: None,
                                    extension: None 
                                }
                            );
                        } else if
                            let ReaderElement::Type(ty) = &mut reader.element &&
                            let Type::Funcpointer(funcpointer) = ty
                        {
                            if name.local_name == "proto" {
                                funcpointer.parsing = FuncpointerParsing::None;
                            } else if name.local_name == "param" &&
                                !matches!(funcpointer.parsing, FuncpointerParsing::ImplicitParams)
                            {
                                let FuncpointerParsing::Param {
                                    ty, ptr, name, ..
                                } = funcpointer.parsing.clone() else { unreachable!() };
                                funcpointer.params.push(Funcparam { ty, ptr, name });
                                funcpointer.parsing = FuncpointerParsing::None;
                            } else if name.local_name == "implicitexternsyncparams" {
                                funcpointer.parsing = FuncpointerParsing::None;
                            }
                        }
                    },
                    DepthType::Feature => {
                        if name.local_name == "require" &&
                            let ReaderElement::Feature {
                                parsing_requires, ..
                            } = &mut reader.element
                        {
                            *parsing_requires = None;
                        } else if name.local_name == "feature" &&
                            depth.depth == reader.depth
                        {
                            let ReaderElement::Feature {
                                version, feature, is_sc, ..
                            } = &reader.element else { unreachable!() };
                            if !*is_sc {
                                features.entry(version.clone())
                                    .and_modify(|f| {
                                        f.commands.extend_from_slice(&feature.commands);
                                    }).or_insert_with(|| {
                                        feature.clone()
                                    });
                            }
                            reader.element = ReaderElement::Inactive;
                            depth.ty = DepthType::Unknown;
                        }
                    },
                    DepthType::Extensions => {
                        if name.local_name == "extensions" {
                            reader.element = ReaderElement::Inactive;
                            depth.ty = DepthType::Unknown;
                        } else if let ReaderElement::Extensions {
                            parsing_extension,
                        } = &mut reader.element
                        {
                            if name.local_name == "extension" {
                                let Some(ext) = parsing_extension.take() else {
                                    continue
                                };
                                extensions.insert(
                                    ext.name,
                                    ext.ext,
                                );
                            } else if let Some(ext) = parsing_extension &&
                                name.local_name == "require" &&
                                let Some((depth, _)) = ext.parsing_requires.take()
                            {
                                assert!(depth == reader.depth);
                            }
                        }
                    },
                    DepthType::Unknown => {},
                };
            },
            Err(e) => {
                eprintln!("error: {e}");
                break
            }
            _ => {},
        }
    }
    println!("{:?}", unique_categories);
    for (name_str, variants) in enum_variants {
        if name_str.find("FlagBits").is_some() { continue }
        let link = format!("https://docs.vulkan.org/refpages/latest/refpages/source/{name_str}.html");
        let doc = 
            if url_exists(&link) {
                let doc = format!("<{link}>");
                Some(quote! {#[doc = #doc]})
            } else {
                None
            };
        let name = Ident::new(name_str.trim_start_matches("Vk"), Span::call_site());
        let def = quote! {
           
            #doc
            #[repr(transparent)]
            #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
            pub struct #name(i32);

            impl #name {
                #[inline]
                pub const fn from_raw(x: i32) -> Self {
                    Self(x)
                }
                #[inline]
                pub const fn as_raw(self) -> i32 {
                    self.0
                }
            }
        };
        let mut display = vec![];
        let var1 = variants.variants.first().map(|var| var.name.as_str()).unwrap_or_default();
        let sc = name_str.trim_end_matches("EXT")
            .trim_end_matches("KHR")
            .trim_end_matches("NV");
        let mut sc = upper_snake_case(sc);
        if !sc.ends_with('_') {
            sc.push('_');
        }
        let mut i = 0;
        while var1.chars().nth(i) == sc.chars().nth(i) { i += 1; }
        let replace = var1.split_at(i).0;
        let mut var_names = vec![];
        let variants = variants
            .variants
            .iter().map(|Variant { name, value, doc, alias, deprecated, }| {
                let mut name_str = name
                    .trim_start_matches(replace)
                    .trim_start_matches("VK_")
                    .to_uppercase();
                if name_str.starts_with(char::is_numeric) {
                    name_str.insert_str(0, "TYPE_")
                }
                let name = Ident::new(&name_str, Span::call_site());
                let doc = doc.as_ref().map(|doc| quote! { #[doc = #doc] });
                var_names.push(name.clone());
                let deprecated = deprecated
                    .as_ref()
                    .map(|reason| quote! { #[deprecated = #reason] });
                let value = if let Some(alias) = alias {
                    let mut alias_str = alias
                        .trim_start_matches(replace)
                        .to_uppercase();
                    if alias_str.starts_with(char::is_numeric) {
                        alias_str.insert_str(0, "TYPE_")
                    }
                    let alias = Ident::new(&alias_str, Span::call_site());
                    quote! { Self::#alias }
                } else {
                    display.push(quote! {
                        Self::#name => write!(f, #name_str),
                    });
                    quote! { Self(#value) }
                };
                quote! {
                    #doc
                    #deprecated
                    pub const #name: Self = #value;
                }
            });
        let variants = quote! {
            impl #name {
                #(#variants)*
            }
        };
        let body = if var_names.is_empty() {
            quote! { write!(f, "{}", self.0) }
        } else if var_names.len() == 1 {
            let var = var_names.first().unwrap();
            let fmt = var.to_string();
            quote! {
                if *self == Self::#var {
                    write!(f, #fmt)
                } else {
                    write!(f, "{}", self.0)
                }
            }
        } else {
            quote! {
                match *self {
                    #(#display)*
                    x => write!(f, "{x}"),
                }
            }
        };
        let display = quote! {
            impl fmt::Display for #name {
                
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    #body
                }
            }
        };
        write!(enums, "{def}{variants}{display}")?;
    }
    for (requires, bits) in &bitmask_bits {
        let requires = requires.trim_start_matches("Vk");
        let name = requires.replace("FlagBits", "Flags");
        let link = format!("https://docs.vulkan.org/refpages/latest/refpages/source/{name}.html");
        let doc = 
            if url_exists(&link) {
                let doc = format!("<{link}>");
                Some(quote! {#[doc = #doc]})
            } else {
                None
            };
        let name = Ident::new(&name, Span::call_site());
        let ty = match bits.flag_type {
            FlagType::U32 => Ident::new("Flags", Span::call_site()),
            FlagType::U64 => Ident::new("Flags64", Span::call_site())
        };
        let def = quote! {
            #doc
            #[repr(transparent)]
            #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct #name(#ty);
            bitflags!(#name, #ty); 
        };
        let requires_sc = upper_snake_case(requires);
        let idx = requires_sc.find("FLAG_BITS").unwrap_or_default();
        let mut requires_sc = requires_sc
            .split_at(idx).0
            .trim_start_matches("_")
            .to_string();
        if requires.ends_with("2")
        {
            requires_sc.push_str("2_");
        } else if requires.ends_with("3") ||
            requires.ends_with("3KHR")
        {
            requires_sc.push_str("3_");
        }
        let mut variants = vec![];
        let mut display = vec![];
        let as_u64 = (bits.flag_type == FlagType::U32).then(||
            quote! { as u64 }
        );
        let mut duplicates = HashSet::new();
        for Bit { name, bit, doc, alias, deprecated }  in &bits.bits {
            let mut name_str = name
                .trim_start_matches("VK_")
                .replace("_BIT", "")
                .replace(&requires_sc, "");
            if !duplicates.insert(name_str.clone()) {
                continue
            }
            if name_str.starts_with(char::is_numeric) {
                name_str.insert_str(0, "TYPE_");
            }
            let name = Ident::new(&name_str, Span::call_site());
            let doc = doc.as_ref().map(|doc| quote! { #[doc = #doc] });
            let deprecated = deprecated
                .as_ref()
                .map(|reason| quote! { #[deprecated = #reason] });
            let value = if let Some(alias) = alias {
                let mut alias_str = alias
                    .trim_start_matches("VK_")
                    .replace("_BIT", "")
                    .replace(&requires_sc, "");
                if alias_str.starts_with(char::is_numeric) {
                    alias_str.insert_str(0, "TYPE_");
                }
                let alias = Ident::new(&alias_str, Span::call_site());
                quote! {
                    Self::#alias
                }
            } else {
                if *bit != 0 {
                    display.push(quote! {
                        (#bit, #name_str),
                    });
                }
                let bit = syn::LitInt::new(&format!("0x{:x}", bit), Span::call_site());
                quote! {
                    Self(#bit)
                }
            };
            variants.push( quote! {
                #doc
                #deprecated
                pub const #name: Self = #value;
            });
        }
        let variants = quote! {
            impl #name {
                #(#variants)*
            }
        };
        let display = quote! {

            impl fmt::Display for #name {

                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    if self.is_empty() {
                        write!(f, "NONE")?;
                    }
                    const BITS: &[(u64, &str)] = &[
                        #(#display)*
                    ];
                    flag_display(
                        self.0 #as_u64,
                        BITS,
                        f,
                    )
                }
            }
        };
        write!(enums, "{def}{variants}{display}")?;
    }
    let not_send_sync_types: HashSet<&'static str> =
        [
            "LPCWSTR",
            "HANDLE",
            "NvSciSyncAttrList",
            "NvSciSyncObj",
            "NvSciBufAttrList",
            "NvSciBufObj",
            "MTLDevice_id",
            "MTLCommandQueue_id",
            "MTLBuffer_id",
            "MTLTexture_id",
            "MTLSharedEvent_id",
            "IOSurfaceRef",
            "RemoteAddressNV",
        ].into_iter().collect();
    let mut type_needs_lifetime: IndexSet<String> =
        struct_defs
            .iter()
            .filter(|def| def.needs_lifetime)
            .map(|def| def.name.clone())
            .collect();
    loop {
        let new: Vec<_> = struct_defs
            .iter()
            .filter(|def| {
                if type_needs_lifetime.contains(&def.name) {
                    return false
                }
                for member in &def.members {
                    if type_needs_lifetime.contains(&member.ty) {
                        return true;
                    }
                }
                false
            }).map(|def| def.name.clone()).collect();
        if new.is_empty() {
            break
        }
        type_needs_lifetime.extend(new);
    }
    let mut defined_aliases = HashSet::new(); 
    let resolve_ty = |ty: &str| -> Option<proc_macro2::TokenStream> {
        Some(if ty.starts_with("uint") {
            let ident = syn::Ident::new(
                &format!("u{}", ty
                    .trim_start_matches("uint")
                    .trim_end_matches("_t")
                ),
                Span::call_site()
            );
            quote! { #ident }
        } else if ty.starts_with("int") {
            if ty == "int" {
                quote! { ffi::c_int }
            } else {
                let ident = syn::Ident::new(
                    &format!("i{}", ty
                        .trim_start_matches("int")
                        .trim_end_matches("_t")
                    ),
                    Span::call_site()
                );
                quote! { #ident }
            }
        } else if ty == "size_t" {
            quote! { usize }
        } else if ty == "float" {
            quote! { f32 }
        }  else if ty == "double" {
            quote! { f64 }
        } else if ty == "void" {
            quote! { ffi::c_void }
        } else if ty == "char" {
            quote! { ffi::c_char }
        } else {
            let ty = ty.replace("FlagBits", "Flags");
            let ty = ty
                .trim_start_matches("Vk");
            if sc_exclude_list.contains(ty) {
                return None
            }
            let ident = syn::Ident::new(
                ty,
                Span::call_site()
            );
            quote! { #ident }
        })
    };
    for def in struct_defs {
        if sc_exclude_list.contains(&def.name) {
            continue
        }
        let mut field_defs = vec![];
        let mut field_defaults = vec![];
        let mut field_setters = vec![];
        let mut prev_field = None;
        let mut has_pointer = false;
        let mut p_next_mutability = None;
        let mut needs_lifetime = def.needs_lifetime;
        let mut s_type = None;
        let mut derive_default = true;
        for member in &def.members {
            let resolved_ty = resolve_ty(&member.ty).unwrap();
            if not_send_sync_types.contains(member.ty.as_str()) {
                has_pointer = true;
            }
            let pointer = member.ptr_type;
            if let Some(prev_field) = prev_field.take() &&
                prev_field == member.name
            {
                continue
            }
            prev_field = Some(member.name.clone());
            if member.name == "p_next" {
                p_next_mutability = match member.ptr_type {
                    PtrType::ConstPtr => Some(false),
                    PtrType::MutPtr => Some(true),
                    _ => unreachable!(),
                };
            }
            let lifetime = (type_needs_lifetime.contains(&member.ty)).then(||
            {
                needs_lifetime = true;
                quote! { <'a> }
            });
            let array;
            let ty =
                if let Some(len) = &member.array_len {
                    let len =
                        if len.starts_with(char::is_numeric) {
                            quote! { #len }
                        } else {
                            let len = Ident::new(len, Span::call_site());
                            quote! { #len as usize }
                        };
                    let q = quote! {
                        [#pointer #resolved_ty #lifetime; #len]
                    };
                    array = Some(len);
                    q
                } else {
                    array = None;
                    quote! {
                        #pointer #resolved_ty #lifetime
                    }
                };
            let has_s_type =
                if let Some(value) = &member.value {
                    let ident = Ident::new(
                        value.trim_start_matches("VK_STRUCTURE_TYPE_"),
                        Span::call_site()
                    );
                    s_type = Some(ident);
                    true
                } else {
                    false
                };
            let doc = member.comment
                .as_ref()
                .map(|comment| {
                    quote! {#[doc = #comment]}
                });
            let deprecated = member.deprecated
                .as_ref()
                .map(|reason| {
                    quote! {#[deprecated = #reason]}
                });
            let name = if member.name == "type" {
                "ty"
            } else { &member.name };
            let name = Ident::new(name, Span::call_site());
            field_defs.push(
                quote! {
                    #doc
                    #deprecated
                    pub #name: #ty,
                }
            );
            if !def.is_union {
                if deprecated.is_none() {
                    field_setters.push(
                        if member.ty == "Bool32" &&
                            member.ptr_type == PtrType::None
                        {
                            quote! {
                                #[inline]
                                pub const fn #name(mut self, value: bool) -> Self {
                                    self.#name = value as Bool32;
                                    self
                                }
                            }
                        } else {
                            quote! {
                                #[inline]
                                pub const fn #name(mut self, value: #ty) -> Self {
                                    self.#name = value;
                                    self
                                }
                            }
                        }
                    );
                }
                field_defaults.push(
                    if has_s_type {
                        derive_default = false;
                        quote! {
                            #name: Self::S_TYPE,
                        }
                    } else if let Some(array) = array {
                        derive_default = false;
                        quote! {
                            #name: [#resolved_ty::default(); #array],
                        }
                    } else {
                        match member.ptr_type {
                            PtrType::None => {
                                if member.ty.starts_with("PFN") {
                                    derive_default = false;
                                }
                                quote! {
                                    #name: #resolved_ty::default(),
                                }
                            },
                            PtrType::MutPtr | PtrType::MutPtrMutPtr => {
                                quote! {
                                    #name: ::core::ptr::null_mut(),
                                }
                            },
                            PtrType::ConstPtr | PtrType::ConstPtrConstPtr => {
                                quote! {
                                    #name: ::core::ptr::null(),
                                }
                            },
                        }
                    }
                );
            }
        }
        let name_str = &def.name;
        let link = format!("https://docs.vulkan.org/refpages/latest/refpages/source/Vk{name_str}.html");
        let doc = 
            if url_exists(&link) {
                let doc = format!("<{link}>");
                Some(quote! {#[doc = #doc]})
            } else {
                None
            };
        let name = Ident::new(name_str, Span::call_site());
        if !def.is_union {
            let mut extends_trait = None;
            let push_next = struct_extends
                .contains(name_str)
                .then(|| {
                    let p_next_mutability = if p_next_mutability.unwrap() {
                        quote! { mut }
                    } else { quote! { const } };
                    let trait_name = Ident::new(
                        &format!("Extends{name_str}"),
                        Span::call_site(),
                    );
                    let doc = format!("A trait for structures that extend {name_str}.");
                    let doc = quote! {
                        #[doc = #doc]
                        ///
                        /// # Safety
                        /// A struct implementing this trait *must* adhere to the memory layout of
                        /// [`BaseOutStrucutre`].
                    };
                    extends_trait = Some(quote! {
                        #doc
                        pub unsafe trait #trait_name<'a>: Chainable<'a> {}
                    });
                    quote! {

                        /// Pushes an item to the [`p_next`][1] chain of this structure.
                        ///
                        /// [1]: Self::p_next
                        #[inline]
                        pub fn push_next<T>(mut self, next: &'a mut T) -> Self
                            where T: ?Sized + #trait_name<'a>,
                        {
                            let p_next: *#p_next_mutability ffi::c_void = <*mut T>::cast(next);
                            let last = chain_iter(next).last().unwrap();
                            last.p_next = self.p_next as *mut BaseOutStructure<'a>;
                            self.p_next = p_next;
                            self
                        }
                    }
                });
            let lifetime = needs_lifetime.then(|| {
                field_defs.push(quote! {
                    pub _marker: PhantomData<&'a ()>,
                });
                field_defaults.push(quote! {
                    _marker: PhantomData,
                });
                quote! { <'a> }
            });
            let impl_send_sync = has_pointer.then(|| quote! {
                unsafe impl #lifetime Send for #name #lifetime {}
                unsafe impl #lifetime Sync for #name #lifetime {}
            });
            let impl_chainable = (s_type.is_some() && p_next_mutability.is_some()).then(||
                quote! {
                    unsafe impl<'a> Chainable<'a> for #name<'a> {

                        #[inline]
                        fn base_out(&mut self) -> &'a mut BaseOutStructure<'a> {
                            unsafe {
                                &mut *<*mut Self>::cast::<BaseOutStructure>(self)
                            }
                        }
                    }
                }
            );
            let s_type = s_type.map(|s_type| quote! {
                pub const S_TYPE: StructureType = StructureType::#s_type;
            });
            let (derive_default, impl_default) = if derive_default {
                (quote! { Default, }, quote! {})
            } else {
                (quote! {}, quote! {

                    impl #lifetime Default for #name #lifetime {
                            
                        #[allow(deprecated)]
                        fn default() -> Self {
                            Self {
                                #(#field_defaults)*
                            }
                        }
                    }
                })
            };
            let impl_extends = def.extends
                .iter()
                .map(|extends| quote! {
                    unsafe impl<'a> #extends<'a> for #name<'a> {}
                });
            let def = quote! {
                #doc
                #[repr(C)]
                #[derive(#derive_default Clone, Copy)]
                pub struct #name #lifetime {
                    #(#field_defs)*
                }

                #extends_trait

                impl #lifetime #name #lifetime {
                    #s_type
                    #(#field_setters)*
                    #push_next
                }

                #impl_default

                #impl_chainable

                #(#impl_extends)*

                #impl_send_sync
            };
            write!(structs, "{def}")?;
        } else {
            let lifetime = needs_lifetime.then(|| {
                quote! { <'a> }
            });
            let def = quote! {
                #doc
                #[repr(C)]
                #[derive(Clone, Copy)]
                pub union #name #lifetime {
                    #(#field_defs)*
                }

                impl #lifetime Default for #name #lifetime {

                    #[inline]
                    fn default() -> Self {
                        unsafe { ::core::mem::zeroed() }
                    }
                }
            };
            write!(unions, "{def}")?;
        }
    }
    let mut pfn_duplicates = HashSet::new();
    for (name, func) in funcpointers {
        let ret_ptr = func.ret_ptr;
        let ty = if ret_ptr == PtrType::None && func.ret_ty == "void" {
            quote! {}
        } else {
            let Some(ty) = resolve_ty(&func.ret_ty) else {
                continue
            };
            quote! {
                ->  #ret_ptr #ty
            }
        };
        let name = Ident::new(&name, Span::call_site());
        let mut skip_sc = false;
        let mut field_tys = vec![];
        let params: Vec<_> = func.params
            .iter()
            .map(|param| {
                let Some(ty) = resolve_ty(&param.ty) else {
                    skip_sc = true;
                    return quote! {}
                };
                let name = lower_snake_case(&param.name);
                let name = Ident::new(&name, Span::call_site());
                let ty = match param.ptr {
                    FuncParamPtrType::PtrType(ptr) => quote! { #ptr #ty },
                    FuncParamPtrType::Array(len) => quote! { [#ty; #len] },
                };
                field_tys.push(ty.to_string());
                quote! {
                    #name: #ty,
                }
            }).collect();
        let pfn_default = pfn_duplicates.insert(field_tys).then(|| quote! {
            impl PFN_Default for #name {

                #[inline]
                #[allow(unused_variables)]
                fn default() -> Self {
                    unsafe extern "system" fn f(
                        #(#params)*
                    ) #ty {
                        panic!(concat!(
                            stringify!(#name),
                            " not defined"
                        ))
                    }
                    f
                }
            }
        });
        let def = quote! {
            #[allow(non_camel_case_types)]
            pub type #name = unsafe extern "system" fn(
                #(#params)*
            ) #ty; 

            #pfn_default
        };
        if skip_sc {
            continue
        }
        write!(type_defs, "{def}")?;
    }
    for (name, command) in &commands {
        let doc = if let Some(version) = &command.version {
            let doc = format!("Provided by Vulkan version {version}");
            quote! {#[doc = #doc]}
        } else if let Some(ext) = &command.extension {
            let doc = format!("Provided by {ext}");
            quote! {#[doc = #doc]}
        } else { quote! {} };
        let CommandBody::Func(func) = &command.body else {
            continue
        };
        let ret_ptr = func.ret_ptr;
        let ty = if ret_ptr == PtrType::None && func.ret_ty == "void" {
            quote! {}
        } else {
            let Some(ty) = resolve_ty(&func.ret_ty) else {
                continue
            };
            quote! {
                ->  #ret_ptr #ty
            }
        };
        let name = Ident::new(&format!("PFN_{name}"), Span::call_site());
        let mut skip_sc = false;
        let params = func.params
            .iter()
            .map(|param| {
                let Some(ty) = resolve_ty(&param.ty) else {
                    skip_sc = true;
                    return quote! {}
                };
                let mut name = lower_snake_case(&param.name);
                if name == "type" {
                    name = "ty".to_string();
                }
                let name = Ident::new(&name, Span::call_site());
                let ty = match param.ptr {
                    FuncParamPtrType::PtrType(ptr) => quote! { #ptr #ty },
                    FuncParamPtrType::Array(len) => quote! { [#ty; #len] },
                };
                quote! {
                    #name: #ty,
                }
            });
        let def = quote! {
            #doc
            #[allow(non_camel_case_types)]
            pub type #name = unsafe extern "system" fn(
                #(#params)*
            ) #ty;
        };
        if skip_sc {
            continue
        }
        write!(type_defs, "{def}")?;
    }
    let type_aliases = type_aliases
        .iter()
        .filter_map(|(name, alias)| {
            let link = format!("https://docs.vulkan.org/refpages/latest/refpages/source/{name}.html");
            let doc = 
                if url_exists(&link) {
                    let doc = format!("<{link}>");
                    Some(quote! {#[doc = #doc]})
                } else {
                    None
                };
            let name_str = name.trim_start_matches("Vk").replace("FlagBits", "Flags");
            if defined_aliases.contains(&name_str) {
                return None
            }
            let non_camel = name_str.contains('_').then(|| quote! {
                #[allow(non_camel_case_types)]
            });
            let name = Ident::new(&name_str, Span::call_site());
            defined_aliases.insert(name_str);
            let alias_str = &alias.trim_start_matches("Vk").replace("FlagBits", "Flags");
            let alias = Ident::new(alias_str, Span::call_site());
            if type_needs_lifetime.contains(alias_str) {
                type_needs_lifetime.insert(alias_str.to_string());
                Some(quote! {
                    #doc
                    pub type #name<'a> = #alias<'a>;
                })
            } else {
                Some(quote! {
                    #doc
                    #non_camel
                    pub type #name = #alias;
                })
            }
        });
    write!(type_defs, "{}", quote! {
        #(#type_aliases)*
    })?;
    for (version, feature) in features {
    }
    let mut ext_dirs = HashSet::new();
    for (name, ext) in extensions {
        let dir = format!("../../src/{}", ext.author.to_lowercase());
        if ext_dirs.insert(ext.author.to_lowercase()) {
            //DirBuilder::new()
                //.recursive(true)
                //.create(&dir)?;
        }
    }
    core::mem::drop(handles);
    core::mem::drop(enums);
    core::mem::drop(structs);
    core::mem::drop(unions);
    core::mem::drop(type_defs);
    process::Command::new("rustfmt")
        .arg("../../src/vk/handles.rs")
        .arg("../../src/vk/enums.rs")
        .arg("../../src/vk/structs.rs")
        .arg("../../src/vk/unions.rs")
        .arg("../../src/vk/type_defs.rs")
        .output()?;
    Ok(())
}
