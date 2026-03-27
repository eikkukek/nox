use core::{
    ptr,
    ffi::{CStr, c_void},
};

use nox_ash::{
    load_fn, prelude::VkResult, vk,
    khr,
};

use nox_mem::{
    slice::AllocSlice,
    option::{OptionExt, OptionSlice},
    result::ResultExt,
};

use super::PhysicalDevice;

/// Raw Vulkan 1.0 device-level function pointers.
pub type DeviceFnV1_0 = nox_ash::DeviceFnV1_0;

/// Raw Vulkan 1.1 device-level function pointers.
pub type DeviceFnV1_1 = nox_ash::DeviceFnV1_1;

/// Raw Vulkan 1.2 device-level function pointers guaranteed to be supported by Nox.
///
/// # Pre-1.2 extension dependencies
/// - VK_KHR_timeline_semaphore
#[derive(Clone)]
pub struct DeviceFnV1_2 {
    /// VK_KHR_timeline_semaphore
    pub wait_semaphores: vk::PFN_vkWaitSemaphores,
    /// VK_KHR_timeline_semaphore
    pub get_semaphore_counter_value: vk::PFN_vkGetSemaphoreCounterValue,
    /// VK_KHR_timeline_semaphore
    pub signal_semaphore: vk::PFN_vkSignalSemaphore,
}

unsafe impl Send for DeviceFnV1_2 {}
unsafe impl Sync for DeviceFnV1_2 {}

impl DeviceFnV1_2 {

    pub fn load<F: FnMut(&CStr) -> *const c_void>(
        api_version: u32,
        mut f: F,
    ) -> Self {
        if api_version >= vk::API_VERSION_1_2 {
            unsafe { Self {
                wait_semaphores: load_fn!(
                    fn wait_semaphores(vk::Device, *const vk::SemaphoreWaitInfo, u64) -> vk::Result,
                    f,
                    c"vkWaitSemaphores",
                    vk::PFN_vkWaitSemaphores,
                ),
                get_semaphore_counter_value: load_fn!(
                    fn get_semaphore_counter_value(
                        vk::Device,
                        vk::Semaphore,
                        *mut u64,
                    ) -> vk::Result,
                    f,
                    c"vkGetSemaphoreCounterValue",
                    vk::PFN_vkGetSemaphoreCounterValue,
                ),
                signal_semaphore: load_fn!(
                    fn signal_semaphore(
                        vk::Device,
                        *const vk::SemaphoreSignalInfo<'_>
                    ) -> vk::Result,
                    f,
                    c"vkSignalSemaphore",
                    vk::PFN_vkSignalSemaphore,
                ),
            } }
        } else {
            unsafe { Self {
                wait_semaphores: load_fn!(
                    fn wait_semaphores(vk::Device, *const vk::SemaphoreWaitInfo, u64) -> vk::Result,
                    f,
                    c"vkWaitSemaphoresKHR",
                    vk::PFN_vkWaitSemaphores,
                ),
                get_semaphore_counter_value: load_fn!(
                    fn get_semaphore_counter_value(
                        vk::Device,
                        vk::Semaphore,
                        *mut u64,
                    ) -> vk::Result,
                    f,
                    c"vkGetSemaphoreCounterValueKHR",
                    vk::PFN_vkGetSemaphoreCounterValue,
                ),
                signal_semaphore: load_fn!(
                    fn signal_semaphore(
                        vk::Device,
                        *const vk::SemaphoreSignalInfo<'_>
                    ) -> vk::Result,
                    f,
                    c"vkSignalSemaphoreKHR",
                    vk::PFN_vkSignalSemaphore,
                ),
            } }
        }
    }
}

/// Raw Vulkan 1.3 device-level function pointers supported by Nox.
///
/// # Pre-1.3 extension dependencies
/// - VK_KHR_dynamic_rendering
/// - VK_EXT_extended_dynamic_state
/// - VK_KHR_copy_commands2
/// - VK_KHR_synchronization2
/// - VK_KHR_maintenance4
#[derive(Clone)]
pub struct DeviceFnV1_3 {
   
    /// VK_KHR_dynamic_rendering
    pub cmd_begin_rendering: vk::PFN_vkCmdBeginRendering,
    /// VK_KHR_dynamic_rendering
    pub cmd_end_rendering: vk::PFN_vkCmdEndRendering,

    /// VK_EXT_extended_dynamic_state
    pub cmd_bind_vertex_buffers2: vk::PFN_vkCmdBindVertexBuffers2,
    /// VK_EXT_extended_dynamic_state
    pub cmd_set_cull_mode: vk::PFN_vkCmdSetCullMode,
    /// VK_EXT_extended_dynamic_state
    pub cmd_set_depth_bounds_test_enable: vk::PFN_vkCmdSetDepthBoundsTestEnable,
    /// VK_EXT_extended_dynamic_state
    pub cmd_set_depth_compare_op: vk::PFN_vkCmdSetDepthCompareOp,
    /// VK_EXT_extended_dynamic_state
    pub cmd_set_depth_test_enable: vk::PFN_vkCmdSetDepthTestEnable,
    /// VK_EXT_extended_dynamic_state
    pub cmd_set_depth_write_enable: vk::PFN_vkCmdSetDepthWriteEnable,
    /// VK_EXT_extended_dynamic_state
    pub cmd_set_front_face: vk::PFN_vkCmdSetFrontFace,
    /// VK_EXT_extended_dynamic_state
    pub cmd_set_primitive_topology: vk::PFN_vkCmdSetPrimitiveTopology,
    /// VK_EXT_extended_dynamic_state
    pub cmd_set_scissor_with_count: vk::PFN_vkCmdSetScissorWithCount,
    /// VK_EXT_extended_dynamic_state
    pub cmd_set_stencil_op: vk::PFN_vkCmdSetStencilOp,
    /// VK_EXT_extended_dynamic_state
    pub cmd_set_stencil_test_enable: vk::PFN_vkCmdSetStencilTestEnable,
    /// VK_EXT_extended_dynamic_state
    pub cmd_set_viewport_with_count: vk::PFN_vkCmdSetViewportWithCount,

    /// VK_EXT_extended_dynamic_state2
    pub cmd_set_rasterizer_discard_enable: vk::PFN_vkCmdSetRasterizerDiscardEnable,
    /// VK_EXT_extended_dynamic_state2
    pub cmd_set_depth_bias_enable: vk::PFN_vkCmdSetDepthBiasEnable,
    /// VK_EXT_extended_dynamic_state2
    pub cmd_set_primitive_restart_enable: vk::PFN_vkCmdSetPrimitiveRestartEnable,

    /// VK_KHR_copy_commands2
    pub cmd_blit_image2: vk::PFN_vkCmdBlitImage2,
    /// VK_KHR_copy_commands2
    pub cmd_copy_buffer2: vk::PFN_vkCmdCopyBuffer2,
    /// VK_KHR_copy_commands2
    pub cmd_copy_buffer_to_image2: vk::PFN_vkCmdCopyBufferToImage2,
    /// VK_KHR_copy_commands2
    pub cmd_copy_image2: vk::PFN_vkCmdCopyImage2,
    /// VK_KHR_copy_commands2
    pub cmd_copy_image_to_buffer2: vk::PFN_vkCmdCopyImageToBuffer2,
    /// VK_KHR_copy_commands2
    pub cmd_resolve_image2: vk::PFN_vkCmdResolveImage2,

    /// VK_KHR_synchronization2
    pub cmd_wait_events2: vk::PFN_vkCmdWaitEvents2,
    /// VK_KHR_synchronization2
    pub cmd_pipeline_barrier2: vk::PFN_vkCmdPipelineBarrier2,
    /// VK_KHR_synchronization2
    pub queue_submit2: vk::PFN_vkQueueSubmit2,

    /// VK_KHR_maintenance4
    pub get_device_buffer_memory_requirements: vk::PFN_vkGetDeviceBufferMemoryRequirements,
    /// VK_KHR_maintenance4
    pub get_device_image_memory_requirements: vk::PFN_vkGetDeviceImageMemoryRequirements,
    /// VK_KHR_maintenance4
    pub get_device_image_sparse_memory_requirements: vk::PFN_vkGetDeviceImageSparseMemoryRequirements,
}

impl DeviceFnV1_3 {

