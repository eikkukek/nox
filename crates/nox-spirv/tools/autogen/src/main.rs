use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    collections::{HashMap, HashSet},
    process::Command,
};

use quote::quote;
use proc_macro2::{Ident, Span};

fn upper_snake_case(name: &str) -> Ident {
    let mut uppercase = vec![];
    let mut last_uppercase = 0;
    let mut last_char = ' ';
    for (i, ch) in name.char_indices().skip(1) {
        if ch.is_uppercase() {
            if last_uppercase != i - 1 && last_char != '_' {
                uppercase.push(i);
            }
            last_uppercase = i;
        }
        last_char = ch;
    }
    let mut snake_case = name.to_uppercase();
    for idx in uppercase.into_iter().rev() {
        snake_case.insert(idx, '_');
    }
    if snake_case.chars().next().unwrap().is_numeric() {
        snake_case.insert_str(0, "TYPE_");
    }
    Ident::new(
        &snake_case,
        Span::call_site(),
    )
}

fn field_name(name: &str) -> Ident {
    let mut name = name.to_string();
    if let Some(idx) = name.find("<<") {
        let link = name.split_off(idx);
        let idx = link.find(",").unwrap();
        name.push_str(link.split_at(idx + 1).1);
    }
    let mut uppercase = vec![];
    let mut last_uppercase = 0;
    let mut last_char = ' ';
    for (i, ch) in name.char_indices().skip(1) {
        if ch.is_uppercase() {
            if last_uppercase != i - 1 && last_char.is_alphabetic() {
                uppercase.push(i);
            }
            last_uppercase = i;
        }
        last_char = ch;
    }
    for idx in uppercase.into_iter().rev() {
        name.insert(idx, '_');
    }
    let mut name = name
        .replace("<<", "")
        .replace(">>", " ")
        .replace(" ", "_")
        .replace("-", "_")
        .replace("D~ref~", "dref")
        .replace("'", "")
        .to_lowercase();
    if let Some(idx) = name.find("+\n") {
        let _ = name.split_off(idx);
        name = name.replace(",", "");
        name = name.replace("_0_", "s_and_");
        name = name.replace("_1_", "s");
    }
    if let Some(_) = name.find("...") &&
        let Some(idx) = name.find(",")
    {
        let _ = name.split_off(idx);
        name = name.replace("_1", "");
        name = name.replace("_0", "");
        name += "s";
    }
    if name == "type" {
        name = "ty".to_string();
    }
    if name == "use" {
        name = "using".to_string();
    }
    Ident::new(
        name
            .trim_end_matches("_")
            .trim_end_matches(".")
            .trim_end(),
        Span::call_site(),
    )
}

