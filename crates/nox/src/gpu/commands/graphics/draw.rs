use core::ops::{Deref, DerefMut};
use std::fs::create_dir_all;

use nox_ash::vk;

use compact_str::format_compact;

use nox_mem::{
    vec::{NonNullVec32, Vec32, ArrayVec, Vector},
    alloc::LocalAlloc,
    borrow::SizedCowMut,
};

use nox_alloc::arena::*;

use crate::{
    gpu::{
        prelude::*,
        commands::prelude::ShaderResourceBindCall,
    },
    dev::error::*,
};

#[derive(Clone, Copy, Debug)]
pub struct IndexedDrawInfo {
    first_index: u32,
    index_count: u32,
    index_type: IndexType,
    first_instance: u32,
    instance_count: u32,
    vertex_offset: i32,
}

impl IndexedDrawInfo {
    
    #[inline(always)]
    pub fn new(
        first_index: u32,
        index_count: u32,
        index_type: IndexType,
    ) -> Self {
        Self {
            first_index,
            index_count,
            index_type,
            first_instance: 0,
            instance_count: 1,
            vertex_offset: 0,
        }
    }

    #[inline(always)]
    pub fn with_instances(mut self, first_instance: u32, instance_count: u32) -> Self {
        self.first_instance = first_instance;
        self.instance_count = instance_count;
        self
    }

    #[inline(always)]
    pub fn with_vertex_offset(mut self, offset: i32) -> Self {
        self.vertex_offset = offset;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DrawBufferRange {
    pub id: BufferId,
    pub offset: u64,
    pub size: u64
}

impl DrawBufferRange {

    #[inline(always)]
    pub fn new(id: BufferId, offset: u64, size: u64) -> Self {
        Self {
            id,
            offset,
            size,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct IndexBufferInfo {
    pub id: BufferId,
    pub offset: u64,
}

impl IndexBufferInfo {

    #[inline(always)]
    pub fn new(id: BufferId, offset: u64) -> Self {
        Self {
            id,
            offset,
        }
    }
}

pub(super) struct DrawCall {
    pub index_buffer: Option<DrawBufferRange>,
    pub vertex_buffers: NonNullVec32<'static, DrawBufferRange>,
}

#[derive(Default)]
pub(crate) struct DrawCommandCache {
    pub(super) pipelines: Vec32<PipelineHandle>,
    pub(super) shader_resource_binds: Vec32<ShaderResourceBindCall>,
    pub(super) draw_calls: Vec32<DrawCall>,
}

unsafe impl Send for DrawCommandCache {}
unsafe impl Sync for DrawCommandCache {}

impl DrawCommandCache {

    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.pipelines.clear();
        self.shader_resource_binds.clear();
        self.draw_calls.clear();
    }
}

pub(crate) enum DrawCommandAlloc<'a> {
    Borrowed(&'a (dyn LocalAlloc<Error = Error> + 'a)),
    Owned(Box<dyn LocalAlloc<Error = Error> + 'a>),
}

impl<'a> Deref for DrawCommandAlloc<'a> {

    type Target = dyn LocalAlloc<Error = Error> + 'a;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(o) => o.deref(),
        }
    }
}

pub(crate) struct DrawCommandStorage<'a> {
    pub(super) command_buffer: vk::CommandBuffer,
    color_formats: NonNullVec32<'a, vk::Format>,
    depth_format: vk::Format,
    stencil_format: vk::Format,
    sample_count: MSAA,
    pub(super) cache: SizedCowMut<'a, DrawCommandCache>,
    pub(super) wait_scope: vk::PipelineStageFlags2,
    alloc: DrawCommandAlloc<'a>,
}

impl<'a> DrawCommandStorage<'a> {

    #[inline(always)]
    pub unsafe fn new(
        command_buffer: vk::CommandBuffer,
        color_formats: &[vk::Format],
        depth_format: vk::Format,
        stencil_format: vk::Format,
        sample_count: MSAA,
        cache: SizedCowMut<'a, DrawCommandCache>,
        alloc: DrawCommandAlloc<'a>,
    ) -> Result<Self> {
        let mut color_f = NonNullVec32::with_capacity(
            color_formats.len() as u32,
            &*alloc,
        )?.into_static();
        color_f.append(&color_formats);
        Ok(Self {
            command_buffer,
            color_formats: color_f,
            depth_format,
            stencil_format,
            sample_count,
            cache,
            wait_scope: vk::PipelineStageFlags2::NONE,
            alloc,
        })
    }
}