    pub fn load<F: FnMut(&CStr) -> *const c_void>(
        api_version: u32,
        mut f: F,
    ) -> Self {
        if api_version >= vk::API_VERSION_1_3 {
            unsafe { Self {
                cmd_begin_rendering: load_fn!(
                    fn cmd_begin_rendering(vk::CommandBuffer, *const vk::RenderingInfo) -> (),
                    f,
                    c"vkCmdBeginRendering",
                    vk::PFN_vkCmdBeginRendering,
                ),
                cmd_end_rendering: load_fn!(
                    fn cmd_end_rendering(vk::CommandBuffer) -> (),
                    f,
                    c"vkCmdEndRendering",
                    vk::PFN_vkCmdEndRendering,
                ),
                cmd_bind_vertex_buffers2: load_fn!(
                    fn cmd_bind_vertex_buffers2(
                        vk::CommandBuffer,
                        u32,
                        u32,
                        *const vk::Buffer,
                        *const vk::DeviceSize,
                        *const vk::DeviceSize,
                        *const vk::DeviceSize,
                    ) -> (),
                    f,
                    c"vkCmdBindVertexBuffers2",
                    vk::PFN_vkCmdBindVertexBuffers2,
                ),
                cmd_set_cull_mode: load_fn!(
                    fn cmd_set_cull_mode(vk::CommandBuffer, vk::CullModeFlags) -> (),
                    f,
                    c"vkCmdSetCullMode",
                    vk::PFN_vkCmdSetCullMode,
                ),
                cmd_set_depth_bounds_test_enable: load_fn!(
                    fn cmd_set_depth_bounds_test_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetDepthBoundsTestEnable",
                    vk::PFN_vkCmdSetDepthBoundsTestEnable,
                ),
                cmd_set_depth_compare_op: load_fn!(
                    fn cmd_set_depth_compare_op(vk::CommandBuffer, vk::CompareOp) -> (),
                    f,
                    c"vkCmdSetDepthCompareOp",
                    vk::PFN_vkCmdSetDepthCompareOp,
                ),
                cmd_set_depth_test_enable: load_fn!(
                    fn cmd_set_depth_test_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetDepthTestEnable",
                    vk::PFN_vkCmdSetDepthTestEnable,
                ),
                cmd_set_depth_write_enable: load_fn!(
                    fn cmd_set_depth_write_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetDepthWriteEnable",
                    vk::PFN_vkCmdSetDepthWriteEnable,
                ),
                cmd_set_front_face: load_fn!(
                    fn cmd_set_front_face(vk::CommandBuffer, vk::FrontFace) -> (),
                    f,
                    c"vkCmdSetFrontFace",
                    vk::PFN_vkCmdSetFrontFace,
                ),
                cmd_set_primitive_topology: load_fn!(
                    fn cmd_set_primitive_topology(vk::CommandBuffer, vk::PrimitiveTopology) -> (),
                    f,
                    c"vkCmdSetPrimitiveTopology",
                    vk::PFN_vkCmdSetPrimitiveTopology,
                ),
                cmd_set_scissor_with_count: load_fn!(
                    fn cmd_set_scissor_with_count(vk::CommandBuffer, u32, *const vk::Rect2D) -> (),
                    f,
                    c"vkCmdSetScissorWithCount",
                    vk::PFN_vkCmdSetScissorWithCount,
                ),
                cmd_set_stencil_op: load_fn!(
                    fn cmd_set_stencil_op(
                        vk::CommandBuffer,
                        vk::StencilFaceFlags,
                        vk::StencilOp,
                        vk::StencilOp,
                        vk::StencilOp,
                        vk::CompareOp,
                    ) -> (),
                    f,
                    c"vkCmdSetStencilOp",
                    vk::PFN_vkCmdSetStencilOp,
                ),
                cmd_set_stencil_test_enable: load_fn!(
                    fn cmd_set_stencil_test_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetStencilTestEnable",
                    vk::PFN_vkCmdSetStencilTestEnable,
                ),
                cmd_set_viewport_with_count: load_fn!(
                    fn cmd_set_viewport_with_count(vk::CommandBuffer, u32, *const vk::Viewport) -> (),
                    f,
                    c"vkCmdSetViewportWithCount",
                    vk::PFN_vkCmdSetViewportWithCount,
                ),
                cmd_set_rasterizer_discard_enable: load_fn!(
                    fn cmd_set_rasterizer_discard_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetRasterizerDiscardEnable",
                    vk::PFN_vkCmdSetRasterizerDiscardEnable,
                ),
                cmd_set_depth_bias_enable: load_fn!(
                    fn cmd_set_depth_bias_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetDepthBiasEnable",
                    vk::PFN_vkCmdSetDepthBiasEnable,
                ),
                cmd_set_primitive_restart_enable: load_fn!(
                    fn cmd_set_primitive_restart_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetPrimitiveRestartEnable",
                    vk::PFN_vkCmdSetPrimitiveRestartEnable,
                ),
                cmd_blit_image2: load_fn!(
                    fn cmd_blit_image2(vk::CommandBuffer, *const vk::BlitImageInfo2) -> (),
                    f,
                    c"vkCmdBlitImage2",
                    vk::PFN_vkCmdBlitImage2,
                ),
                cmd_copy_buffer2: load_fn!(
                    fn cmd_copy_buffer2(vk::CommandBuffer, *const vk::CopyBufferInfo2) -> (),
                    f,
                    c"vkCmdCopyBuffer2",
                    vk::PFN_vkCmdCopyBuffer2,
                ),
                cmd_copy_buffer_to_image2: load_fn!(
                    fn cmd_copy_buffer_to_image2(vk::CommandBuffer, *const vk::CopyBufferToImageInfo2) -> (),
                    f,
                    c"vkCmdCopyBufferToImage2",
                    vk::PFN_vkCmdCopyBufferToImage2,
                ),
                cmd_copy_image2: load_fn!(
                    fn cmd_copy_image2(vk::CommandBuffer, *const vk::CopyImageInfo2) -> (),
                    f,
                    c"vkCmdCopyImage2",
                    vk::PFN_vkCmdCopyImage2,
                ),
                cmd_copy_image_to_buffer2: load_fn!(
                    fn cmd_copy_image_to_buffer2(vk::CommandBuffer, *const vk::CopyImageToBufferInfo2) -> (),
                    f,
                    c"vkCmdCopyImageToBuffer2",
                    vk::PFN_vkCmdCopyImageToBuffer2,
                ),
                cmd_resolve_image2: load_fn!(
                    fn cmd_resolve_image2(vk::CommandBuffer, *const vk::ResolveImageInfo2) -> (),
                    f,
                    c"vkCmdResolveImage2",
                    vk::PFN_vkCmdResolveImage2,
                ),
                cmd_wait_events2: load_fn!(
                    fn cmd_wait_events2(
                        vk::CommandBuffer,
                        u32,
                        *const vk::Event,
                        *const vk::DependencyInfo,
                    ) -> (),
                    f,
                    c"vkCmdWaitEvents2",
                    vk::PFN_vkCmdWaitEvents2,
                ),
                cmd_pipeline_barrier2: load_fn!(
                    fn cmd_pipeline_barrier2(vk::CommandBuffer, *const vk::DependencyInfo) -> (),
                    f,
                    c"vkCmdPipelineBarrier2",
                    vk::PFN_vkCmdPipelineBarrier2,
                ),
                queue_submit2: load_fn!(
                    fn queue_submit2(vk::Queue, u32, *const vk::SubmitInfo2, vk::Fence) -> vk::Result,
                    f,
                    c"vkQueueSubmit2",
                    vk::PFN_vkQueueSubmit2,
                ),
                get_device_buffer_memory_requirements: load_fn!(
                    fn get_device_buffer_memory_requirements(
                        vk::Device,
                        *const vk::DeviceBufferMemoryRequirements,
                        *mut vk::MemoryRequirements2,
                    ) -> (),
                    f,
                    c"vkGetDeviceBufferMemoryRequirements",
                    vk::PFN_vkGetDeviceBufferMemoryRequirements,
                ),
                get_device_image_memory_requirements: load_fn!(
                    fn get_device_image_memory_requirements(
                        vk::Device,
                        *const vk::DeviceImageMemoryRequirements,
                        *mut vk::MemoryRequirements2,
                    ) -> (),
                    f,
                    c"vkGetDeviceImageMemoryRequirements",
                    vk::PFN_vkGetDeviceImageMemoryRequirements,
                ),
                get_device_image_sparse_memory_requirements: load_fn!(
                    fn get_device_image_sparse_memory_requirements(
                        vk::Device,
                        *const vk::DeviceImageMemoryRequirements,
                        *mut u32,
                        *mut vk::SparseImageMemoryRequirements2,
                    ) -> (),
                    f,
                    c"vkGetDeviceImageSparseMemoryRequirements",
                    vk::PFN_vkGetDeviceImageSparseMemoryRequirements,
                ),
            } }
        } else {
            unsafe { Self {
                cmd_begin_rendering: load_fn!(
                    fn cmd_begin_rendering(vk::CommandBuffer, *const vk::RenderingInfo) -> (),
                    f,
                    c"vkCmdBeginRenderingKHR",
                    vk::PFN_vkCmdBeginRendering,
                ),
                cmd_end_rendering: load_fn!(
                    fn cmd_end_rendering(vk::CommandBuffer) -> (),
                    f,
                    c"vkCmdEndRenderingKHR",
                    vk::PFN_vkCmdEndRendering,
                ),
                cmd_bind_vertex_buffers2: load_fn!(
                    fn cmd_bind_vertex_buffers2(
                        vk::CommandBuffer,
                        u32,
                        u32,
                        *const vk::Buffer,
                        *const vk::DeviceSize,
                        *const vk::DeviceSize,
                        *const vk::DeviceSize,
                    ) -> (),
                    f,
                    c"vkCmdBindVertexBuffers2EXT",
                    vk::PFN_vkCmdBindVertexBuffers2,
                ),
                cmd_set_cull_mode: load_fn!(
                    fn cmd_set_cull_mode(vk::CommandBuffer, vk::CullModeFlags) -> (),
                    f,
                    c"vkCmdSetCullModeEXT",
                    vk::PFN_vkCmdSetCullMode,
                ),
                cmd_set_depth_bounds_test_enable: load_fn!(
                    fn cmd_set_depth_bounds_test_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetDepthBoundsTestEnableEXT",
                    vk::PFN_vkCmdSetDepthBoundsTestEnable,
                ),
                cmd_set_depth_compare_op: load_fn!(
                    fn cmd_set_depth_compare_op(vk::CommandBuffer, vk::CompareOp) -> (),
                    f,
                    c"vkCmdSetDepthCompareOpEXT",
                    vk::PFN_vkCmdSetDepthCompareOp,
                ),
                cmd_set_depth_test_enable: load_fn!(
                    fn cmd_set_depth_test_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetDepthTestEnableEXT",
                    vk::PFN_vkCmdSetDepthTestEnable,
                ),
                cmd_set_depth_write_enable: load_fn!(
                    fn cmd_set_depth_write_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetDepthWriteEnableEXT",
                    vk::PFN_vkCmdSetDepthWriteEnable,
                ),
                cmd_set_front_face: load_fn!(
                    fn cmd_set_front_face(vk::CommandBuffer, vk::FrontFace) -> (),
                    f,
                    c"vkCmdSetFrontFaceEXT",
                    vk::PFN_vkCmdSetFrontFace,
                ),
                cmd_set_primitive_topology: load_fn!(
                    fn cmd_set_primitive_topology(vk::CommandBuffer, vk::PrimitiveTopology) -> (),
                    f,
                    c"vkCmdSetPrimitiveTopologyEXT",
                    vk::PFN_vkCmdSetPrimitiveTopology,
                ),
                cmd_set_scissor_with_count: load_fn!(
                    fn cmd_set_scissor_with_count(vk::CommandBuffer, u32, *const vk::Rect2D) -> (),
                    f,
                    c"vkCmdSetScissorWithCountEXT",
                    vk::PFN_vkCmdSetScissorWithCount,
                ),
                cmd_set_stencil_op: load_fn!(
                    fn cmd_set_stencil_op(
                        vk::CommandBuffer,
                        vk::StencilFaceFlags,
                        vk::StencilOp,
                        vk::StencilOp,
                        vk::StencilOp,
                        vk::CompareOp,
                    ) -> (),
                    f,
                    c"vkCmdSetStencilOpEXT",
                    vk::PFN_vkCmdSetStencilOp,
                ),
                cmd_set_stencil_test_enable: load_fn!(
                    fn cmd_set_stencil_test_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetStencilTestEnableXT",
                    vk::PFN_vkCmdSetStencilTestEnable,
                ),
                cmd_set_viewport_with_count: load_fn!(
                    fn cmd_set_viewport_with_count(vk::CommandBuffer, u32, *const vk::Viewport) -> (),
                    f,
                    c"vkCmdSetViewportWithCountEXT",
                    vk::PFN_vkCmdSetViewportWithCount,
                ),
                cmd_set_rasterizer_discard_enable: load_fn!(
                    fn cmd_set_rasterizer_discard_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetRasterizerDiscardEnableEXT",
                    vk::PFN_vkCmdSetRasterizerDiscardEnable,
                ),
                cmd_set_depth_bias_enable: load_fn!(
                    fn cmd_set_depth_bias_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetDepthBiasEnableEXT",
                    vk::PFN_vkCmdSetDepthBiasEnable,
                ),
                cmd_set_primitive_restart_enable: load_fn!(
                    fn cmd_set_primitive_restart_enable(vk::CommandBuffer, vk::Bool32) -> (),
                    f,
                    c"vkCmdSetPrimitiveRestartEnableEXT",
                    vk::PFN_vkCmdSetPrimitiveRestartEnable,
                ),
                cmd_blit_image2: load_fn!(
                    fn cmd_blit_image2(vk::CommandBuffer, *const vk::BlitImageInfo2) -> (),
                    f,
                    c"vkCmdBlitImage2KHR",
                    vk::PFN_vkCmdBlitImage2,
                ),
                cmd_copy_buffer2: load_fn!(
                    fn cmd_copy_buffer2(vk::CommandBuffer, *const vk::CopyBufferInfo2) -> (),
                    f,
                    c"vkCmdCopyBuffer2KHR",
                    vk::PFN_vkCmdCopyBuffer2,
                ),
                cmd_copy_buffer_to_image2: load_fn!(
                    fn cmd_copy_buffer_to_image2(vk::CommandBuffer, *const vk::CopyBufferToImageInfo2) -> (),
                    f,
                    c"vkCmdCopyBufferToImage2KHR",
                    vk::PFN_vkCmdCopyBufferToImage2,
                ),
                cmd_copy_image2: load_fn!(
                    fn cmd_copy_image2(vk::CommandBuffer, *const vk::CopyImageInfo2) -> (),
                    f,
                    c"vkCmdCopyImage2KHR",
                    vk::PFN_vkCmdCopyImage2,
                ),
                cmd_copy_image_to_buffer2: load_fn!(
                    fn cmd_copy_image_to_buffer2(vk::CommandBuffer, *const vk::CopyImageToBufferInfo2) -> (),
                    f,
                    c"vkCmdCopyImageToBuffer2KHR",
                    vk::PFN_vkCmdCopyImageToBuffer2,
                ),
                cmd_resolve_image2: load_fn!(
                    fn cmd_resolve_image2(vk::CommandBuffer, *const vk::ResolveImageInfo2) -> (),
                    f,
                    c"vkCmdResolveImage2KHR",
                    vk::PFN_vkCmdResolveImage2,
                ),
                cmd_wait_events2: load_fn!(
                    fn cmd_wait_events2(
                        vk::CommandBuffer,
                        u32,
                        *const vk::Event,
                        *const vk::DependencyInfo,
                    ) -> (),
                    f,
                    c"vkCmdWaitEvents2KHR",
                    vk::PFN_vkCmdWaitEvents2,
                ),
                cmd_pipeline_barrier2: load_fn!(
                    fn cmd_pipeline_barrier2(vk::CommandBuffer, *const vk::DependencyInfo) -> (),
                    f,
                    c"vkCmdPipelineBarrier2KHR",
                    vk::PFN_vkCmdPipelineBarrier2,
                ),
                queue_submit2: load_fn!(
                    fn queue_submit2(vk::Queue, u32, *const vk::SubmitInfo2, vk::Fence) -> vk::Result,
                    f,
                    c"vkQueueSubmit2KHR",
                    vk::PFN_vkQueueSubmit2,
                ),
                get_device_buffer_memory_requirements: load_fn!(
                    fn get_device_buffer_memory_requirements(
                        vk::Device,
                        *const vk::DeviceBufferMemoryRequirements,
                        *mut vk::MemoryRequirements2,
                    ) -> (),
                    f,
                    c"vkGetDeviceBufferMemoryRequirementsKHR",
                    vk::PFN_vkGetDeviceBufferMemoryRequirements,
                ),
                get_device_image_memory_requirements: load_fn!(
                    fn get_device_image_memory_requirements(
                        vk::Device,
                        *const vk::DeviceImageMemoryRequirements,
                        *mut vk::MemoryRequirements2,
                    ) -> (),
                    f,
                    c"vkGetDeviceImageMemoryRequirementsKHR",
                    vk::PFN_vkGetDeviceImageMemoryRequirements,
                ),
                get_device_image_sparse_memory_requirements: load_fn!(
                    fn get_device_image_sparse_memory_requirements(
                        vk::Device,
                        *const vk::DeviceImageMemoryRequirements,
                        *mut u32,
                        *mut vk::SparseImageMemoryRequirements2,
                    ) -> (),
                    f,
                    c"vkGetDeviceImageSparseMemoryRequirementsKHR",
                    vk::PFN_vkGetDeviceImageSparseMemoryRequirements,
                ),
            } }
        }
    }
}

