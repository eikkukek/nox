use bindgen::{builder, BindgenError};

fn main() -> Result<(), BindgenError> {
    let bindings = builder()
        .header("vk_video/vulkan_video_codec_av1std_decode.h")
        .header("vk_video/vulkan_video_codec_h265std_decode.h")
        .header("vk_video/vulkan_video_codec_av1std_encode.h")
        .header("vk_video/vulkan_video_codec_h265std_encode.h")
        .header("vk_video/vulkan_video_codec_av1std.h")
        .header("vk_video/vulkan_video_codec_h265std.h")
        .header("vk_video/vulkan_video_codec_h264std_decode.h")
        .header("vk_video/vulkan_video_codecs_common.h")
        .header("vk_video/vulkan_video_codec_h264std_encode.h")
        .header("vk_video/vulkan_video_codec_vp9std_decode.h")
        .header("vk_video/vulkan_video_codec_h264std.h")
        .header("vk_video/vulkan_video_codec_vp9std.h")
        .generate()?;
    bindings.write_to_file("output.rs").unwrap();
    Ok(())
}