impl<'a> Drop for DrawCommandStorage<'a> {

    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            let alloc = &*self.alloc;
            self.color_formats.drop_and_free(alloc);
            let cache = self.cache.deref_mut();
            for call in &mut cache.shader_resource_binds {
                call.sets.drop_and_free(alloc);
                call.barriers.drop_and_free(alloc);
            }
            for call in &mut cache.draw_calls {
                call.vertex_buffers.drop_and_free(alloc);
            }
            cache.clear();
        }
    }
}

pub struct DrawCommands<'a, 'b> {
    pub(super) gpu: &'a Gpu,
    pub(super) storage: &'a mut DrawCommandStorage<'b>,
    pub(super) buffers: DynResourceReadGuard<'a, BufferMeta, BufferId>,
    pub(super) images: DynResourceReadGuard<'a, ImageMeta, ImageId>,
    pub(super) last_pipeline: Option<GraphicsPipeline>,
}

impl<'a, 'b> DrawCommands<'a, 'b> {

    /// Binds a graphics pipeline used for all subsequent draw commands.
    ///
    /// # Valid usage
    /// - `id` *must* be a valid [`GraphicsPipelineId`].
    /// - The bound pipeline's sample count *must* match the sample counts defined when creating
    /// [`self`].
    /// - The bound pipeline's color output formats *must* match the color output formats defined
    /// when creating [`self`].
    /// - The bound pipeline's depth and stencil formats *must* match the respective formats
    /// defined when creating [`self`].
    /// - The valid usage section of [`DrawCommands::set_multi_viewport`] apply when the number of
    /// viewports and scissors is not one.
    ///
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBindPipeline.html>**
    pub fn bind_pipeline(
        &mut self,
        id: GraphicsPipelineId,
        viewports: &[Viewport],
        scissors: &[Scissor],
    ) -> Result<()> {
        let pipeline = self.gpu.get_graphics_pipeline(id)?;
        if pipeline.samples() != self.storage.sample_count {
            return Err(Error::just_context(format_compact!(
                "pipeline sample count {} must match pass sample count {}",
                self.storage.sample_count, pipeline.samples(),
            )))
        }
        if *pipeline.color_output_formats() != **self.storage.color_formats {
            return Err(Error::just_context(
                "pipeline color output formats don't match with pass"
            ))
        }
        if pipeline.depth_output_format() != self.storage.depth_format {
            return Err(Error::just_context(format_compact!(
                "pipeline depth output format {:?} doesn't match with pass depth format {:?}",
                pipeline.depth_output_format(), self.storage.depth_format,
            )))
        }
        if pipeline.stencil_output_format() != self.storage.stencil_format {
            return Err(Error::just_context(format_compact!(
                "pipeline stencil output format {:?} doesn't match with pass stencil format {:?}",
                pipeline.stencil_output_format(), self.storage.stencil_format,
            )))
        }
        let handle = pipeline.handle().clone();
        unsafe {
            self.gpu.vk().device().cmd_bind_pipeline(
                self.storage.command_buffer, vk::PipelineBindPoint::GRAPHICS, handle.handle()
            );
        }
        self.storage.cache.pipelines.push(handle);
        self.last_pipeline = Some(pipeline.clone());
        if viewports.len() != 1 {
            self.set_multi_viewport(viewports, scissors)?;
        } else {
            self.set_viewport(viewports[0], scissors[0])?;
        }
        Ok(())
    }

    pub fn check_dynamic_state(&self, dynamic_state: DynamicState) -> Result<()> {
        let Some(pipeline) = &self.last_pipeline else {
            return Err(Error::just_context(format_compact!(
                "attempting to set {dynamic_state} with no pipeline bound"
            )))
        };
        if !pipeline.has_dynamic_state(dynamic_state) {
            return Err(Error::just_context(format_compact!(
                "current pipeline's dynamic state doesn't include {dynamic_state}"
            )))
        }
        Ok(())
    }