unsafe impl Send for DeviceFnV1_3 {}
unsafe impl Sync for DeviceFnV1_3 {}

/// Raw Vulkan 1.4 device-level function pointers guaranteed to be supported by Nox.
///
/// # Pre-1.4 extension dependencies
/// - VK_KHR_dynamic_rendering_local_read
/// - VK_KHR_maintenance5
/// - VK_KHR_maintenance6
#[derive(Clone)]
pub struct DeviceFnV1_4 {

    /// VK_KHR_dynamic_rendering_local_read
    pub cmd_set_rendering_attachment_locations: vk::PFN_vkCmdSetRenderingAttachmentLocations,
    /// VK_KHR_dynamic_rendering_local_read
    pub cmd_set_rendering_input_attachment_indices: vk::PFN_vkCmdSetRenderingInputAttachmentIndices,

    /// VK_KHR_maintenance5
    pub cmd_bind_index_buffer2: vk::PFN_vkCmdBindIndexBuffer2,

    /// VK_KHR_maintenance6
    pub cmd_bind_descriptor_sets2: vk::PFN_vkCmdBindDescriptorSets2,
    /// VK_KHR_maintenance6
    pub cmd_push_constants2: vk::PFN_vkCmdPushConstants2,
}

impl DeviceFnV1_4 {

    pub fn load<F: FnMut(&CStr) -> *const c_void>(
        api_version: u32,
        mut f: F,
    ) -> Self {
        if api_version >= vk::API_VERSION_1_4 {
            unsafe { Self {
                cmd_set_rendering_attachment_locations: load_fn!(
                    fn cmd_set_rendering_attachment_locations(
                        vk::CommandBuffer,
                        *const vk::RenderingAttachmentLocationInfo,
                    ) -> (),
                    f,
                    c"vkCmdSetRenderingAttachmentLocations",
                    vk::PFN_vkCmdSetRenderingAttachmentLocations,
                ),
                cmd_set_rendering_input_attachment_indices: load_fn!(
                    fn cmd_set_rendering_input_attachment_indices(
                        vk::CommandBuffer,
                        *const vk::RenderingInputAttachmentIndexInfo<'_>,
                    ) -> (),
                    f,
                    c"vkCmdSetRenderingInputAttachmentIndices",
                    vk::PFN_vkCmdSetRenderingInputAttachmentIndices,
                ),
                cmd_bind_index_buffer2: load_fn!(
                    fn cmd_bind_index_buffer2(
                        vk::CommandBuffer,
                        vk::Buffer,
                        vk::DeviceSize,
                        vk::DeviceSize,
                        vk::IndexType,
                    ) -> (),
                    f,
                    c"vkCmdBindIndexBuffer2",
                    vk::PFN_vkCmdBindIndexBuffer2,
                ),
                cmd_bind_descriptor_sets2: load_fn!(
                    fn cmd_bind_descriptor_sets2(
                        vk::CommandBuffer,
                        *const vk::BindDescriptorSetsInfo,
                    ) -> (),
                    f,
                    c"vkCmdBindDescriptorSets2",
                    vk::PFN_vkCmdBindDescriptorSets2,
                ),
                cmd_push_constants2: load_fn!(
                    fn cmd_push_constants2(
                        vk::CommandBuffer,
                        *const vk::PushConstantsInfo,
                    ) -> (),
                    f,
                    c"vkCmdPushConstants2",
                    vk::PFN_vkCmdPushConstants2,
                ),
            } }
        } else {
            unsafe { Self {
                cmd_set_rendering_attachment_locations: load_fn!(
                    fn cmd_set_rendering_attachment_locations(
                        vk::CommandBuffer,
                        *const vk::RenderingAttachmentLocationInfo<'_>,
                    ) -> (),
                    f,
                    c"vkCmdSetRenderingAttachmentLocationsKHR",
                    vk::PFN_vkCmdSetRenderingAttachmentLocations,
                ),
                cmd_set_rendering_input_attachment_indices: load_fn!(
                    fn cmd_set_rendering_input_attachment_indices(
                        vk::CommandBuffer,
                        *const vk::RenderingInputAttachmentIndexInfo<'_>,
                    ) -> (),
                    f,
                    c"vkCmdSetRenderingInputAttachmentIndicesKHR",
                    vk::PFN_vkCmdSetRenderingInputAttachmentIndices,
                ),
                cmd_bind_index_buffer2: load_fn!(
                    fn cmd_bind_index_buffer2(
                        vk::CommandBuffer,
                        vk::Buffer,
                        vk::DeviceSize,
                        vk::DeviceSize,
                        vk::IndexType,
                    ) -> (),
                    f,
                    c"vkCmdBindIndexBuffer2KHR",
                    vk::PFN_vkCmdBindIndexBuffer2,
                ),
                cmd_bind_descriptor_sets2: load_fn!(
                    fn cmd_bind_descriptor_sets2(
                        vk::CommandBuffer,
                        *const vk::BindDescriptorSetsInfo,
                    ) -> (),
                    f,
                    c"vkCmdBindDescriptorSets2KHR",
                    vk::PFN_vkCmdBindDescriptorSets2,
                ),
                cmd_push_constants2: load_fn!(
                    fn cmd_push_constants2(
                        vk::CommandBuffer,
                        *const vk::PushConstantsInfo,
                    ) -> (),
                    f,
                    c"vkCmdPushConstants2KHR",
                    vk::PFN_vkCmdPushConstants2,
                ),
            } }
        }
    }
}

unsafe impl Send for DeviceFnV1_4 {}
unsafe impl Sync for DeviceFnV1_4 {}

/// Raw [`device`][1] level function pointers
///
/// Designed for Nox use only and only exposes functions that are guaranteed to be usabel with Nox.
///
/// [1]: https://docs.vulkan.org/refpages/latest/refpages/source/VkDevice.html
#[derive(Clone)]
pub struct DeviceFunctions {
    device_fn_1_0: DeviceFnV1_0,
    device_fn_1_1: DeviceFnV1_1,
    device_fn_1_2: DeviceFnV1_2,
    device_fn_1_3: DeviceFnV1_3,
    device_fn_1_4: DeviceFnV1_4,
    swapchain_fn: khr::swapchain::DeviceFn,
    present_wait2_fn: khr::present_wait2::DeviceFn,
}

impl DeviceFunctions {

    pub(super) fn new(
        instance: &nox_ash::Instance,
        device: &nox_ash::Device,
        physical_device: &PhysicalDevice,
    ) -> Self {
        let handle = device.handle();
        let api_version = physical_device.api_version().as_u32();
        Self {
            device_fn_1_0: device.fp_v1_0().clone(),
            device_fn_1_1: device.fp_v1_1().clone(),
            device_fn_1_2: DeviceFnV1_2::load(
                api_version, |name| unsafe {
                    core::mem::transmute(instance.get_device_proc_addr(
                        handle, name.as_ptr()
                    ))
                }
            ),
            device_fn_1_3: DeviceFnV1_3::load(
                api_version, |name| unsafe {
                    core::mem::transmute(instance.get_device_proc_addr(
                        handle, name.as_ptr()
                    ))
                }),
            device_fn_1_4: DeviceFnV1_4::load(
                api_version, |name| unsafe {
                    core::mem::transmute(instance.get_device_proc_addr(
                        handle, name.as_ptr()
                    ))
                }),
            swapchain_fn: khr::swapchain::DeviceFn::load(|name| unsafe {
                core::mem::transmute(instance.get_device_proc_addr(
                    handle, name.as_ptr()
                ))
            }),
            present_wait2_fn: khr::present_wait2::DeviceFn::load(|name| unsafe {
                core::mem::transmute(instance.get_device_proc_addr(handle, name.as_ptr()))
            }),
        }
    }

    /// Raw Vulkan 1.0 functions.
    #[inline]
    pub fn fp_v1_0(&self) -> &DeviceFnV1_0 {
        &self.device_fn_1_0
    }

    /// Raw Vulkan 1.1 functions.
    #[inline]
    pub fn fp_v1_1(&self) -> &DeviceFnV1_1 {
        &self.device_fn_1_1
    }

    /// Raw Vulkan 1.2 Nox supported functions.
    #[inline]
    pub fn fp_v1_2(&self) -> &DeviceFnV1_2 {
        &self.device_fn_1_2
    }

    /// Raw Vulkan 1.3 Nox supported functions.
    #[inline]
    pub fn fp_v1_3(&self) -> &DeviceFnV1_3 {
        &self.device_fn_1_3
    }

    /// Raw Vulkan 1.4 Nox supported functions.
    #[inline]
    pub fn fp_v1_4(&self) -> &DeviceFnV1_4 {
        &self.device_fn_1_4
    }

    /// Raw VK_KHR_swapchain functions.
    #[inline]
    pub fn fp_swapchain(&self) -> &khr::swapchain::DeviceFn {
        &self.swapchain_fn
    }

    /// Raw VK_KHR_present_wait2 functions
    #[inline]
    pub fn fp_present_wat2(&self) -> &khr::present_wait2::DeviceFn {
        &self.present_wait2_fn
    }
}