fn enum_gen(
    op_path: &Path,
    parsed: &json::JsonValue,
) -> io::Result<PathBuf>
{
    let path = op_path.with_file_name("op/enums.rs");
    let mut enums = File::create(&path)?;
    let struct_derive = quote! {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    };
    let enum_derive = quote! {
        #[derive(Clone, Copy, Debug)]
    };
    let header = quote! {
        //! SPIR-V enums
        //!
        //! This file is auto-generated, do not modify manually.
        use core::fmt::{self, Display};
        use core::ops::{BitAnd, BitOr};
        use crate::{
            op::*,
            stream::*,
            core::*,
            module::*,
        };

        struct BitFmt(u32);

        impl BitFmt {

            pub fn fmt(
                &mut self,
                value: u32,
                f: &mut fmt::Formatter<'_>
            ) -> fmt::Result {
                if self.0 != 0 {
                    write!(f, "|")?;
                }
                self.0 |= value;
                Ok(())
            }
        }
    };
    writeln!(enums, "{header}")?;
    for member in parsed["operand_kinds"].members() {
        let category = &member["category"];
        if category == "BitEnum" {
            let kind = Ident::new(
                member["kind"].as_str().unwrap(),
                Span::call_site()
            );
            let def = quote! {
                #struct_derive
                pub struct #kind(pub(crate) u32);
            };
            let enumerants = &member["enumerants"];
            let enumerants: Vec<_> = enumerants
                .members()
                .map(|enumerant| {
                    let name = enumerant["enumerant"].as_str().unwrap();
                    let snake_case = upper_snake_case(name);
                    let value = enumerant["value"].as_str().unwrap();
                    let lit = syn::LitInt
                        ::new(value, Span::call_site());
                    (
                        quote! {
                            pub const #snake_case: Self = Self(#lit);
                        },
                        { 
                            if u32::from_str_radix(&value.replace("0x", ""), 16)
                                .unwrap()
                                .count_ones() == 1
                            {
                                quote! {
                                    if *self & Self::#snake_case == Self::#snake_case
                                    {
                                        bitfmt.fmt(Self::#snake_case.0, f)?;
                                        write!(f, #name)?;
                                    }
                                }
                            } else { quote! {} }
                        },
                    )
                }).collect();
            let inherent = enumerants
                .iter()
                .map(|(def, _)| def);
            let inherent = quote! {
                impl #kind {
                    #(#inherent)*
                }
            };
            let word = quote! {
                impl Word for #kind {

                    #[inline]
                    fn from_word(word: u32) -> Self {
                        Self(word)
                    }
                }
            };
            let display = enumerants
                .iter()
                .map(|(_, display)| display);
            let display = quote! {
                impl Display for #kind {

                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        if self.0 == 0 {
                            return write!(f, "None")
                        }
                        let mut bitfmt = BitFmt(0);
                        #(#display)*
                        Ok(())
                    }
                }
            };
            let bitop = quote! {
                impl BitAnd for #kind {

                    type Output = Self;

                    #[inline]
                    fn bitand(self, rhs: Self) -> Self {
                        Self(self.0 & rhs.0)
                    }
                }

                impl BitOr for #kind {

                    type Output = Self;

                    #[inline]
                    fn bitor(self, rhs: Self) -> Self {
                        Self(self.0 | rhs.0)
                    }
                }
            };
            write!(enums, "{def}\n{inherent}\n{word}\n{bitop}\n{display}\n")?;
        }
        else if category == "ValueEnum" {
            let kind = Ident::new(
                member["kind"].as_str().unwrap(),
                Span::call_site(),
            );
            let enumerants = &member["enumerants"];
            if enumerants
                .members()
                .any(|enumerant| !enumerant["parameters"].is_null())
            {
                let mut variants = vec![];
                let mut display = vec![];
                let mut parse = vec![];
                let mut needs_lifetime = false;
                for enumerant in enumerants.members() {
                    let value = enumerant["value"].as_u32().unwrap();
                    let params = &enumerant["parameters"];
                    if !params.is_empty() {
                        let mut fields = vec![];
                        let mut field_names = vec![];
                        let mut field_tys = vec![];
                        for param in params.members() {
                            let ty = param["kind"].as_str().unwrap();
                            let mut lifetime = None;
                            if ty == "LiteralString" {
                                needs_lifetime = true;
                                lifetime = Some(quote! { <'a> });
                            }
                            let ty_ident = Ident::new(ty, Span::call_site());
                            let ty_path = quote! { #ty_ident };
                            let name = field_name(param["name"]
                                .as_str()
                                .unwrap_or(ty)
                            );
                            fields.push(quote! {
                                #name: #ty_path #lifetime,
                            });
                            field_names.push(quote! {#name});
                            field_tys.push(ty_path);
                        }
                        let name = Ident::new(
                            enumerant["enumerant"].as_str().unwrap(),
                            Span::call_site()
                        );
                        variants.push(quote! {
                            #name {
                                #(#fields)*
                            },
                        });
                        display.push(quote! {
                            Self::#name {
                                #(#field_names),*
                            } => {
                                write!(f, "{}", stringify!(#name))?;
                                #(
                                    write!(f, " {}", #field_names)?;
                                )*
                                Ok(())
                            },
                        });
                        parse.push(quote! {
                            #value => Ok(Self::#name {#(
                                #field_names: #field_tys::parse_one(stream)?,
                            )*}),
                        });
                    } else {
                        let name = Ident::new(
                            enumerant["enumerant"].as_str().unwrap(),
                            Span::call_site()
                        );
                        variants.push(quote! {
                            #name,
                        });
                        display.push(quote! {
                            Self::#name => write!(f, "{}", stringify!(#name)),
                        });
                        parse.push(quote! {
                            #value => Ok(Self::#name),
                        });
                    }
                }
                let lifetime = needs_lifetime
                    .then(|| quote! { <'a> });
                let def = quote! {
                    #enum_derive
                    pub enum #kind #lifetime {
                        #(#variants)*
                    }
                };
                let parse = quote! {
                    impl<'a> #kind #lifetime {

                        #[inline]
                        pub fn parse_one(
                            stream: &mut InstructionStream<'a>,
                        ) -> ParseResult<Self> {
                            let variant = stream.read()?;
                            match variant {
                                #(#parse)*
                                x => Err(ParseError::UnknownVariant {
                                    kind: stringify!(#kind),
                                    value: x,
                                })
                            }
                        }
                   }
                };
                let display = quote! {
                    impl #lifetime Display for #kind #lifetime {

                        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                            match self {
                                #(#display)*
                            }
                        }
                    }
                };
                write!(enums, "{def}\n{parse}\n{display}\n")?;
            } else {
                let def = quote! {
                    #struct_derive
                    pub struct #kind(pub(crate) u32);
                };
                let enumerants: Vec<_> = enumerants
                    .members()
                    .map(|enumerant| {
                        let name = enumerant["enumerant"].as_str().unwrap();
                        let snake_case = upper_snake_case(name);
                        let value = enumerant["value"].as_u32().unwrap();
                        (
                            quote! {
                                pub const #snake_case: Self = Self(#value);
                            },
                            quote! {
                                Self::#snake_case => write!(f, #name),
                            },
                        )
                    }).collect();
                let inherent = enumerants
                    .iter()
                    .map(|(def, _)| def);
                let inherent = quote! {
                    impl #kind {
                        #(#inherent)*
                    }
                };
                let word = quote! {

                    impl Word for #kind {

                        #[inline]
                        fn from_word(word: u32) -> Self {
                            Self(word)
                        }
                    }
                };
                let display = enumerants
                    .iter()
                    .map(|(_, display)| display);
                let display = quote! {
                    impl Display for #kind {

                        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                            match *self {
                                #(#display)*
                                x => write!(f, "{x}"),
                            }
                        }
                    }
                };
                write!(enums, "{def}\n{inherent}\n{word}\n{display}\n")?;
            }
        }
    }
    enums.flush()?;
    Ok(path)
}