    /// Dynamically sets the line width for subsequent drawing commands.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic state 
    /// includes [`DynamicState::LineWidth`] and if subsequent drawing commands generate line primitives.
    /// - If [`Gpu::enabled_base_features`] `wide_lines` is set to `false`, `line_width` *must* be 1.0.
    /// # Vulkan docs link
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetLineWidth.html>**
    pub fn set_line_width(
        &mut self,
        line_width: f32,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::LineWidth)?;
        if !self.gpu.enabled_base_features().wide_lines && line_width != 1.0 {
            return Err(Error::just_context(format_compact!(
                "line width must be 1.0 if the wide lines base device feature is not enabled, given line width is {line_width}"
            )))
        }
        unsafe {
            self.gpu.vk().device().cmd_set_line_width(
                self.storage.command_buffer,
                line_width
            );
        }
        Ok(())
    }

    /// Dynamically sets the depth bias factors and clamp for subsequent drawing commands.
    ///
    /// # Parameters
    /// - `depth_bias_constant_factor`: controls the constant value added to each fragment
    /// - `depth_bias_clamp`: is the maximum (or minimum) depth bias of the fragment
    /// - `depth_bias_slope_factor`: a scalar applied to a fragment's slope in depth bias
    /// calculations
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic state
    /// includes [`DynamicState::DepthBias`] and if subsequent drawing commands have depth bias enabled.
    /// - If [`Gpu::enabled_base_features`] `depth_bias_clamp` is set to `false`, `depth_bias_clamp` *must*
    /// be 0.0.
    /// # Vulkan docs link
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthBias.html>**
    pub fn set_depth_bias(
        &mut self,
        depth_bias_constant_factor: f32,
        depth_bias_clamp: f32,
        depth_bias_slope_factor: f32,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::DepthBias)?;
        if !self.gpu.enabled_base_features().depth_bias_clamp && depth_bias_clamp != 0.0 {
            return Err(Error::just_context(format_compact!(
                "depth bias clamp must be 0.0 if depth bias clamp base device feature is not enabled, given depth bias clamp was {}",
                depth_bias_clamp,
            )))
        }
        unsafe {
            self.gpu.vk().device().cmd_set_depth_bias(
                self.storage.command_buffer,
                depth_bias_constant_factor, 
                depth_bias_clamp,
                depth_bias_slope_factor
            );
        }
        Ok(())
    }

    /// Dynamically sets the values of blend constants.
    ///
    /// # Parameters
    /// - `blend_constants`: specifies the `[R, G, B, A]` color components of the blend constant
    /// color used in blending, depending on the [`BlendFactor`].
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic state
    /// includes [`DynamicState::BlendConstants`] and if subsequent drawing commands have blending
    /// enabled with blend functions using a constant blend constant.
    /// # Vulkan docs link
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetBlendConstants.html>**
    pub fn set_blend_constants(
        &self,
        blend_constants: [f32; 4],
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::BlendConstants)?;
        unsafe {
            self.gpu.vk().device().cmd_set_blend_constants(
                self.storage.command_buffer,
                &blend_constants
            );
        }
        Ok(())
    }

    /// Dynamically sets the depth bounds range.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic state
    /// includes [`DynamicState::DepthBounds`] and if subsequent drawing commands have depth
    /// bounds test enabled.
    /// - `min_depth_bounds` must be less than or equal to `max_depth_bounds` and both bounds need to be
    /// between `0.0` and `1.0` (inclusively).
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthBounds.html>**
    pub fn set_depth_bounds(
        &mut self,
        min_depth_bounds: f32,
        max_depth_bounds: f32,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::DepthBounds)?;
        if min_depth_bounds > max_depth_bounds {
            return Err(Error::just_context(format_compact!(
                "min depth bounds {min_depth_bounds} must be less than or equal to max depth bounds {max_depth_bounds}"
            )))
        }
        if min_depth_bounds.clamp(0.0, 1.0) != min_depth_bounds {
            return Err(Error::just_context(format_compact!(
                "min depth bounds {min_depth_bounds} must be inclusively between 0.0 and 1.0"
            )))
        }
        if max_depth_bounds.clamp(0.0, 1.0) != max_depth_bounds {
            return Err(Error::just_context(format_compact!(
                "max depth bounds {max_depth_bounds} must be inclusively between 0.0 and 1.0"
            )))
        }
        unsafe {
            self.gpu.vk().device().cmd_set_depth_bounds(
                self.storage.command_buffer,
                min_depth_bounds,
                max_depth_bounds
            );
        }
        Ok(())
    }

    /// Dynamically sets the stencil compare mask.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic
    /// state inludes [`DynamicState::StencilCompareMask`] and if subsequent drawing commands
    /// have stencil test enabled.
    /// - Both [`StencilFaceFlags::FRONT`] and [`StencilFaceFlags::BACK`] *must* be set (either together or
    /// separately) if stencil test is enabled.
    /// - `face_mask` *must* not be [`StencilFaceFlags::empty`] and *must* be a valid bitmask of
    /// [`StencilFaceFlags`] bits.
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetStencilCompareMask.html>**
    pub fn set_stencil_compare_mask(
        &mut self,
        face_mask: StencilFaceFlags,
        compare_mask: u32
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::StencilCompareMask)?;
        if face_mask.is_empty() {
            return Err(Error::just_context(
                "stencil face mask must not be empty"
            ))
        }
        unsafe {
            self.gpu.vk().device().cmd_set_stencil_compare_mask(
                self.storage.command_buffer,
                face_mask.into(),
                compare_mask
            );
        }
        Ok(())
    }
    
    /// Dynamically sets stencil write mask.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic
    /// state includes [`DynamicState::StencilWriteMask`] and if subsequent drawing commands have
    /// stencil test enabled.
    /// - Both [`StencilFaceFlags::FRONT`] and [`StencilFaceFlags::BACK`] *must* be set (either
    /// together or separately) if stencil test is enabled.
    /// - `face_mask` *must* not be [`StencilFaceFlags::empty`] and *must* be a valid bitmask of
    /// [`StencilFaceFlags`] bits.
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetStencilWriteMask.html>**
    pub fn set_stencil_write_mask(
        &mut self,
        face_mask: StencilFaceFlags,
        write_mask: u32,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::StencilWriteMask)?;
        if face_mask.is_empty() {
            return Err(Error::just_context(
                "stencil face mask must not be empty"
            ))
        }
        unsafe {
            self.gpu.vk().device().cmd_set_stencil_write_mask(
                self.storage.command_buffer,
                face_mask.into(),
                write_mask
            );
        }
        Ok(())
    }

    /// Dynamically sets stencil reference.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic
    /// state includes [`DynamicState::StencilReference`] and if subsequent drawing commands have
    /// stencil test enabled.
    /// - Both [`StencilFaceFlags::FRONT`] and [`StencilFaceFlags::BACK`] *must* be set (either
    /// together or separately) if stencil test is enabled.
    /// - `face_mask` *must* not be [`StencilFaceFlags::empty`] and *must* be a valid bitmask of
    /// [`StencilFaceFlags`] bits.
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetStencilReference.html>**
    pub fn set_stencil_reference(
        &mut self,
        face_mask: StencilFaceFlags,
        reference: u32,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::StencilReference)?;
        if face_mask.is_empty() {
            return Err(Error::just_context(
                "stencil face mask must not be empty"
            ))
        }
        unsafe {
            self.gpu.vk().device().cmd_set_stencil_reference(
                self.storage.command_buffer,
                face_mask.into(),
                reference
            );
        }
        Ok(())
    }

    /// Dynamically sets cull mode.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic
    /// state includes [`DynamicState::CullMode`] and if there are *any* subsequent drawing
    /// commands using the pipeline.
    /// - `cull_mode` *must* be a valid bitmask of [`CullModeFlags`] bits.
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetCullMode.html>**
    pub fn set_cull_mode(
        &mut self,
        cull_mode: CullModeFlags,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::CullMode)?;
        unsafe {
            self.gpu.vk().device().cmd_set_cull_mode(
                self.storage.command_buffer,
                vk::CullModeFlags::from_raw(cull_mode.as_raw()),
            );
        }
        Ok(())
    }

    /// Dynamically sets front face orientation.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic
    /// state includes [`DynamicState::FrontFace`] and if there are *any* subsequent drawing
    /// commands using the pipeline.
    /// - `front_face` *must* be a valid [`FrontFace`] value.
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetFrontFace.html>**
    pub fn set_front_face(
        &mut self,
        front_face: FrontFace,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::FrontFace)?;
        unsafe {
            self.gpu.vk().device().cmd_set_front_face(
                self.storage.command_buffer,
                vk::FrontFace::from_raw(front_face.as_raw()),
            );
        }
        Ok(())
    }

    /// Dynamically sets primitive topology.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic
    /// state includes [`DynamicState::PrimitiveTopology`] and if there are *any* subsequent drawing
    /// commands using the pipeline.
    /// - `primitive_topology` *must* be a valid [`PrimitiveTopology`] value.
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetPrimitiveTopology.html>**
    pub fn set_primitive_topology(
        &mut self,
        primitive_topology: PrimitiveTopology,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::PrimitiveTopology)?;
        unsafe {
            self.gpu.vk().device().cmd_set_primitive_topology(
                self.storage.command_buffer,
                primitive_topology.into(),
            );
        }
        Ok(())
    }

    /// Dynamically sets viewport.
    ///
    /// This is equivalent to (and more efficient than) calling
    /// [`DrawCommands::set_multi_viewport`] with just one viewport.
    ///
    /// This is automatically called when binding a pipeline with a viewport count of 1.
    ///
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetViewportWithCount.html>**
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetScissorWithCount.html>**
    pub fn set_viewport(
        &mut self,
        viewport: Viewport,
        scissor: Scissor,
    ) -> Result<()> {
        if self.last_pipeline.is_none() {
            return Err(Error::just_context(
                "attempting to set viewport with no pipeline bound"
            ))
        }
        let scissor = vk::Rect2D {
            offset: vk::Offset2D {
                x: scissor.x as i32,
                y: scissor.y as i32,
            },
            extent: vk::Extent2D {
                width: scissor.width,
                height: scissor.height,
            },
        };
        unsafe {
            self.gpu.vk().device().cmd_set_viewport_with_count(
                self.storage.command_buffer,
                &[viewport],
            );
            self.gpu.vk().device().cmd_set_scissor_with_count(
                self.storage.command_buffer,
                &[scissor],
            );
        }
        Ok(())
    }

    /// Dynamically sets multi viewport.
    ///
    /// This combines `vkCmdSetViewportWithCount` and `vkCmdSetScissorWithCount`.
    ///
    /// If you only have one viewport, consider using [`DrawCommands::set_viewport`] for
    /// efficiency.
    ///
    /// This is automatically called when binding a pipeline with more than one viewports.
    ///
    /// # Valid usage
    /// - The number of viewports and scissors *must* match.
    /// - If [`Gpu::enabled_base_features`] `multi_viewport` is set to `false`, the number of viewports
    /// *must* be 1. Otherwise the number of viewports *must* be between 1 and [`DeviceLimits::max_viewports`].
    /// You can get [`DeviceLimits`] with [`Gpu::device_limits`].
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetViewportWithCount.html>**
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetScissorWithCount.html>**
    pub fn set_multi_viewport(
        &mut self,
        viewports: &[Viewport],
        scissors: &[Scissor],
    ) -> Result<()> {
        if self.last_pipeline.is_none() {
            return Err(Error::just_context(
                "attempting to set multi viewport with no pipeline bound"
            ))
        }
        let n = viewports.len() as u32;
        if scissors.len() as u32 != n {
            return Err(Error::just_context(format_compact!(
                "the number of viewports {n} and scissors {} must match",
                scissors.len(),
            )))
        }
        if !self.gpu.enabled_base_features().multi_viewport && n != 1 {
            return Err(Error::just_context(format_compact!(
                "viewport count must be 1 if the multi viewport base device feature is not enabled, given view count is {n}"
            )))
        } else if n.clamp(1, self.gpu.device_limits().max_viewports()) != n {
            return Err(Error::just_context(format_compact!(
                "viewport count must be between inclusively between 1 and DeviceLimits::max_viewports() ({}), given view count is {n}",
                self.gpu.device_limits().max_viewports(),
            )))
        }
        let tmp_alloc = self.gpu.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let mut vk_scissors = NonNullVec32::with_capacity(n, &tmp_alloc)?;
        vk_scissors.append_map(scissors, |scissor| {
            vk::Rect2D {
                offset: vk::Offset2D {
                    x: scissor.x as i32,
                    y: scissor.y as i32,
                },
                extent: vk::Extent2D {
                    width: scissor.width,
                    height: scissor.height,
                },
            }
        });
        unsafe {
            self.gpu.vk().device().cmd_set_viewport_with_count(
                self.storage.command_buffer,
                viewports
            );
            self.gpu.vk().device().cmd_set_scissor_with_count(
                self.storage.command_buffer,
                &vk_scissors
            );
        }
        Ok(())
    }

    /// Dynamically enables depth test.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic
    /// state includes [`DynamicState::DepthTestEnable`] and if there are *any* subsequent drawing
    /// commands using the pipeline.
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthTestEnable.html>**
    pub fn set_depth_test_enable(
        &mut self,
        enabled: bool
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::DepthTestEnable)?;
        unsafe {
            self.gpu.vk().device().cmd_set_depth_test_enable(
                self.storage.command_buffer,
                enabled
            );
        }
        Ok(())
    }

    /// Dynamically enables depth write.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic
    /// state includes [`DynamicState::DepthWriteEnable`] and if there are *any* subsequent drawing
    /// commands using the pipeline.
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthWriteEnable.html>**
    pub fn set_depth_write_enable(
        &mut self,
        enabled: bool,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::DepthWriteEnable)?;
        unsafe {
            self.gpu.vk().device().cmd_set_depth_write_enable(
                self.storage.command_buffer,
                enabled
            );
        }
        Ok(())
    }

    /// Dynamically sets the depth compare operation.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic
    /// state includes [`DynamicState::DepthCompareOp`] and if there are *any* subsequent drawing
    /// commands using the pipeline.
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthCompareOp.html>**
    pub fn set_depth_compare_op(
        &mut self,
        compare_op: CompareOp,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::DepthCompareOp)?;
        unsafe {
            self.gpu.vk().device().cmd_set_depth_compare_op(
                self.storage.command_buffer,
                compare_op.into()
            );
        }
        Ok(())
    }

    /// Dynamically enables depth bounds test.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic
    /// state includes [`DynamicState::DepthBoundsTestEnable`] and if there are *any* subsequent drawing
    /// commands using the pipeline.
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetDepthBoundsTestEnable.html>**
    pub fn set_depth_bounds_test_enable(
        &mut self,
        enabled: bool,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::DepthBoundsTestEnable)?;
        unsafe {
            self.gpu.vk().device().cmd_set_depth_bounds_test_enable(
                self.storage.command_buffer,
                enabled
            );
        }
        Ok(())
    }

    /// Dynamically enables stencil test.
    ///
    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic
    /// state includes [`DynamicState::StencilTestEnable`] and if there are *any* subsequent drawing
    /// commands using the pipeline.
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetStencilTestEnable.html>**
    pub fn set_stencil_test_enable(
        &mut self,
        enabled: bool,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::StencilTestEnable)?;
        unsafe {
            self.gpu.vk().device().cmd_set_stencil_test_enable(
                self.storage.command_buffer,
                enabled
            );
        }
        Ok(())
    }

    /// Dynamically sets the stencil operation.
    ///    /// # Valid usage
    /// - This *must* be set if and only if the currently bound [`GraphicsPipeline`]'s dynamic
    /// state includes [`DynamicState::StencilOp`] and if there are *any* subsequent drawing
    /// commands using the pipeline with stencil test enabled.
    /// - Both [`StencilFaceFlags::FRONT`] and [`StencilFaceFlags::BACK`] *must* be set (either together or
    /// separately) if stencil test is enabled.
    /// - `face_mask` *must* not be [`StencilFaceFlags::empty`] and *must* be a valid bitmask of
    /// [`StencilFaceFlags`] bits.
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetStencilOp.html>**
    pub fn set_stencil_op(
        &mut self,
        face_mask: StencilFaceFlags,
        fail_op: StencilOp,
        pass_op: StencilOp,
        depth_fail_op: StencilOp,
        compare_op: CompareOp,
    ) -> Result<()> {
        self.check_dynamic_state(DynamicState::StencilOp)?;
        unsafe {
            self.gpu.vk().device().cmd_set_stencil_op(
                self.storage.command_buffer,
                face_mask.into(),
                fail_op.into(),
                pass_op.into(),
                depth_fail_op.into(),
                compare_op.into()
            );
        }
        Ok(())
    }

    pub fn bind_shader_resources<F>(
        &mut self,
        barrier_info: &[BindingBarrierInfo],
        mut f: F,
    ) -> Result<()>
        where
            F: FnMut(u32) -> Option<ShaderResourceId>,
    {
        let cache = self.storage.cache.deref_mut();
        let Some(pipeline) = cache.pipelines.last() else {
            return Err(Error::just_context("attempting to bind shader resources with no pipeline bound"))
        };
        let tmp_alloc = self.gpu.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let mut barriers = NonNullVec32
            ::with_capacity(barrier_info.len() as u32, &*self.storage.alloc)?
            .into_static();
        barriers.append(barrier_info);
        let mut bind_call = ShaderResourceBindCall {
            sets: NonNullVec32::with_capacity(
                pipeline.shader_set().descriptor_set_layouts().len() as u32,
                &*self.storage.alloc,
            )?.into_static(),
            barriers,
        };
        let sets = self.gpu.get_shader_set_resources(
            pipeline.shader_set(),
            &tmp_alloc,
            |index| {
                let id = f(index);
                bind_call.sets.push(id);
                id
            },
        )?;
        if sets.is_empty() {
            return Ok(())
        }
        cache.shader_resource_binds.push(bind_call);
        unsafe {
            self.gpu.vk().device().cmd_bind_descriptor_sets(
                self.storage.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.shader_set().pipeline_layout(),
                0, &sets, &[]
            );
        }
        Ok(())
    }

    pub fn push_constants<'c, F>(
        &self,
        f: F,
    ) -> Result<()>
        where
            F: FnMut(PushConstantRange) -> &'c [u8],
    {
        let Some(pipeline) = self.storage.cache.pipelines.last() else {
            return Err(Error::just_context("attempting to push constants with no pipeline bound"))
        };
        let tmp_alloc = self.gpu.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let push_constants = self.gpu.get_shader_set_push_constant_ranges(
            pipeline.shader_set(),
            &tmp_alloc,
            f
        )?;
        for (pc, bytes) in &push_constants {
            unsafe {
                self.gpu.vk().device().cmd_push_constants(
                    self.storage.command_buffer,
                    pipeline.shader_set().pipeline_layout(),
                    pc.stage.into(),
                    pc.offset,
                    bytes,
                );
            }
        }
        Ok(())
    }
   
    /// Binds index buffer and vertex buffers and performs an indexed draw call.
    ///
    /// # Valid usage
    /// - A [`GraphicsPipeline`] needs to be bound.
    /// - The index type of `draw_info` *must* be a valid [`IndexType`] value.
    /// - If the index type of `draw_info` is [`IndexType::U8`], the [`index_type_uint8`] device
    /// extension *must* be enabled.
    /// - Index buffer id *must* be a valid [`BufferId`].
    /// - The index buffer *must* have been created with [`BufferUsageFlags::INDEX_BUFFER`] bit
    /// set.
    /// - Index buffer offset + [`IndexType::index_size`] * (first_index + index_count) *must* be
    /// less than or equal to the index buffer's size.
    /// - Each vertex binding's buffer id *must* be  a valid [`BufferId`].
    /// - Each vertex buffer *must* have been created with [`BufferUsageFlags::VERTEX_BUFFER`] bit
    /// set.
    /// - Each vertex binding's offset + size *must* be less than or equal to the buffer's size.
    /// - If and only if currently bound [`GraphicsPipeline`]'s dynamic state includes
    /// [`DynamicState::VERTEX_INPUT_BINDING_STRIDE`], `vertex_strides` needs to be [`Some`] and
    /// the number of vertex strides *must* be equal to the number of vertex bindings.
    /// - If [`ext::robustness2`] robust buffer access 2 or [`Gpu::enabled_base_features`] robust
    /// buffer access are not enabled, and the graphics pipeline was not created with
    /// [`PipelineRobustnessInfo`] vertex input behavior with a value other than
    /// [`PipelineRobustnessBufferBehavior::DISABLED`], then for a given vertex buffer binding, any
    /// attribute data fetched *must* be entirely contained within the corresponding vertex buffer
    /// binding.
    ///
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBindVertexBuffers2.html>**
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdBindIndexBuffer2.html>**
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDrawIndexed.html>**
    pub fn draw_indexed(
        &mut self,
        draw_info: IndexedDrawInfo,
        index_buffer: IndexBufferInfo,
        vertex_bindings: &[DrawBufferRange],
        vertex_strides: Option<&[DeviceSize]>,
    ) -> Result<()>
    {
        let cache = self.storage.cache.deref_mut();
        if cache.pipelines.is_empty() {
            return Err(Error::just_context("attempting to draw with no pipeline bound"))
        }
        self.storage.wait_scope = vk::PipelineStageFlags2::VERTEX_INPUT;
        let index_buf_size = (draw_info.first_index + draw_info.index_count) as vk::DeviceSize * draw_info.index_type.index_size();
        let n_bindings = vertex_bindings.len() as u32;
        let mut call = DrawCall {
            index_buffer: Some(DrawBufferRange::new(
                index_buffer.id,
                index_buffer.offset,
                index_buf_size,
            )),
            vertex_buffers: NonNullVec32::with_capacity(
                n_bindings,
                &*self.storage.alloc,
            )?,
        };
        unsafe {
            let command_buffer = self.storage.command_buffer;
            let index_buf = self.buffers.get(index_buffer.id)?;
            if index_buffer.offset + index_buf_size > index_buf.properties().size
            {
                return Err(Error::new(
                    BufferError::OutOfRange {
                        buffer_size: index_buf.properties().size,
                        requested_offset: index_buffer.offset,
                        requested_size: index_buf_size,
                    },
                    "given buffer size and offset are out of range of index buffer",
                ))
            }
            if let Some(err) = index_buf.validate_usage(vk::BufferUsageFlags::INDEX_BUFFER) {
                return Err(Error::new(err, "index buffer has incompatible usage"))
            }
            let tmp_alloc = self.gpu.tmp_alloc();
            let tmp_alloc = tmp_alloc.guard();
            let mut vert_bufs = NonNullVec32::with_capacity(n_bindings, &tmp_alloc)?;
            let mut vert_offsets = NonNullVec32::with_capacity(n_bindings, &tmp_alloc)?;
            let mut vert_sizes = NonNullVec32::with_capacity(n_bindings, &tmp_alloc)?;
            for buf_info in vertex_bindings.iter().copied() {
                let buf = self.buffers.get(buf_info.id)?;
                if buf_info.offset + buf_info.size > buf.properties().size {
                    return Err(Error::new(
                        BufferError::OutOfRange {
                            buffer_size: buf.properties().size,
                            requested_offset: buf_info.offset, requested_size: buf_info.size,
                        },
                        "given buffer size and offset are out of range of vertex buffer",
                    ))
                }
                if let Some(err) = buf.validate_usage(vk::BufferUsageFlags::VERTEX_BUFFER) {
                    return Err(Error::new(err, format_compact!("vertex buffer has incompatible usage")))
                }
                vert_bufs.push(buf.handle());
                vert_offsets.push(buf_info.offset);
                vert_sizes.push(buf_info.size);
                call.vertex_buffers.push(buf_info);
            }
            if let Some(strides) = vertex_strides {
                self.check_dynamic_state(DynamicState::VERTEX_INPUT_BINDING_STRIDE)?;
                if strides.len() as u32 != n_bindings {
                    return Err(Error::just_context(
                        "the number of vertex strides must match the number of vertex bindings"
                    ))
                }
            }
            let vk = self.gpu.vk();
            vk.device().cmd_bind_vertex_buffers2(
                command_buffer,
                0,
                &vert_bufs, &vert_offsets, Some(&vert_sizes),
                vertex_strides,
            );
            vk.device().cmd_bind_index_buffer2(
                command_buffer, index_buf.handle(),
                index_buffer.offset, index_buf_size, draw_info.index_type.into()
            );
            vk.device().cmd_draw_indexed(
                command_buffer,
                draw_info.index_count, draw_info.instance_count,
                draw_info.first_index, draw_info.vertex_offset,
                draw_info.first_instance,
            );
            vert_bufs.drop_and_free(&tmp_alloc);
            vert_offsets.drop_and_free(&tmp_alloc);
            vert_sizes.drop_and_free(&tmp_alloc);
        }
        cache.draw_calls.push(call);
        Ok(())
    }

    pub fn draw<const VERTEX_BINDING_COUNT: usize>(
        &mut self,
        first_vertex: u32,
        vertex_count: u32,
        first_instance: u32,
        instance_count: u32,
        vertex_bindings: [VertexBufferInfo; VERTEX_BINDING_COUNT],
    ) -> Result<()>
    {
        let cache = self.storage.cache.deref_mut();
        if cache.pipelines.is_empty() {
            return Err(Error::just_context("attempting to draw with no pipeline bound"))
        };
        self.storage.wait_scope = vk::PipelineStageFlags2::VERTEX_INPUT;
        let mut call = DrawCall {
            index_buffer: None,
            vertex_buffers: NonNullVec32::with_capacity(
                VERTEX_BINDING_COUNT as u32,
                &*self.storage.alloc,
            )?,
        };
        unsafe {
            let mut vert = ArrayVec::<vk::Buffer, VERTEX_BINDING_COUNT>::new();
            let mut vert_offsets = ArrayVec::<vk::DeviceSize, VERTEX_BINDING_COUNT>::new();
            let mut vert_sizes = ArrayVec::<vk::DeviceSize, VERTEX_BINDING_COUNT>::new();
            for buf_info in vertex_bindings.iter().copied() {
                let buf = self.buffers.get(buf_info.id)?;
                if buf_info.offset + buf_info.size > buf.properties().size {
                    return Err(Error::new(
                        BufferError::OutOfRange {
                            buffer_size: buf.properties().size,
                            requested_offset: buf_info.size, requested_size: buf_info.size,
                        },
                        format_compact!("given buffer size and offset are out of range of vertex buffer"),
                    ))
                }
                if let Some(err) = buf.validate_usage(vk::BufferUsageFlags::VERTEX_BUFFER) {
                    return Err(Error::new(err, format_compact!("vertex buffer has incompatible usage")))
                }
                vert.push(buf.handle());
                vert_offsets.push(buf_info.offset);
                vert_sizes.push(buf_info.size);
                call.vertex_buffers.push(buf_info);
            }
            let command_buffer = self.storage.command_buffer;
            let vk = self.gpu.vk();
            vk.device().cmd_bind_vertex_buffers2(
                command_buffer,
                0,
                &vert, &vert_offsets,
                Some(&vert_sizes), None
            );
            vk.device().cmd_draw(
                command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance
            );
        }
        cache.draw_calls.push(call);
        Ok(())
    }

    #[inline(always)]
    pub fn draw_bufferless(&self, vertex_count: u32, instance_count: u32) -> Result<()> {
        if self.storage.cache.pipelines.is_empty() {
            return Err(Error::just_context("attempting to draw with no pipeline bound"))
        };
        unsafe {
            self.gpu.vk().device().cmd_draw(self.storage.command_buffer, vertex_count, instance_count, 0, 0);
        }
        Ok(())
    }
}