impl super::LogicalDevice { 

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetDeviceQueue.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`get_device_queue2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::get_device_queue2
    #[inline]
    pub unsafe fn get_device_queue(
        &self,
        queue_family_index: u32,
        queue_index: u32,
    ) -> vk::Queue {
        let mut queue = vk::Queue::null();
        unsafe {
            (self.fns().fp_v1_0().get_device_queue)(
                self.handle(),
                queue_family_index,
                queue_index,
                &mut queue,
            );
        }
        queue
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkQueueWaitIdle.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn queue_wait_idle(
        &self,
        queue: vk::Queue,
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().queue_wait_idle)(queue)
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDeviceWaitIdle.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn device_wait_idle(
        &self,
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().device_wait_idle)(self.handle())
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkAllocateMemory.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn allocate_memory(
        &self,
        allocate_info: &vk::MemoryAllocateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::DeviceMemory> {
        let mut memory = vk::DeviceMemory::null();
        unsafe {
            (self.fns().fp_v1_0().allocate_memory)(
                self.handle(),
                allocate_info,
               allocator.as_ptr(),
                &mut memory
            )
        }.result_with_success(memory)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkFreeMemory.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn free_memory(
        &self,
        memory: vk::DeviceMemory,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().free_memory)(
                self.handle(),
                memory,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkMapMemory.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn map_memory(
        &self,
        memory: vk::DeviceMemory,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
        flags: vk::MemoryMapFlags,
    ) -> VkResult<*mut ()> {
        let mut ptr = ptr::null_mut();
        unsafe {
            (self.fns().fp_v1_0().map_memory)(
                self.handle(),
                memory,
                offset,
                size,
                flags,
                &mut ptr,
            )
        }.result_with_success(ptr.cast())
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkUnmapMemory.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn unmap_memory(
        &self,
        memory: vk::DeviceMemory,
    ) {
        unsafe {
            (self.fns().fp_v1_0().unmap_memory)(self.handle(), memory)
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkFlushMappedMemoryRanges.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn flush_mapped_memory_ranges(
        &self,
        memory_ranges: &[vk::MappedMemoryRange],
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().flush_mapped_memory_ranges)(
                self.handle(),
                memory_ranges.len() as u32,
                memory_ranges.as_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkInvalidateMappedMemoryRanges.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn invalidate_mapped_memory_ranges(
        &self,
        memory_ranges: &[vk::MappedMemoryRange<'_>],
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().invalidate_mapped_memory_ranges)(
                self.handle(),
                memory_ranges.len() as u32,
                memory_ranges.as_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetDeviceMemoryCommitment.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn get_device_memory_commitment(
        &self,
        memory: vk::DeviceMemory,
    ) -> vk::DeviceSize {
        let mut committed_memory_in_bytes = 0;
        unsafe {
            (self.fns().fp_v1_0().get_device_memory_commitment)(
                self.handle(),
                memory,
                &mut committed_memory_in_bytes,
            )
        }
        committed_memory_in_bytes
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkBindBufferMemory.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`bind_buffer_memory2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::bind_buffer_memory2
    #[inline]
    pub unsafe fn bind_buffer_memory(
        &self,
        buffer: vk::Buffer,
        memory: vk::DeviceMemory,
        memory_offset: vk::DeviceSize,
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().bind_buffer_memory)(
                self.handle(),
                buffer,
                memory,
                memory_offset,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkBindImageMemory.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`bind_image_memory2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::bind_image_memory2
    #[inline]
    pub unsafe fn bind_image_memory(
        &self,
        image: vk::Image,
        memory: vk::DeviceMemory,
        memory_offset: vk::DeviceSize,
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().bind_image_memory)(
                self.handle(),
                image,
                memory,
                memory_offset,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkQueueBindSparse.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn queue_bind_sparse(
        &self,
        queue: vk::Queue,
        binding_infos: &[vk::BindSparseInfo<'_>],
        fence: vk::Fence,
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().queue_bind_sparse)(
                queue,
                binding_infos.len() as u32,
                binding_infos.as_ptr(),
                fence,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateFence.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_fence(
        &self,
        create_info: &vk::FenceCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::Fence> {
        let mut fence = vk::Fence::null();
        unsafe {
            (self.fns().fp_v1_0().create_fence)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut fence,
            )
        }.result_with_success(fence)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyFence.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_fence(
        &self,
        fence: vk::Fence,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_fence)(
                self.handle(),
                fence,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkResetFences.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn reset_fences(
        &self,
        fences: &[vk::Fence]
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().reset_fences)(
                self.handle(),
                fences.len() as u32,
                fences.as_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetFenceStatus.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # [`Ok`] values
    /// [`vk::Result::NOT_READY`]
    /// [`vk::Result::SUCCESS`]
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn get_fence_status(
        &self,
        fence: vk::Fence,
    ) -> VkResult<vk::Result> {
        unsafe {
            (self.fns().fp_v1_0().get_fence_status)(
                self.handle(),
                fence,
            )
        }.result_with_success(vk::Result::SUCCESS)
        .filter_err(|&err| matches!(
            err,
            vk::Result::NOT_READY
        ).then_some(err))
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkWaitForFences.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # [`Ok`] values
    /// [`vk::Result::SUCCESS`]
    /// [`vk::Result::TIMEOUT`]
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn wait_for_fences(
        &self,
        fences: &[vk::Fence],
        wait_all: bool,
        timeout: u64,
    ) -> VkResult<vk::Result> {
        unsafe {
            (self.fns().fp_v1_0().wait_for_fences)(
                self.handle(),
                fences.len() as u32,
                fences.as_ptr(),
                wait_all as vk::Bool32,
                timeout,
            )
        }.result_with_success(vk::Result::SUCCESS)
        .filter_err(|&err| matches!(err, vk::Result::TIMEOUT).then_some(err))
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateSemaphore.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_semaphore(
        &self,
        create_info: &vk::SemaphoreCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::Semaphore> {
        let mut semaphore = vk::Semaphore::null();
        unsafe {
            (self.fns().fp_v1_0().create_semaphore)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut semaphore
            )
        }.result_with_success(semaphore)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroySemaphore.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_semaphore(
        &self,
        semaphore: vk::Semaphore,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_semaphore)(
                self.handle(),
                semaphore,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateEvent.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_event(
        &self,
        create_info: &vk::EventCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::Event> {
        let mut event = vk::Event::null();
        unsafe {
            (self.fns().fp_v1_0().create_event)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut event
            )
        }.result_with_success(event)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyEvent.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_event(
        &self,
        event: vk::Event,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_event)(
                self.handle(),
                event,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetEventStatus.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # [`Ok`] values
    /// [`vk::Result::EVENT_RESET`]
    /// [`vk::Result::EVENT_SET`]
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn get_event_status(
        &self,
        event: vk::Event,
    ) -> VkResult<vk::Result> {
        unsafe {
            (self.fns().fp_v1_0().get_event_status)(
                self.handle(),
                event,
            )
        }.result_with_success(vk::Result::SUCCESS)
        .filter_err(|&err| matches!(
            err,
            vk::Result::EVENT_RESET |
            vk::Result::EVENT_SET,
        ).then_some(err))
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkSetEvent.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn set_event(
        &self,
        event: vk::Event
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().set_event)(
                self.handle(),
                event,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkResetEvent.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn reset_event(
        &self,
        event: vk::Event,
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().reset_event)(
                self.handle(),
                event,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateQueryPool.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_query_pool(
        &self,
        create_info: &vk::QueryPoolCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::QueryPool> {
        let mut query_pool = vk::QueryPool::null();
        unsafe {
            (self.fns().fp_v1_0().create_query_pool)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut query_pool,
            )
        }.result_with_success(query_pool)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyQueryPool.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_query_pool(
        &self,
        query_pool: vk::QueryPool,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_query_pool)(
                self.handle(),
                query_pool,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetQueryPoolResults.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # [`Ok`] values
    /// [`vk::Result::NOT_READY`]
    /// [`vk::Result::SUCCESS`]
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn get_query_pool_results(
        &self,
        query_pool: vk::QueryPool,
        first_query: u32,
        query_count: u32,
        data_size: usize,
        p_data: *mut (),
        stride: vk::DeviceSize,
        flags: vk::QueryResultFlags,
    ) -> VkResult<vk::Result> {
        unsafe {
            (self.fns().fp_v1_0().get_query_pool_results)(
                self.handle(),
                query_pool,
                first_query,
                query_count,
                data_size,
                p_data.cast(),
                stride,
                flags
            )
        }.result_with_success(vk::Result::SUCCESS)
        .filter_err(|&err| matches!(
            err,
            vk::Result::NOT_READY
        ).then_some(err))
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateBuffer.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_buffer(
        &self,
        create_info: &vk::BufferCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::Buffer> {
        let mut buffer = vk::Buffer::null();
        unsafe {
            (self.fns().fp_v1_0().create_buffer)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut buffer
            )
        }.result_with_success(buffer)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyBuffer.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_buffer(
        &self,
        buffer: vk::Buffer,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_buffer)(
                self.handle(),
                buffer,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateBufferView.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_buffer_view(
        &self,
        create_info: &vk::BufferViewCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::BufferView> {
        let mut buffer_view = vk::BufferView::null();
        unsafe {
            (self.fns().fp_v1_0().create_buffer_view)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut buffer_view,
            )
        }.result_with_success(buffer_view)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyBufferView.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_buffer_view(
        &self,
        buffer_view: vk::BufferView,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_buffer_view)(
                self.handle(),
                buffer_view,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateImage.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_image(
        &self,
        create_info: &vk::ImageCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::Image> {
        let mut image = vk::Image::null();
        unsafe {
            (self.fns().fp_v1_0().create_image)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut image
            )
        }.result_with_success(image)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyImage.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_image(
        &self,
        image: vk::Image,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_image)(
                self.handle(),
                image,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetImageSubresourceLayout.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn get_image_subresource_layout(
        &self,
        image: vk::Image,
        subresource: &vk::ImageSubresource,
    ) -> vk::SubresourceLayout {
        let mut layout = vk::SubresourceLayout::default();
        unsafe {
            (self.fns().fp_v1_0().get_image_subresource_layout)(
                self.handle(),
                image,
                subresource,
                &mut layout,
            )
        }
        layout
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateImageView.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_image_view(
        &self,
        create_info: &vk::ImageViewCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::ImageView> {
        let mut image_view = vk::ImageView::null();
        unsafe {
            (self.fns().fp_v1_0().create_image_view)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut image_view,
            )
        }.result_with_success(image_view)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyImageView.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_image_view(
        &self,
        image_view: vk::ImageView,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_image_view)(
                self.handle(),
                image_view,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateShaderModule.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_shader_module(
        &self,
        create_info: &vk::ShaderModuleCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::ShaderModule> {
        let mut shader_module = vk::ShaderModule::null();
        unsafe {
            (self.fns().fp_v1_0().create_shader_module)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut shader_module,
            )
        }.result_with_success(shader_module)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyShaderModule.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_shader_module(
        &self,
        shader_module: vk::ShaderModule,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_shader_module)(
                self.handle(),
                shader_module,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreatePipelineCache.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_pipeline_cache(
        &self,
        create_info: &vk::PipelineCacheCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::PipelineCache> {
        let mut pipeline_cache = vk::PipelineCache::null();
        unsafe {
            (self.fns().fp_v1_0().create_pipeline_cache)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut pipeline_cache,
            )
        }.result_with_success(pipeline_cache)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyPipelineCache.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_pipeline_cache(
        &self,
        pipeline_cache: vk::PipelineCache,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_pipeline_cache)(
                self.handle(),
                pipeline_cache,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetPipelineCacheData.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// For convenience, this returns a boxed slice over the data since this is unlikely to be called
    /// in a hot loop or per frame.
    ///
    /// If you want to allocate the buffer yourself, use the raw [`get_pipeline_cache_data`][1]
    /// function pointer.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: DeviceFnV1_0::get_pipeline_cache_data
    #[inline]
    pub unsafe fn get_pipeline_cache_data(
        &self,
        pipeline_cache: vk::PipelineCache,
    ) -> VkResult<Box<[u8]>> {
        unsafe {
            let mut data_size = 0;
            (self.fns().fp_v1_0().get_pipeline_cache_data)(
                self.handle(),
                pipeline_cache,
                &mut data_size,
                ptr::null_mut(),
            ).result()?;
            let mut data: Box<[u8]> = Box::uninit_slice(data_size);
            (self.fns().fp_v1_0().get_pipeline_cache_data)(
                self.handle(),
                pipeline_cache,
                &mut data_size,
                data.as_mut_ptr().cast(),
            ).result_with_success(data)
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkMergePipelineCaches.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn merge_pipeline_caches(
        &self,
        dst_cache: vk::PipelineCache,
        src_caches: &mut [vk::PipelineCache],
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().merge_pipeline_caches)(
                self.handle(),
                dst_cache,
                src_caches.len() as u32,
                src_caches.as_mut_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateGraphicsPipelines.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// The lengths of the `create_infos` and `pipelines` *must* match.
    ///
    /// This is *not* checked at runtime.
    ///
    /// *General safety:*
    ///
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_graphics_pipelines(
        &self,
        pipeline_cache: vk::PipelineCache,
        create_infos: &[vk::GraphicsPipelineCreateInfo<'_>],
        allocator: Option<&vk::AllocationCallbacks<'_>>,
        pipelines: &mut [vk::Pipeline],
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().create_graphics_pipelines)(
                self.handle(),
                pipeline_cache,
                create_infos.len() as u32,
                create_infos.as_ptr(),
                allocator.as_ptr(),
                pipelines.as_mut_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateComputePipelines.html>
    ///
    /// Part of Vulkan 1.0 core
    ///
    /// # Safety
    /// The lengths of the `create_infos` and `pipelines` *must* match.
    ///
    /// This is *not* checked at runtime.
    ///
    /// *General safety:*
    ///
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_compute_pipelines(
        &self,
        pipeline_cache: vk::PipelineCache,
        create_infos: &[vk::ComputePipelineCreateInfo<'_>],
        allocator: Option<&vk::AllocationCallbacks<'_>>,
        pipelines: &mut [vk::Pipeline],
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().create_compute_pipelines)(
                self.handle(),
                pipeline_cache,
                create_infos.len() as u32,
                create_infos.as_ptr(),
                allocator.as_ptr(),
                pipelines.as_mut_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyPipeline.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_pipeline(
        &self,
        pipeline: vk::Pipeline,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_pipeline)(
                self.handle(),
                pipeline,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreatePipelineLayout.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_pipeline_layout(
        &self,
        create_info: &vk::PipelineLayoutCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::PipelineLayout> {
        let mut pipeline_layout = vk::PipelineLayout::null();
        unsafe {
            (self.fns().fp_v1_0().create_pipeline_layout)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut pipeline_layout,
            )
        }.result_with_success(pipeline_layout)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyPipelineLayout.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_pipeline_layout(
        &self,
        pipeline_layout: vk::PipelineLayout,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_pipeline_layout)(
                self.handle(),
                pipeline_layout,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateSampler.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_sampler(
        &self,
        create_info: &vk::SamplerCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::Sampler> {
        let mut sampler = vk::Sampler::null();
        unsafe {
            (self.fns().fp_v1_0().create_sampler)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut sampler
            )
        }.result_with_success(sampler)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroySampler.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_sampler(
        &self,
        sampler: vk::Sampler,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_sampler)(
                self.handle(),
                sampler,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateDescriptorSetLayout.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_descriptor_set_layout(
        &self,
        create_info: &vk::DescriptorSetLayoutCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::DescriptorSetLayout> {
        let mut set_layout = vk::DescriptorSetLayout::null();
        unsafe {
            (self.fns().fp_v1_0().create_descriptor_set_layout)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut set_layout,
            )
        }.result_with_success(set_layout)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyDescriptorSetLayout.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_descriptor_set_layout(
        &self,
        descriptor_set_layout: vk::DescriptorSetLayout,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_descriptor_set_layout)(
                self.handle(),
                descriptor_set_layout,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateDescriptorPool.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_descriptor_pool(
        &self,
        create_info: &vk::DescriptorPoolCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::DescriptorPool> {
        let mut descriptor_pool = vk::DescriptorPool::null();
        unsafe {
            (self.fns().fp_v1_0().create_descriptor_pool)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut descriptor_pool,
            )
        }.result_with_success(descriptor_pool)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyDescriptorPool.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_descriptor_pool(
        &self,
        descriptor_pool: vk::DescriptorPool,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_descriptor_pool)(
                self.handle(),
                descriptor_pool,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkResetDescriptorPool.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn reset_descriptor_pool(
        &self,
        descriptor_pool: vk::DescriptorPool,
        flags: vk::DescriptorPoolResetFlags,
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().reset_descriptor_pool)(
                self.handle(),
                descriptor_pool,
                flags,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkAllocateDescriptorSets.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// `descriptor_sets` must hold *at least* `allocate_info.descriptor_set_count` elements.
    ///
    /// This is *not* checked at runtime.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn allocate_descriptor_sets(
        &self,
        allocate_info: &vk::DescriptorSetAllocateInfo<'_>,
        descriptor_sets: &mut [vk::DescriptorSet],
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().allocate_descriptor_sets)(
                self.handle(),
                allocate_info,
                descriptor_sets.as_mut_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkFreeDescriptorSets.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn free_descriptor_sets(
        &self,
        descriptor_pool: vk::DescriptorPool,
        descriptor_sets: &[vk::DescriptorSet],
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().free_descriptor_sets)(
                self.handle(),
                descriptor_pool,
                descriptor_sets.len() as u32,
                descriptor_sets.as_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkUpdateDescriptorSets.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn update_descriptor_sets(
        &self,
        descriptor_writes: &[vk::WriteDescriptorSet],
        descriptor_copies: &[vk::CopyDescriptorSet],
    ) {
        unsafe {
            (self.fns().fp_v1_0().update_descriptor_sets)(
                self.handle(),
                descriptor_writes.len() as u32,
                descriptor_writes.as_ptr(),
                descriptor_copies.len() as u32,
                descriptor_copies.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateCommandPool.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_command_pool(
        &self,
        create_info: &vk::CommandPoolCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::CommandPool> {
        let mut command_pool = vk::CommandPool::null();
        unsafe {
            (self.fns().fp_v1_0().create_command_pool)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut command_pool,
            )
        }.result_with_success(command_pool)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyCommandPool.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_command_pool(
        &self,
        command_pool: vk::CommandPool,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_0().destroy_command_pool)(
                self.handle(),
                command_pool,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkResetCommandPool.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn reset_command_pool(
        &self,
        command_pool: vk::CommandPool,
        flags: vk::CommandPoolResetFlags,
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().reset_command_pool)(
                self.handle(),
                command_pool,
                flags,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkAllocateCommandBuffers.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// `command_buffers` must hold *at least* `allocate_info.command_buffer_count` elements.
    ///
    /// This is *not* checked at runtime.
    ///
    /// *General safety:*
    ///
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn allocate_command_buffers(
        &self,
        allocate_info: &vk::CommandBufferAllocateInfo<'_>,
        command_buffers: &mut [vk::CommandBuffer],
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().allocate_command_buffers)(
                self.handle(),
                allocate_info,
                command_buffers.as_mut_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkFreeCommandBuffers.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn free_command_buffers(
        &self,
        command_pool: vk::CommandPool,
        command_buffers: &[vk::CommandBuffer],
    ) {
        unsafe {
            (self.fns().fp_v1_0().free_command_buffers)(
                self.handle(),
                command_pool,
                command_buffers.len() as u32,
                command_buffers.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkBeginCommandBuffer.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn begin_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        begin_info: &vk::CommandBufferBeginInfo<'_>
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().begin_command_buffer)(
                command_buffer,
                begin_info,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkEndCommandBuffer.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn end_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().end_command_buffer)(
                command_buffer,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkResetCommandBuffer.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn reset_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        flags: vk::CommandBufferResetFlags,
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().reset_command_buffer)(
                command_buffer,
                flags,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBindPipeline.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_bind_pipeline(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_bind_point: vk::PipelineBindPoint,
        pipeline: vk::Pipeline,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_bind_pipeline)(
                command_buffer,
                pipeline_bind_point,
                pipeline,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetViewport.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_viewport(
        &self,
        command_buffer: vk::CommandBuffer,
        first_viewport: u32,
        viewports: &[vk::Viewport],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_set_viewport)(
                command_buffer,
                first_viewport,
                viewports.len() as u32,
                viewports.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetScissor.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_scissor(
        &self,
        command_buffer: vk::CommandBuffer,
        first_scissor: u32,
        scissors: &[vk::Rect2D]
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_set_scissor)(
                command_buffer,
                first_scissor,
                scissors.len() as u32,
                scissors.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetLineWidth.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_line_width(
        &self,
        command_buffer: vk::CommandBuffer,
        line_width: f32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_set_line_width)(
                command_buffer,
                line_width,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthBias.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_depth_bias(
        &self,
        command_buffer: vk::CommandBuffer,
        depth_bias_constant_factor: f32,
        depth_bias_clamp: f32,
        depth_bias_slope_factor: f32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_set_depth_bias)(
                command_buffer,
                depth_bias_constant_factor,
                depth_bias_clamp,
                depth_bias_slope_factor,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetBlendConstants.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_blend_constants(
        &self,
        command_buffer: vk::CommandBuffer,
        blend_constants: &[f32; 4],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_set_blend_constants)(
                command_buffer,
                blend_constants,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthBounds.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_depth_bounds(
        &self,
        command_buffer: vk::CommandBuffer,
        min_depth_bounds: f32,
        max_depth_bounds: f32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_set_depth_bounds)(
                command_buffer,
                min_depth_bounds,
                max_depth_bounds,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetStencilCompareMask.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_stencil_compare_mask(
        &self,
        command_buffer: vk::CommandBuffer,
        face_mask: vk::StencilFaceFlags,
        compare_mask: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_set_stencil_compare_mask)(
                command_buffer,
                face_mask,
                compare_mask,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetStencilWriteMask.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_stencil_write_mask(
        &self,
        command_buffer: vk::CommandBuffer,
        face_mask: vk::StencilFaceFlags,
        write_mask: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_set_stencil_write_mask)(
                command_buffer,
                face_mask,
                write_mask,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetStencilReference.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_stencil_reference(
        &self,
        command_buffer: vk::CommandBuffer,
        face_mask: vk::StencilFaceFlags,
        reference: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_set_stencil_reference)(
                command_buffer,
                face_mask,
                reference,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBindDescriptorSets.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`cmd_bind_descriptor_sets2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::cmd_bind_descriptor_sets2
    #[inline]
    pub unsafe fn cmd_bind_descriptor_sets(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_bind_point: vk::PipelineBindPoint,
        layout: vk::PipelineLayout,
        first_set: u32,
        descriptor_sets: &[vk::DescriptorSet],
        dynamic_offsets: &[u32],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_bind_descriptor_sets)(
                command_buffer,
                pipeline_bind_point,
                layout,
                first_set,
                descriptor_sets.len() as u32,
                descriptor_sets.as_ptr(),
                dynamic_offsets.len() as u32,
                dynamic_offsets.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBindIndexBuffer.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`cmd_bind_index_buffer2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::cmd_bind_index_buffer2
    #[inline]
    pub unsafe fn cmd_bind_index_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: vk::IndexType,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_bind_index_buffer)(
                command_buffer,
                buffer,
                offset,
                index_type.into_ash(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBindVertexBuffers.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`cmd_bind_vertex_buffers2`][1]
    ///
    /// # Safety
    /// The length of `bindings` and `offsets` *must* match.
    ///
    /// This is *not* checked at runtime.
    ///
    /// *General safety:*
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::cmd_bind_vertex_buffers2
    #[inline]
    pub unsafe fn cmd_bind_vertex_buffers(
        &self,
        command_buffer: vk::CommandBuffer,
        first_binding: u32,
        bindings: &[vk::Buffer],
        offsets: &[vk::DeviceSize],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_bind_vertex_buffers)(
                command_buffer,
                first_binding,
                bindings.len() as u32,
                bindings.as_ptr(),
                offsets.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDraw.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_draw(
        &self,
        command_buffer: vk::CommandBuffer,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_draw)(
                command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDrawIndexed.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_draw_indexed(
        &self,
        command_buffer: vk::CommandBuffer,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_draw_indexed)(
                command_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDrawIndirect.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_draw_indirect(
        &self,
        command_buffer: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        draw_count: u32,
        stride: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_draw_indirect)(
                command_buffer,
                buffer,
                offset,
                draw_count,
                stride,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDrawIndexedIndirect.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_draw_indexed_indirect(
        &self,
        command_buffer: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        draw_count: u32,
        stride: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_draw_indexed_indirect)(
                command_buffer,
                buffer,
                offset,
                draw_count,
                stride,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDispatch.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_dispatch(
        &self,
        command_buffer: vk::CommandBuffer,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_dispatch)(
                command_buffer,
                group_count_x,
                group_count_y,
                group_count_z,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDispatchIndirect.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_dispatch_indirect(
        &self,
        command_buffer: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_dispatch_indirect)(
                command_buffer,
                buffer,
                offset,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdCopyBuffer.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`cmd_copy_buffer2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::cmd_copy_buffer2
    #[inline]
    #[deprecated = "use cmd_copy_buffer2"]
    pub unsafe fn cmd_copy_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        regions: &[vk::BufferCopy],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_copy_buffer)(
                command_buffer,
                src_buffer,
                dst_buffer,
                regions.len() as u32,
                regions.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdCopyImage.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`cmd_copy_image2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::cmd_copy_image2
    #[inline]
    #[deprecated = "use cmd_copy_image2"]
    pub unsafe fn cmd_copy_image(
        &self,
        command_buffer: vk::CommandBuffer,
        src_image: vk::Image,
        src_image_layout: vk::ImageLayout,
        dst_image: vk::Image,
        dst_image_layout: vk::ImageLayout,
        regions: &[vk::ImageCopy],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_copy_image)(
                command_buffer,
                src_image,
                src_image_layout,
                dst_image,
                dst_image_layout,
                regions.len() as u32,
                regions.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBlitImage.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`cmd_blit_image2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::cmd_blit_image2
    #[inline]
    #[deprecated = "use cmd_blit_image2"]
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn cmd_blit_image(
        &self,
        command_buffer: vk::CommandBuffer,
        src_image: vk::Image,
        src_image_layout: vk::ImageLayout,
        dst_image: vk::Image,
        dst_image_layout: vk::ImageLayout,
        regions: &[vk::ImageBlit],
        filter: vk::Filter,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_blit_image)(
                command_buffer,
                src_image,
                src_image_layout,
                dst_image,
                dst_image_layout,
                regions.len() as u32,
                regions.as_ptr(),
                filter,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdCopyBufferToImage.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`cmd_copy_buffer_to_image2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::cmd_copy_buffer_to_image2
    #[inline]
    #[deprecated = "use cmd_copy_buffer_to_image2"]
    pub unsafe fn cmd_copy_buffer_to_image(
        &self,
        command_buffer: vk::CommandBuffer,
        src_buffer: vk::Buffer,
        dst_image: vk::Image,
        dst_image_layout: vk::ImageLayout,
        regions: &[vk::BufferImageCopy],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_copy_buffer_to_image)(
                command_buffer,
                src_buffer,
                dst_image,
                dst_image_layout,
                regions.len() as u32,
                regions.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdCopyImageToBuffer.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`cmd_copy_image_to_buffer2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::cmd_copy_image_to_buffer2
    #[inline]
    #[deprecated = "use cmd_copy_image_to_buffer2"]
    pub unsafe fn cmd_copy_image_to_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        src_image: vk::Image,
        src_image_layout: vk::ImageLayout,
        dst_buffer: vk::Buffer,
        regions: &[vk::BufferImageCopy],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_copy_image_to_buffer)(
                command_buffer,
                src_image,
                src_image_layout,
                dst_buffer,
                regions.len() as u32,
                regions.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdUpdateBuffer.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_update_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        dst_buffer: vk::Buffer,
        dst_offset: vk::DeviceSize,
        data: &[u8],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_update_buffer)(
                command_buffer,
                dst_buffer,
                dst_offset,
                data.len() as vk::DeviceSize,
                data.as_ptr().cast::<c_void>()
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdFillBuffer.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_fill_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        dst_buffer: vk::Buffer,
        dst_offset: vk::DeviceSize,
        size: vk::DeviceSize,
        data: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_fill_buffer)(
                command_buffer,
                dst_buffer,
                dst_offset,
                size,
                data,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdClearColorImage.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn clear_color_image(
        &self,
        command_buffer: vk::CommandBuffer,
        image: vk::Image,
        image_layout: vk::ImageLayout,
        color: &vk::ClearColorValue,
        ranges: &[vk::ImageSubresourceRange],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_clear_color_image)(
                command_buffer,
                image,
                image_layout,
                color,
                ranges.len() as u32,
                ranges.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdClearColorImage.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    pub unsafe fn cmd_clear_color_image(
        &self,
        command_buffer: vk::CommandBuffer,
        image: vk::Image,
        image_layout: vk::ImageLayout,
        color: &vk::ClearColorValue,
        ranges: &[vk::ImageSubresourceRange],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_clear_color_image)(
                command_buffer,
                image,
                image_layout,
                color,
                ranges.len() as u32,
                ranges.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdClearDepthStencilImage.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_clear_depth_stencil_image(
        &self,
        command_buffer: vk::CommandBuffer,
        image: vk::Image,
        image_layout: vk::ImageLayout,
        depth_stencil: &vk::ClearDepthStencilValue,
        ranges: &[vk::ImageSubresourceRange],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_clear_depth_stencil_image)(
                command_buffer,
                image,
                image_layout,
                depth_stencil,
                ranges.len() as u32,
                ranges.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdClearAttachments.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_clear_attachments(
        &self,
        command_buffer: vk::CommandBuffer,
        attachments: &[vk::ClearAttachment],
        rects: &[vk::ClearRect],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_clear_attachments)(
                command_buffer,
                attachments.len() as u32,
                attachments.as_ptr(),
                rects.len() as u32,
                rects.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdResolveImage.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`cmd_resolve_image2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::cmd_resolve_image2
    #[inline]
    #[deprecated = "use cmd_resolve_image2"]
    pub unsafe fn cmd_resolve_image(
        &self,
        command_buffer: vk::CommandBuffer,
        src_image: vk::Image,
        src_image_layout: vk::ImageLayout,
        dst_image: vk::Image,
        dst_image_layout: vk::ImageLayout,
        regions: &[vk::ImageResolve],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_resolve_image)(
                command_buffer,
                src_image,
                src_image_layout,
                dst_image,
                dst_image_layout,
                regions.len() as u32,
                regions.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetEvent.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_event(
        &self,
        command_buffer: vk::CommandBuffer,
        event: vk::Event,
        stage_mask: vk::PipelineStageFlags,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_set_event)(
                command_buffer,
                event,
                stage_mask,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdResetEvent.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_reset_event(
        &self,
        command_buffer: vk::CommandBuffer,
        event: vk::Event,
        stage_mask: vk::PipelineStageFlags,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_reset_event)(
                command_buffer,
                event,
                stage_mask,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdWaitEvents.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// - [`cmd_wait_events2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::cmd_wait_events2
    #[inline]
    #[allow(clippy::too_many_arguments)]
    #[deprecated = "use cmd_wait_events2"]
    pub unsafe fn cmd_wait_events(
        &self,
        command_buffer: vk::CommandBuffer,
        events: &[vk::Event],
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
        memory_barriers: &[vk::MemoryBarrier],
        buffer_memory_barriers: &[vk::BufferMemoryBarrier],
        image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_wait_events)(
                command_buffer,
                events.len() as u32,
                events.as_ptr(),
                src_stage_mask,
                dst_stage_mask,
                memory_barriers.len() as u32,
                memory_barriers.as_ptr(),
                buffer_memory_barriers.len() as u32,
                buffer_memory_barriers.as_ptr(),
                image_memory_barriers.len() as u32,
                image_memory_barriers.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdPipelineBarrier.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # See also
    /// [`cmd_pipeline_barrier2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::cmd_pipeline_barrier2
    #[inline]
    #[allow(clippy::too_many_arguments)]
    #[deprecated = "use cmd_pipeline_barrier2"]
    pub unsafe fn cmd_pipeline_barrier(
        &self,
        command_buffer: vk::CommandBuffer,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
        dependency_flags: vk::DependencyFlags,
        memory_barriers: &[vk::MemoryBarrier],
        buffer_memory_barriers: &[vk::BufferMemoryBarrier],
        image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_pipeline_barrier)(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                memory_barriers.len() as u32,
                memory_barriers.as_ptr(),
                buffer_memory_barriers.len() as u32,
                buffer_memory_barriers.as_ptr(),
                image_memory_barriers.len() as u32,
                image_memory_barriers.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkQueueSubmit.html>
    ///
    /// Part of Vulkan 1.0
    ///
    /// # See also
    /// [`queue_submit2`][1]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::queue_submit2
    #[inline]
    #[deprecated = "use queue_submit2"]
    pub unsafe fn queue_submit(
        &self,
        queue: vk::Queue,
        submits: &[vk::SubmitInfo],
        fence: vk::Fence,
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_0().queue_submit)(
                queue,
                submits.len() as u32,
                submits.as_ptr(),
                fence,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBeginQuery.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_begin_query(
        &self,
        command_buffer: vk::CommandBuffer,
        query_pool: vk::QueryPool,
        query: u32,
        flags: vk::QueryControlFlags,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_begin_query)(
                command_buffer,
                query_pool,
                query,
                flags,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdEndQuery.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_end_query(
        &self,
        command_buffer: vk::CommandBuffer,
        query_pool: vk::QueryPool,
        query: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_end_query)(
                command_buffer,
                query_pool,
                query,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdResetQueryPool.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_reset_query_pool(
        &self,
        command_buffer: vk::CommandBuffer,
        query_pool: vk::QueryPool,
        first_query: u32,
        query_count: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_reset_query_pool)(
                command_buffer,
                query_pool,
                first_query,
                query_count,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdWriteTimestamp.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_write_timestamp(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_stage: vk::PipelineStageFlags,
        query_pool: vk::QueryPool,
        query: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_write_timestamp)(
                command_buffer,
                pipeline_stage,
                query_pool,
                query,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdCopyQueryPoolResults.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn cmd_copy_query_pool_results(
        &self,
        command_buffer: vk::CommandBuffer,
        query_pool: vk::QueryPool,
        first_query: u32,
        query_count: u32,
        dst_buffer: vk::Buffer,
        dst_offset: vk::DeviceSize,
        stride: vk::DeviceSize,
        flags: vk::QueryResultFlags,
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_copy_query_pool_results)(
                command_buffer,
                query_pool,
                first_query,
                query_count,
                dst_buffer,
                dst_offset,
                stride,
                flags,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdPushConstants.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_push_constants(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        stage_flags: vk::ShaderStageFlags,
        offset: u32,
        data: &[u8],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_push_constants)(
                command_buffer,
                pipeline_layout,
                stage_flags,
                offset,
                data.len() as u32,
                data.as_ptr().cast::<c_void>(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdExecuteCommands.html>
    ///
    /// Part of Vulkan 1.0 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_execute_commands(
        &self,
        primary: vk::CommandBuffer,
        secondaries: &[vk::CommandBuffer],
    ) {
        unsafe {
            (self.fns().fp_v1_0().cmd_execute_commands)(
                primary,
                secondaries.len() as u32,
                secondaries.as_ptr(),
            )
        }
    }


    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkBindBufferMemory2.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn bind_buffer_memory2(
        &self,
        bind_infos: &[vk::BindBufferMemoryInfo<'_>],
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_1().bind_buffer_memory2)(
                self.handle(),
                bind_infos.len() as u32,
                bind_infos.as_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkBindImageMemory2.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn bind_image_memory2(
        &self,
        bind_infos: &[vk::BindImageMemoryInfo<'_>]
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_1().bind_image_memory2)(
                self.handle(),
                bind_infos.len() as u32,
                bind_infos.as_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDispatchBase.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub unsafe fn cmd_dispatch_base(
        &self,
        command_buffer: vk::CommandBuffer,
        base_group_x: u32,
        base_group_y: u32,
        base_group_z: u32,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) {
        unsafe {
            (self.fns().fp_v1_1().cmd_dispatch_base)(
                command_buffer,
                base_group_x,
                base_group_y,
                base_group_z,
                group_count_x,
                group_count_y,
                group_count_z,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetBufferMemoryRequirements2.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// If you are looking for [`get_buffer_memory_requirements`] (without 2), you should
    /// prefer this one because it can get you more information about the requirements (see
    /// [`vk::MemoryDedicatedRequirements`]). If you still want to use
    /// `vkGetBufferMemoryRequirements`, you can use the [`raw function pointer`][1].
    ///
    /// # See also
    /// - [`get_device_buffer_memory_requirements`][2] for a version that doesn't require a
    ///   buffer handle.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: DeviceFnV1_0::get_buffer_memory_requirements
    /// [2]: Self::get_device_buffer_memory_requirements
    #[inline]
    pub unsafe fn get_buffer_memory_requirements2(
        &self,
        info: &vk::BufferMemoryRequirementsInfo2<'_>,
        memory_requirements: &mut vk::MemoryRequirements2<'_>,
    ) {
        unsafe {
            (self.fns().fp_v1_1().get_buffer_memory_requirements2)(
                self.handle(),
                info,
                memory_requirements,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetImageMemoryRequirements2.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// If you are looking for [`get_image_memory_requirements`] (without 2), you should
    /// prefer this one because it can get you more information about the requirements (see
    /// [`vk::MemoryDedicatedRequirements`]). If you still want to use
    /// `vkGetImageMemoryRequirements`, you can use [`raw function pointer`][1].
    ///
    /// # See also
    /// - [`get_device_image_memory_requirements`][2] for a version that doesn't require an
    ///   image handle.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: DeviceFnV1_0::get_image_memory_requirements
    /// [2]: Self::get_device_image_memory_requirements
    #[inline]
    pub unsafe fn get_image_memory_requirements2(
        &self,
        info: &vk::ImageMemoryRequirementsInfo2<'_>,
        memory_requirements: &mut vk::MemoryRequirements2<'_>,
    ) {
        unsafe {
            (self.fns().fp_v1_1().get_image_memory_requirements2)(
                self.handle(),
                info,
                memory_requirements,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetImageSparseMemoryRequirements2.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// This gets the number of requirements to allocate.
    ///
    /// If you are looking for [`get_image_sparse_memory_requirements`] (without 2), you should
    /// prefer this one because it's more future-proof. If you still want to use
    /// `vkGetImageSparseMemoryRequirements`, you use the [`raw function pointer`][1].
    //
    /// # See also
    /// - [`get_device_image_sparse_memory_requirements`][2] for a version that doesn't require
    ///   an image handle.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: DeviceFnV1_0::get_image_sparse_memory_requirements
    /// [2]: Self::get_device_image_sparse_memory_requirements
    #[inline]
    pub unsafe fn get_image_sparse_memory_requirements2_len(
        &self,
        info: &vk::ImageSparseMemoryRequirementsInfo2<'_>,
    ) -> u32 {
        let mut sparse_memory_requirement_count = 0;
        unsafe {
            (self.fns().fp_v1_1().get_image_sparse_memory_requirements2)(
                self.handle(),
                info,
                &mut sparse_memory_requirement_count,
                ptr::null_mut(),
            );
        }
        sparse_memory_requirement_count
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetImageSparseMemoryRequirements2.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// To get the number of requirements to allocate, use [`get_image_sparse_memory_requirements2_len`][1].
    ///
    /// If you are looking for [`get_image_sparse_memory_requirements`] (without 2), you should
    /// prefer this one because it's more future-proof. If you still want to use
    /// `vkGetImageSparseMemoryRequirements`, you can use the [`raw function pointer`][2].
    //
    /// # See also
    /// - [`get_device_image_sparse_memory_requirements`][3] for a version that doesn't require
    ///   an image handle.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::get_image_sparse_memory_requirements2_len
    /// [2]: DeviceFnV1_0::get_image_sparse_memory_requirements
    /// [3]: Self::get_device_image_sparse_memory_requirements
    #[inline]
    pub unsafe fn get_image_sparse_memory_requirements2(
        &self,
        info: &vk::ImageSparseMemoryRequirementsInfo2<'_>,
        out: &mut [vk::SparseImageMemoryRequirements2<'_>],
    ) {
        let mut len = out.len() as u32;
        unsafe {
            (self.fns().fp_v1_1().get_image_sparse_memory_requirements2)(
                self.handle(),
                info,
                &mut len,
                out.as_mut_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkTrimCommandPool.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn trim_command_pool(
        &self,
        pool: vk::CommandPool,
        flags: vk::CommandPoolTrimFlags,
    ) {
        unsafe {
            (self.fns().fp_v1_1().trim_command_pool)(
                self.handle(),
                pool,
                flags,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetDeviceQueue2.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn get_device_queue2(
        &self,
        queue_info: &vk::DeviceQueueInfo2<'_>,
    ) -> vk::Queue {
        let mut queue = vk::Queue::null();
        unsafe {
            (self.fns().fp_v1_1().get_device_queue2)(
                self.handle(),
                queue_info,
                &mut queue,
            )
        }
        queue
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateSamplerYcbcrConversion.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_sampler_ycbcr_conversion(
        &self,
        create_info: &vk::SamplerYcbcrConversionCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::SamplerYcbcrConversion> {
        let mut ycbcr_conversion = vk::SamplerYcbcrConversion::null();
        unsafe {
            (self.fns().fp_v1_1().create_sampler_ycbcr_conversion)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut ycbcr_conversion,
            )
        }.result_with_success(ycbcr_conversion)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroySamplerYcbcrConversion.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_sampler_ycbcr_conversion(
        &self,
        ycbcr_conversion: vk::SamplerYcbcrConversion,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_1().destroy_sampler_ycbcr_conversion)(
                self.handle(),
                ycbcr_conversion,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateDescriptorUpdateTemplate.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_descriptor_update_template(
        &self,
        create_info: &vk::DescriptorUpdateTemplateCreateInfo<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::DescriptorUpdateTemplate> {
        let mut descriptor_update_template = vk::DescriptorUpdateTemplate::null();
        unsafe {
            (self.fns().fp_v1_1().create_descriptor_update_template)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut descriptor_update_template,
            )
        }.result_with_success(descriptor_update_template)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroyDescriptorUpdateTemplate.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_descriptor_update_template(
        &self,
        descriptor_update_template: vk::DescriptorUpdateTemplate,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_v1_1().destroy_descriptor_update_template)(
                self.handle(),
                descriptor_update_template,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkUpdateDescriptorSetWithTemplate.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn update_descriptor_set_with_template(
        &self,
        descriptor_set: vk::DescriptorSet,
        descriptor_update_template: vk::DescriptorUpdateTemplate,
        p_data: *const (),
    ) {
        unsafe {
            (self.fns().fp_v1_1().update_descriptor_set_with_template)(
                self.handle(),
                descriptor_set,
                descriptor_update_template,
                p_data.cast(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetDescriptorSetLayoutSupport.html>
    ///
    /// Part of Vulkan 1.1 core.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn get_descriptor_set_layout_support(
        &self,
        create_info: &vk::DescriptorSetLayoutCreateInfo<'_>,
        support: &mut vk::DescriptorSetLayoutSupport<'_>,
    ) {
        unsafe {
            (self.fns().fp_v1_1().get_descriptor_set_layout_support)(
                self.handle(),
                create_info,
                support,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkWaitSemaphores.html>
    ///
    /// Part of Vulkan 1.2 core, otherwise provided by VK_KHR_timeline_semaphore device extension.
    ///
    /// # [`Ok`] values
    /// [`vk::Result::SUCCESS`]
    /// [`vk::Result::TIMEOUT`]
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn wait_semaphores(
        &self,
        wait_info: &vk::SemaphoreWaitInfo<'_>,
        timeout: u64,
    ) -> VkResult<vk::Result> {
        unsafe {
            (self.fns().fp_v1_2().wait_semaphores)(
                self.handle(),
                wait_info,
                timeout,
            )
        }.result_with_success(vk::Result::SUCCESS).filter_err(|&err| matches!(
            err, vk::Result::TIMEOUT,
        ).then_some(err))
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetSemaphoreCounterValue.html>
    ///
    /// Part of Vulkan 1.2 core, otherwise provided by VK_KHR_timeline_semaphore device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn get_semaphore_counter_value(
        &self,
        semaphore: vk::Semaphore,
    ) -> VkResult<u64> {
        let mut value = 0;
        unsafe {
            (self.fns().fp_v1_2().get_semaphore_counter_value)(
                self.handle(),
                semaphore,
                &mut value,
            )
        }.result_with_success(value)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkSignalSemaphore.html>
    ///
    /// Part of Vulkan 1.2 core, otherwise provided by VK_KHR_timeline_semaphore device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn signal_semaphore(
        &self,
        signal_info: &vk::SemaphoreSignalInfo<'_>
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_2().signal_semaphore)(
                self.handle(),
                signal_info,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBeginRendering.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_dynamic_rendering device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_begin_rendering(
        &self,
        command_buffer: vk::CommandBuffer,
        rendering_info: &vk::RenderingInfo<'_>
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_begin_rendering)(
                command_buffer,
                rendering_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdEndRendering.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_dynamic_rendering device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_end_rendering(
        &self,
        command_buffer: vk::CommandBuffer,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_end_rendering)(
                command_buffer,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBindVertexBuffers2.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// The length of `buffers` *must* match the length of `offsets` and if [`Some`], `sizes` and
    /// `strides`.
    ///
    /// This is *not* checked at runtime.
    ///
    /// *General safety:*
    ///
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_bind_vertex_buffers2(
        &self,
        command_buffer: vk::CommandBuffer,
        first_binding: u32,
        buffers: &[vk::Buffer],
        offsets: &[vk::DeviceSize],
        sizes: Option<&[vk::DeviceSize]>,
        strides: Option<&[vk::DeviceSize]>,
    ) {
        unsafe {
            if buffers.is_empty() {
                (self.fns().fp_v1_3().cmd_bind_vertex_buffers2)(
                    command_buffer,
                    first_binding,
                    0,
                    ptr::null(),
                    ptr::null(),
                    ptr::null(),
                    ptr::null(),
                )
            } else {
                (self.fns().fp_v1_3().cmd_bind_vertex_buffers2)(
                    command_buffer,
                    first_binding,
                    buffers.len() as u32,
                    buffers.as_ptr(),
                    offsets.as_ptr(),
                    sizes.as_slice_ptr(),
                    strides.as_slice_ptr(),
                )
            }
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetCullMode.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_cull_mode(
        &self,
        command_buffer: vk::CommandBuffer,
        cull_mode: vk::CullModeFlags,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_cull_mode)(
                command_buffer,
                cull_mode,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthBoundsTestEnable.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_depth_bounds_test_enable(
        &self,
        command_buffer: vk::CommandBuffer,
        depth_bounds_test_enable: bool,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_depth_bounds_test_enable)(
                command_buffer,
                depth_bounds_test_enable as vk::Bool32,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthCompareOp.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_depth_compare_op(
        &self,
        command_buffer: vk::CommandBuffer,
        depth_compare_op: vk::CompareOp,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_depth_compare_op)(
                command_buffer,
                depth_compare_op,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthTestEnable.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_depth_test_enable(
        &self,
        command_buffer: vk::CommandBuffer,
        depth_test_enable: bool,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_depth_test_enable)(
                command_buffer,
                depth_test_enable as vk::Bool32,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthWriteEnable.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_depth_write_enable(
        &self,
        command_buffer: vk::CommandBuffer,
        depth_write_enable: bool,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_depth_write_enable)(
                command_buffer,
                depth_write_enable as vk::Bool32,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetFrontFace.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_front_face(
        &self,
        command_buffer: vk::CommandBuffer,
        front_face: vk::FrontFace,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_front_face)(
                command_buffer,
                front_face,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetPrimitiveTopology.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_primitive_topology(
        &self,
        command_buffer: vk::CommandBuffer,
        primitive_topology: vk::PrimitiveTopology,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_primitive_topology)(
                command_buffer,
                primitive_topology,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetScissorWithCount.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_scissor_with_count(
        &self,
        command_buffer: vk::CommandBuffer,
        scissors: &[vk::Rect2D],
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_scissor_with_count)(
                command_buffer,
                scissors.len() as u32,
                scissors.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetStencilOp.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_stencil_op(
        &self,
        command_buffer: vk::CommandBuffer,
        face_mask: vk::StencilFaceFlags,
        fail_op: vk::StencilOp,
        pass_op: vk::StencilOp,
        depth_fail_op: vk::StencilOp,
        compare_op: vk::CompareOp,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_stencil_op)(
                command_buffer,
                face_mask,
                fail_op,
                pass_op,
                depth_fail_op,
                compare_op,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetStencilTestEnable.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_stencil_test_enable(
        &self,
        command_buffer: vk::CommandBuffer,
        stenci_test_enable: bool,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_stencil_test_enable)(
                command_buffer,
                stenci_test_enable as vk::Bool32,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetViewportWithCount.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_viewport_with_count(
        &self,
        command_buffer: vk::CommandBuffer,
        viewports: &[vk::Viewport],
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_viewport_with_count)(
                command_buffer,
                viewports.len() as u32,
                viewports.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetRasterizerDiscardEnable.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_rasterizer_discard_enable(
        &self,
        command_buffer: vk::CommandBuffer,
        rasterizer_discard_enable: bool,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_rasterizer_discard_enable)(
                command_buffer,
                rasterizer_discard_enable as vk::Bool32,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthBiasEnable.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_depth_bias_enable(
        &self,
        command_buffer: vk::CommandBuffer,
        depth_bias_enable: bool,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_depth_bias_enable)(
                command_buffer,
                depth_bias_enable as vk::Bool32,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetPrimitiveRestartEnable.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_EXT_extended_dynamic_state device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_primitive_restart_enable(
        &self,
        command_buffer: vk::CommandBuffer,
        primitive_restart_enable: bool,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_set_primitive_restart_enable)(
                command_buffer,
                primitive_restart_enable as vk::Bool32,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBlitImage2.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_copy_commands2 device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_blit_image2(
        &self,
        command_buffer: vk::CommandBuffer,
        blit_image_info: &vk::BlitImageInfo2<'_>,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_blit_image2)(
                command_buffer,
                blit_image_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdCopyBuffer2.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_copy_commands2 device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_copy_buffer2(
        &self,
        command_buffer: vk::CommandBuffer,
        copy_buffer_info: &vk::CopyBufferInfo2,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_copy_buffer2)(
                command_buffer,
                copy_buffer_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdCopyBufferToImage2.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_copy_commands2 device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_copy_buffer_to_image2(
        &self,
        command_buffer: vk::CommandBuffer,
        copy_buffer_to_image_info: &vk::CopyBufferToImageInfo2<'_>
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_copy_buffer_to_image2)(
                command_buffer,
                copy_buffer_to_image_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdCopyImage2.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_copy_commands2 device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_copy_image2(
        &self,
        command_buffer: vk::CommandBuffer,
        copy_image_info: &vk::CopyImageInfo2<'_>
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_copy_image2)(
                command_buffer,
                copy_image_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdCopyImageToBuffer2.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_copy_commands2 device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_copy_image_to_buffer2(
        &self,
        command_buffer: vk::CommandBuffer,
        copy_image_to_buffer_info: &vk::CopyImageToBufferInfo2<'_>
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_copy_image_to_buffer2)(
                command_buffer,
                copy_image_to_buffer_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdResolveImage2.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_copy_commands2 device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_resolve_image2(
        &self,
        command_buffer: vk::CommandBuffer,
        resolve_image_info: &vk::ResolveImageInfo2<'_>
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_resolve_image2)(
                command_buffer,
                resolve_image_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdWaitEvents2.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_synchronization2 device extension.
    ///
    /// # Safety
    /// The lengths of `events` and `dependency_infos` *must* match.
    ///
    /// This is *not* checked on runtime.
    ///
    /// *General safety:*
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_wait_events2(
        &self,
        command_buffer: vk::CommandBuffer,
        events: &[vk::Event],
        dependency_infos: &[vk::DependencyInfo<'_>],
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_wait_events2)(
                command_buffer,
                events.len() as u32,
                events.as_ptr(),
                dependency_infos.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdPipelineBarrier2.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_synchronization2 device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_pipeline_barrier2(
        &self,
        command_buffer: vk::CommandBuffer,
        dependency_info: &vk::DependencyInfo<'_>,
    ) {
        unsafe {
            (self.fns().fp_v1_3().cmd_pipeline_barrier2)(
                command_buffer,
                dependency_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkQueueSubmit2.html>
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_synchronization2 device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn queue_submit2(
        &self,
        queue: vk::Queue,
        submits: &[vk::SubmitInfo2],
        fence: vk::Fence,
    ) -> VkResult<()> {
        unsafe {
            (self.fns().fp_v1_3().queue_submit2)(
                queue,
                submits.len() as u32,
                submits.as_ptr(),
                fence,
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetDeviceBufferMemoryRequirements.html>
    ///
    /// Gets [`vk::Buffer`] memory requirements without having to create a buffer.
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_maintenance4 device extension.
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn get_device_buffer_memory_requirements(
        &self,
        info: &vk::DeviceBufferMemoryRequirements<'_>,
        memory_requirements: &mut vk::MemoryRequirements2<'_>,
    ) {
        unsafe {
            (self.fns().fp_v1_3().get_device_buffer_memory_requirements)(
                self.handle(),
                info,
                memory_requirements,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetDeviceImageMemoryRequirements.html>
    ///
    /// Gets [`vk::Image`] memory requirements without having to create an image.
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_maintenance4 device extension.
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn get_device_image_memory_requirements(
        &self,
        info: &vk::DeviceImageMemoryRequirements<'_>,
        memory_requirements: &mut vk::MemoryRequirements2<'_>,
    ) {
        unsafe {
            (self.fns().fp_v1_3().get_device_image_memory_requirements)(
                self.handle(),
                info,
                memory_requirements,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetDeviceImageSparseMemoryRequirements.html>
    ///
    /// Gets sparse [`vk::Image`] memory requirements without having to create an image.
    ///
    /// This gets the number of requirements to allocate.
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_maintenance4 device extension.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn get_device_image_sparse_memory_requirements_len(
        &self,
        info: &vk::DeviceImageMemoryRequirements<'_>,
    ) -> u32 {
        let mut sparse_memory_requirement_count = 0;
        unsafe {
            (self.fns().fp_v1_3().get_device_image_sparse_memory_requirements)(
                self.handle(),
                info,
                &mut sparse_memory_requirement_count,
                ptr::null_mut(),
            )
        };
        sparse_memory_requirement_count
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetDeviceImageSparseMemoryRequirements.html>
    ///
    /// Gets sparse [`vk::Image`] memory requirements without having to create an image.
    ///
    /// To get the number of requirements to allocate, use [`get_device_image_sparse_memory_requirements_len`][1].
    ///
    /// Part of Vulkan 1.3 core, otherwise provided by VK_KHR_maintenance4 device extension.
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    ///
    /// [1]: Self::get_device_image_sparse_memory_requirements_len
    #[inline]
    pub unsafe fn get_device_image_sparse_memory_requirements(
        &self,
        info: &vk::DeviceImageMemoryRequirements<'_>,
        out: &mut [vk::SparseImageMemoryRequirements2<'_>]
    ) {
        let mut len = out.len() as u32;
        unsafe {
            (self.fns().fp_v1_3().get_device_image_sparse_memory_requirements)(
                self.handle(),
                info,
                &mut len,
                out.as_mut_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetRenderingAttachmentLocations.html>
    ///
    /// Part of Vulkan 1.4 core, otherwise provided by VK_KHR_dynamic_rendering_local_read device extension.
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_rendering_attachment_locations(
        &self,
        command_buffer: vk::CommandBuffer,
        location_info: &vk::RenderingAttachmentLocationInfo<'_>,
    ) {
        unsafe {
            (self.fns().fp_v1_4().cmd_set_rendering_attachment_locations)(
                command_buffer,
                location_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetRenderingInputAttachmentIndices.html>
    ///
    /// Part of Vulkan 1.4 core, otherwise provided by VK_KHR_dynamic_rendering_local_read device extension.
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_set_rendering_input_attachment_indices(
        &self,
        command_buffer: vk::CommandBuffer,
        input_attachment_index_info: &vk::RenderingInputAttachmentIndexInfo<'_>,
    ) {
        unsafe {
            (self.fns().fp_v1_4().cmd_set_rendering_input_attachment_indices)(
                command_buffer,
                input_attachment_index_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBindIndexBuffer2.html>
    ///
    /// Part of Vulkan 1.4 core, otherwise provided by VK_KHR_maintenance5 device extension.
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_bind_index_buffer2(
        &self,
        command_buffer: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
        index_type: vk::IndexType,
    ) {
        unsafe {
            (self.fns().fp_v1_4().cmd_bind_index_buffer2)(
                command_buffer,
                buffer,
                offset,
                size,
                index_type,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBindDescriptorSets2.html>
    ///
    /// Part of Vulkan 1.4 core, otherwise provided by VK_KHR_maintenance6 device extension.
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_bind_descriptor_sets2(
        &self,
        command_buffer: vk::CommandBuffer,
        bind_decsriptor_sets_info: &vk::BindDescriptorSetsInfo<'_>
    ) {
        unsafe {
            (self.fns().fp_v1_4().cmd_bind_descriptor_sets2)(
                command_buffer,
                bind_decsriptor_sets_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdPushConstants2.html>
    ///
    /// Part of Vulkan 1.4 core, otherwise provided by VK_KHR_maintenance6 device extension.
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn cmd_push_constants2(
        &self,
        command_buffer: vk::CommandBuffer,
        push_constants_info: &vk::PushConstantsInfo<'_>
    ) {
        unsafe {
            (self.fns().fp_v1_4().cmd_push_constants2)(
                command_buffer,
                push_constants_info,
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateSwapchainKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn create_swapchain(
        &self,
        create_info: &vk::SwapchainCreateInfoKHR<'_>,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<vk::SwapchainKHR> {
        let mut swapchain = vk::SwapchainKHR::null();
        unsafe {
            (self.fns().fp_swapchain().create_swapchain_khr)(
                self.handle(),
                create_info,
                allocator.as_ptr(),
                &mut swapchain,
            )
        }.result_with_success(swapchain)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkDestroySwapchainKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn destroy_swapchain(
        &self,
        swapchain: vk::SwapchainKHR,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            (self.fns().fp_swapchain().destroy_swapchain_khr)(
                self.handle(),
                swapchain,
                allocator.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetSwapchainImagesKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn get_swapchain_images_len(
        &self,
        swapchain: vk::SwapchainKHR,
    ) -> VkResult<u32> {
        let mut count = 0;
        unsafe {
            (self.fns().fp_swapchain().get_swapchain_images_khr)(
                self.handle(),
                swapchain,
                &mut count,
                ptr::null_mut()
            )
        }.result_with_success(count)
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkGetSwapchainImagesKHR.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn get_swapchain_images(
        &self,
        swapchain: vk::SwapchainKHR,
        out: &mut [vk::Image]
    ) -> VkResult<()> {
        let mut count = out.len() as u32;
        unsafe {
            (self.fns().fp_swapchain().get_swapchain_images_khr)(
                self.handle(),
                swapchain,
                &mut count,
                out.as_mut_ptr(),
            )
        }.result()
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkAcquireNextImageKHR.html>
    ///
    /// [`Ok`] values are:
    ///
    /// [`Some`] with image index and a bool indicating whether the swapchain is suboptimal.
    /// [`None`], which indicates that the `timeout` was reached.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn acquire_next_image(
        &self,
        swapchain: vk::SwapchainKHR,
        timeout: u64,
        semaphore: vk::Semaphore,
        fence: vk::Fence,
    ) -> VkResult<(Option<u32>, bool)> {
        let mut image_index = 0;
        unsafe {
            (self.fns().fp_swapchain().acquire_next_image_khr)(
                self.handle(),
                swapchain,
                timeout,
                semaphore,
                fence,
                &mut image_index,
            )
        }.result_with_success((Some(image_index), false))
        .filter_err(|&err| 
            (err == vk::Result::SUBOPTIMAL_KHR).then_some(
                (Some(image_index), true)
            ).or((err == vk::Result::TIMEOUT).then_some(
                (None, false)
            ))
        )
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkQueuePresentKHR.html>
    ///
    /// On success, returns whether the swapchain is suboptimal for the surface.
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn queue_present(
        &self,
        queue: vk::Queue,
        present_info: &vk::PresentInfoKHR<'_>
    ) -> VkResult<bool> {
        unsafe {
            (self.fns().fp_swapchain().queue_present_khr)(
                queue,
                present_info
            )
        }.result_with_success(false)
        .filter_err(|&err|
            (err == vk::Result::SUBOPTIMAL_KHR).then_some(
                true
            )
        )
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkWaitForPresent2KHR.html>
    ///
    /// [`Ok`] values are:
    ///
    /// [`vk::Result::SUBOPTIMAL_KHR`]
    /// [`vk::Result::SUCCESS`]
    /// [`vk::Result::TIMEOUT`]
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline]
    pub unsafe fn wait_for_present2(
        &self,
        swapchain: vk::SwapchainKHR,
        present_wait2_info: &vk::PresentWait2InfoKHR<'_>
    ) -> VkResult<vk::Result> {
        unsafe {
            (self.fns().fp_present_wat2().wait_for_present2_khr)(
                self.handle(),
                swapchain,
                present_wait2_info,
            )
        }.result_with_success(vk::Result::SUCCESS)
        .filter_err(|&err| matches!(
            err,
            vk::Result::SUBOPTIMAL_KHR | vk::Result::TIMEOUT,
        ).then_some(err))
    }
}