fn definition_gen(
    op_path: &Path,
    parsed: &json::JsonValue,
) -> io::Result<PathBuf>
{
    let path = op_path.with_file_name("op/definitions.rs");
    let mut definitions = File::create(&path)?;
    definitions.flush()?;
    let header = quote! {
        //! SPIR-V instruction definitions
        //!
        //! This file is auto-generated, do not modify manually.
        use core::{
            fmt::{Display, self},
        };
        use crate::{
            core::*,
            op::*,
        };
    };
    writeln!(definitions, "{header}")?;
    #[derive(Clone, Copy)]
    struct Operand<'a> {
        needs_lifetime: bool,
        parse_with_ctx: bool,
        kind: &'a str,
        quantifier: Option<char>,
        name: Option<&'a str>,
        duplicate: Option<u32>,
    }
    struct Instruction<'a> {
        name: &'a str,
        code: u16,
        code_idx: usize,
        operands: Vec<Operand<'a>>,
    }
    struct Class<'a> {
        no_result: Vec<Instruction<'a>>,
        with_result: Vec<Instruction<'a>>,
    }
    let mut instruction_classes = HashMap::<&str, Class>::new();
    let mut operand_dup = HashMap::new();
    let lifetime_ty: HashSet<_> =
        ["LiteralString", "LiteralSpecConstantOpInteger", "Decoration"]
        .into_iter().collect();
    let parse_with_ctx: HashSet<_> =
        ["LiteralContextDependentNumber"]
        .into_iter().collect();
    let mut opcodes = vec![];
    let mut opcode_display = vec![];
    for member in parsed["instructions"].members() {
        let opname = member["opname"].as_str().unwrap();
        let class = member["class"].as_str().unwrap();
        if class == "@exclude" {
            continue
        }
        let opcode = member["opcode"].as_u16().unwrap();
        let lit_opcode = proc_macro2::Literal::u16_unsuffixed(
            opcode,
        );
        operand_dup.clear();
        let mut operands = vec![];
        let mut has_result = false;
        for operand in member["operands"].members() {
            let mut operand = Operand {
                needs_lifetime: false,
                parse_with_ctx: false,
                kind: operand["kind"].as_str().unwrap(),
                quantifier: operand["quantifier"].as_str().map(|str| str.chars().next().unwrap()),
                name: operand["name"].as_str(),
                duplicate: None,
            };
            operand.parse_with_ctx = parse_with_ctx.contains(operand.kind);
            if lifetime_ty.contains(operand.kind)
            {
                operand.needs_lifetime = true;
            } else if operand.name.is_none() && operand.kind == "IdResult"
            {
                has_result = true;
            }
            operand_dup
                .entry(operand.name.unwrap_or(operand.kind))
                .and_modify(|num| *num += 1)
                .or_insert(1);
            operands.push(operand);
        }
        operand_dup.retain(|_, dup| *dup > 1);
        for &name in operand_dup.keys() {
            let mut dup = 1;
            for operand in operands
                .iter_mut()
                .filter(|operand| operand.name.unwrap_or(operand.kind) == name)
            {
                operand.duplicate = Some(dup);
                dup += 1;
            }
        }
        let upper_sc_name = upper_snake_case(opname.trim_start_matches("Op"));
        let code_idx = opcodes.len();
        opcodes.push(quote! {
            pub const #upper_sc_name: Self = Self(#lit_opcode);
        });
        opcode_display.push(quote! {
            Self::#upper_sc_name => write!(f, #opname),
        });
        instruction_classes
            .entry(class)
            .and_modify(|class| {
                let instruction = Instruction {
                    name: opname,
                    code: opcode,
                    code_idx,
                    operands: operands.clone(),
                };
                if has_result {
                    class.with_result.push(instruction);
                } else {
                    class.no_result.push(instruction);
                }
            }).or_insert_with(|| {
                let mut class = Class {
                    no_result: vec![],
                    with_result: vec![],
                };
                let instruction = Instruction {
                    name: opname,
                    code: opcode,
                    code_idx,
                    operands,
                };
                if has_result {
                    class.with_result.push(instruction);
                } else {
                    class.no_result.push(instruction);
                }
                class
            });
    }
    let mut inst_infos = vec![];
    for class in instruction_classes.values_mut() {
        for instruction in &class.with_result {
            let name = Ident::new(
                instruction.name.trim_start_matches("Op"),
                Span::call_site()
            );
            let mut fields = vec![];
            let mut has_id_result_type = false;
            let mut needs_lifetime = false;
            for operand in &instruction.operands {
                let name = if let Some(name) = operand.name {
                    if let Some(dup) = operand.duplicate {
                        field_name(&format!("{name}_{dup}"))
                    } else {
                        field_name(name)
                    }
                } else {
                    if let Some(dup) = operand.duplicate {
                        field_name(&format!("{}_{dup}", operand.kind))
                    } else {
                        field_name(operand.kind)
                    }
                };
                has_id_result_type |= name == "id_result_type";
                let kind = Ident::new(operand.kind, Span::call_site());
                let mut operand_needs_lifetime = operand.needs_lifetime;
                match operand.quantifier {
                    None => {
                        let mut field = format!("{name}: {kind}");
                        if operand_needs_lifetime {
                            field += "<'a>";
                        }
                        field += ",";
                        fields.push(field);
                    },
                    Some('?') => {
                        let mut field = format!("{name}: Option<{kind}");
                        if operand_needs_lifetime {
                            field += "<'a>";
                        }
                        field += ">,";
                        fields.push(field);
                    },
                    Some('*') => {
                        let mut field = format!("{name}: &'a [{kind}");
                        if operand_needs_lifetime {
                            field += "<'a>";
                        }
                        field += "],";
                        fields.push(field);
                        operand_needs_lifetime = true;
                    },
                    _ => unreachable!(),
                }
                if operand_needs_lifetime {
                    needs_lifetime = true;
                }
            }
            let mut struct_def_lines = vec![];
            let mut first_line = format!("struct {name}");
            if needs_lifetime {
                first_line += "<'a>"
            }
            first_line += " {";
            struct_def_lines.push(first_line);
            for field in fields {
                struct_def_lines.push(format!("  {field}"));
            }
            struct_def_lines.push("}".to_string());
            let code = instruction.code;
            if code as usize >= inst_infos.len() {
                inst_infos.resize(code as usize + 1, ("", false, false));
            }
            inst_infos[code as usize] = (instruction.name, has_id_result_type, true);
            let opcode = &mut opcodes[instruction.code_idx];
            let docs = struct_def_lines
                .iter()
                .map(|line| {
                    quote! {
                        #[doc = #line]
                    }
                });
            *opcode = quote! {
                #[doc = "# Struct template"]
                #[doc = "``` rust"]
                #(#docs)*
                #[doc = "```"]
                #opcode
            };
        }
        for instruction in &class.no_result {
            let name = Ident::new(
                instruction.name.trim_start_matches("Op"),
                Span::call_site()
            );
            let mut fields = vec![];
            let mut needs_lifetime = false;
            for operand in &instruction.operands {
                let name = if let Some(name) = operand.name {
                    if let Some(dup) = operand.duplicate {
                        field_name(&format!("{name}_{dup}"))
                    } else {
                        field_name(name)
                    }
                } else {
                    if let Some(dup) = operand.duplicate {
                        field_name(&format!("{}_{dup}", operand.kind))
                    } else {
                        field_name(operand.kind)
                    }
                };
                let kind = Ident::new(operand.kind, Span::call_site());
                let mut operand_needs_lifetime = operand.needs_lifetime; 
                match operand.quantifier {
                    None => {
                        let mut field = format!("{name}: {kind}");
                        if operand_needs_lifetime {
                            field += "<'a>";
                        }
                        field += ",";
                        fields.push(field);
                    },
                    Some('?') => {
                        let mut field = format!("{name}: Option<{kind}");
                        if operand_needs_lifetime {
                            field += "<'a>";
                        }
                        field += ">,";
                        fields.push(field);
                    },
                    Some('*') => {
                        let mut field = format!("{name}: &'a [{kind}");
                        if operand_needs_lifetime {
                            field += "<'a>";
                        }
                        field += "],";
                        fields.push(field);
                        operand_needs_lifetime = true;
                    },
                    _ => unreachable!(),
                }
                if operand_needs_lifetime {
                    needs_lifetime = true;
                }
            }
            let mut struct_def_lines = vec![];
            if !fields.is_empty() {
                let mut first_line = format!("struct {name}");
                if needs_lifetime {
                    first_line += "<'a>"
                }
                first_line += " {";
                struct_def_lines.push(first_line);
                for field in fields {
                    struct_def_lines.push(format!("  {field}"));
                }
                struct_def_lines.push("}".to_string());
            } else {
                struct_def_lines.push(format!("struct {name};"));
            }
            let code = instruction.code;
            if code as usize >= inst_infos.len() {
                inst_infos.resize(code as usize + 1, ("", false, false));
            }
            inst_infos[code as usize] = (instruction.name, false, false);
            let opcode = &mut opcodes[instruction.code_idx];
            let docs = struct_def_lines
                .iter()
                .map(|line| {
                    quote! {
                        #[doc = #line]
                    }
                });
            *opcode = quote! {
                #[doc = "# Struct template"]
                #[doc = "``` rust"]
                #(#docs)*
                #[doc = "```"]
                #opcode
            };
        }
    }
    let len = inst_infos.len();
    let inst_infos = inst_infos
        .iter()
        .map(|(name, has_id_result_type, has_id_result)| {
            quote! {
                InstInfo {
                    name: #name,
                    has_id_result_type: #has_id_result_type,
                    has_id_result: #has_id_result,
                }
            }
        });
    let inst_infos = quote! {
        use super::*;
        pub(crate) static INST_INFOS: [InstInfo; #len] = [
            #(#inst_infos),*
        ];
    };
    let opcode_def = quote! {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct Code(pub(crate) u16);
        impl Code {
            #(#opcodes)*
        }
        impl Word for Code {

            #[inline]
            fn from_word(word: u32) -> Self {
                Self(word as u16)
            }
        }
    };
    let opcode_display = quote! {

        impl Display for Code {

            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", INST_INFOS[self.0 as usize].name)
            }
        }
    };
    write!(definitions, "{opcode_def}\n{opcode_display}\n")?;
    write!(File::create(op_path.with_file_name("op/inst_info.rs"))?, "{inst_infos}")?;
    Ok(path)
}

fn main() {
    let grammar = include_str!("../spirv.core.grammar.json");
    let parsed = json::parse(grammar).unwrap();
    let path = Path::new("../../src/op/");
    std::fs::DirBuilder
        ::new()
        .recursive(true)
        .create(path).expect("failed to create directory");
    let enums = enum_gen(path, &parsed).expect("io error");
    let definitions = definition_gen(path, &parsed).expect("io error");
    Command::new("rustfmt")
        .arg(enums)
        .arg(definitions)
        .output().expect("rustfmt failed");
}
