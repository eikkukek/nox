use crate::Version;

#[inline(always)]
pub fn glsl_to_spirv(
    src: &str,
    input_name: &str,
    shader_kind: shaderc::ShaderKind,
    vulkan_version: Version,
) -> Result<shaderc::CompilationArtifact, shaderc::Error>
{
    let compiler = shaderc::Compiler::new()?;
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_target_env(
        shaderc::TargetEnv::Vulkan,
        vulkan_version.as_u32(),
    );
    options.set_source_language(shaderc::SourceLanguage::GLSL);
    options.set_optimization_level(shaderc::OptimizationLevel::Performance);
    Ok(compiler.compile_into_spirv(
        src,
        shader_kind,
        input_name,
        "main",
        Some(&options)
    )?)
}
