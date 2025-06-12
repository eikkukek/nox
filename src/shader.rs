use crate::{
    allocator_traits::{Allocate, Free},
    renderer::Renderer,
    string_types::{array_format, DynString, LargeError}
};

use memmap2::Mmap;

use std::{
    fs::File, io::Write
};

type Stage = shaderc::ShaderKind;

pub struct Shader<'alloc, Alloc>
    where
        Alloc: Allocate + Free
{
    input_name: DynString<'alloc, Alloc>,
}

fn infer_stage(input_name: &str) -> Stage {
    if input_name.ends_with(".vert") { Stage::Vertex }
    else if input_name.ends_with(".frag") { Stage::Fragment }
    else if input_name.ends_with(".comp") { Stage::Compute }
    else { Stage::InferFromSource }
}

impl<'alloc, Alloc> Shader<'alloc, Alloc>
    where
        Alloc: Allocate + Free
{

    pub fn new(
        input_name: String,
        renderer: &Renderer,
    ) -> Result<Self, LargeError> {
        let ifs = File
            ::open(&input_name)
            .map_err(|e|
                array_format!("failed to open input file {} ( {} )", &input_name, e)
            )?;
        let input = unsafe { Mmap
            ::map(&ifs)
            .map_err(|e|
                array_format!("failed to map input file {} ( {} )", &input_name, e)
            )?
        };
        let source = core::str
            ::from_utf8(&input)
            .map_err(|e|
                array_format!("failed to view input file {} ( {} )", &input_name, e)
            )?;
        let compiler = shaderc::Compiler
            ::new()
            .map_err(|e|
                array_format!("failed to create shaderc compiler ( {} )", e)
            )?;
        let mut options = shaderc::CompileOptions::new().unwrap();
        let physical_device_info = renderer.device_info();
        options.set_target_env(
            shaderc::TargetEnv::Vulkan,
            physical_device_info.api_version().as_u32()
        );
        options.set_source_language(shaderc::SourceLanguage::GLSL);
        options.set_optimization_level(shaderc::OptimizationLevel::Performance);
        let spirv = compiler
            .compile_into_spirv(
                source,
                infer_stage(&input_name),
                input_name.as_str(),
                "main",
                Some(&options))
            .map_err(|e|
                array_format!("failed to compile {} ( {} )", &input_name, e)
            )?;
        let mut ofs = File
            ::open(&output_name)
            .map_err(|e|
                array_format!("failed to open output file {} ( {} )", &output_name, e)
            )?;
        ofs.write(spirv.as_binary_u8())
            .map_err(|e|
                array_format!("failed to write to output file {} ( {} )", &output_name, e)
            )?;
        Ok(Self {
            input_name,
            output_name,
        })
    }
}
