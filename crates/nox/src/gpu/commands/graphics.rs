mod structs;
mod cache;
mod draw;

use core::cell::UnsafeCell;

use nox_ash::vk;
use compact_str::format_compact;
use ahash::{AHashMap, AHashSet};

use nox_mem::{
    AsRaw,
    alloc::LocalAlloc,
    vec::{NonNullVec32, FixedVec32, Vec32, Vector},
    borrow::SizedCowMut,
    option::OptionExt,
    conditional::True,
};
use nox_alloc::arena::{Arena, Guard};

use crate::dev::error::*;
use crate::gpu::prelude::*;
use crate::misc::ToRef;

use super::{scheduler, prelude::CommandResult};

pub use structs::*;
pub use draw::*;

#[derive(Default, Clone)]
pub struct RenderPassTemplate {
    color_attachments: Vec32<PassAttachment<'static>>,
    depth_stencil_attachment: DepthStencilAttachment<'static>,
    render_area: Option<RenderArea>,
    msaa_samples: MsaaSamples,
    layer_count: u32,
}

impl RenderPassTemplate {

    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn with_color_attachment(mut self, attachment: PassAttachment<'static>) -> Self
    {
        self.color_attachments.push(attachment);
        self
    }

    #[inline(always)]
    pub fn with_depth_stencil_attachment(
        mut self,
        attachment: DepthStencilAttachment<'static>,
    ) -> Self
    {
        self.depth_stencil_attachment = attachment;
        self
    }

    #[inline(always)]
    pub fn with_render_area(mut self, area: RenderArea) -> Self {
        self.render_area = Some(area);
        self
    }

    #[inline(always)]
    pub fn with_msaa_samples(mut self, samples: MSAA) -> Self {
        self.msaa_samples = samples;
        self
    }

    #[inline(always)]
    pub fn with_layer_count(mut self, count: u32) -> Self {
        self.layer_count = count;
        self
    }
}

pub struct GraphicsCommands<'a, 'b> {
    gpu: Gpu,
    recorder: &'a mut CommandRecorder<'b>,
    primary_command_buffer: vk::CommandBuffer,
    wait_scope: vk::PipelineStageFlags2,
    signal_scope: vk::PipelineStageFlags2,
    command_id: CommandId,
    first_timeline_value: u64,
    command_timeline_value: u64,
}

pub struct ActiveRenderPass<'a, 'b, 'c, Alloc>
    where
        Alloc: Guard<True>,
{
    cmd: &'a mut GraphicsCommands<'b, 'c>,
    current_pipeline_id: Option<GraphicsPipelineId>,
    sample_count: MSAA,
    color_formats: NonNullVec32<'static, vk::Format, True>,
    depth_format: vk::Format,
    stencil_format: vk::Format,
    tmp_alloc: Alloc,
}

impl<'a, 'b> GraphicsCommands<'a, 'b> {

    #[inline(always)]
    pub(crate) unsafe fn new(
        recorder: &'a mut CommandRecorder<'b>,
        command_id: CommandId,
        command_timeline_value: u64,
    ) -> Result<Self>
    {
        let command_buffer = recorder
            .get_current_worker()
            .allocate_primaries(scheduler::CommandBufferKind::Graphics, 1)?[0];
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };
        let gpu = recorder.gpu().clone();
        unsafe {
            gpu.vk().device()
                .begin_command_buffer(command_buffer, &begin_info)
                .context("failed to begin primary command buffer")?;
        }
        Ok(Self {
            gpu,
            recorder,
            primary_command_buffer: command_buffer,
            wait_scope: vk::PipelineStageFlags2::NONE,
            signal_scope: vk::PipelineStageFlags2::NONE,
            command_id,
            first_timeline_value: command_timeline_value,
            command_timeline_value,
        })
    }

    pub fn begin_rendering<F>(
        &mut self,
        template: &RenderPassTemplate,
    ) -> Result<ActiveRenderPass<'_, 'a, 'b, impl Guard<True>>>
    {
        self.command_timeline_value += 1;
        let command_timeline_value = self.command_timeline_value;
        let command_id = self.command_id;
        let command_buffer = self.primary_command_buffer;
        let tmp_alloc = self.gpu.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let msaa_samples = template.msaa_samples;
        let mut vk_color_attachments = NonNullVec32
            ::with_capacity(template.color_attachments.len() as u32, &tmp_alloc)?;
        let queue_family_index = self.gpu.vk().queue_family_indices().graphics_index(); 
        let mut min_extent = vk::Extent2D {
            width: u32::MAX,
            height: u32::MAX,
        };
        let mut min_extent = vk::Extent2D { width: u32::MAX, height: u32::MAX, };
        let mut process_attachment = |
            attachment: &PassAttachment,
            dst_state: ImageSubresourceState,
            resolve_aspect: ResolveAspect,
            usage: vk::ImageUsageFlags,
        |
        {
            let image = self.recorder.register_image(attachment.image_id)?;
            if let Some(err) = image.validate_usage(usage) {
                return Err(Error::new(err, format_compact!(
                    "image {} can't be used as a {} attachment",
                    attachment.image_id, resolve_aspect,
                )))
            }
            let handle = image.handle();
            let properties = image.properties();
            let format = properties.format;
            if msaa_samples != properties.samples {
                return Err(Error::just_context(format_compact!(
                "image sample counts must match, image (id {}) had sample count {} while pass sample count was {}",
                    attachment.image_id, properties.samples, msaa_samples,
                )))
            }
            let dimensions = properties.dimensions;
            min_extent.width = min_extent.width.min(dimensions.width);
            min_extent.height = min_extent.height.min(dimensions.height);
            let mut layer_count = properties.array_layers;
            let view = if let Some(range) = attachment.range {
                layer_count = range.subresource.layer_count.get();
                if range.subresource.level_count.get() != 1 {
                    return Err(Error::just_context(format_compact!(
                        "image (id: {}) subresource range mip levels {} must be 1 when used as a rendering attachment",
                        attachment.image_id, range.subresource.level_count,
                    )))
                }
                image.get_subview(range).context_with(|| format_compact!(
                    "failed to get image (id: {}) subview", attachment.image_id,
                ))?
            } else {
                if properties.mip_levels > 1 {
                    let mut subresource = properties.whole_subresource();
                    subresource.level_count = 1.into();
                    image.get_subview(ImageRange::new(subresource, None)).context_with(|| format_compact!(
                        "failed to get image (id: {}) subview", attachment.image_id,
                    ))?
                } else {
                    image.get_view()
                }
            };
            if layer_count < pass.layer_count {
                return Err(Error::just_context(format_compact!(
                    "image subresource layer count {} was less than pass layer count {}",
                    layer_count, pass.layer_count,
                )))
            }
            let memory_barriers = image.memory_barrier(
                dst_state,
                attachment.range.map(|range| range.subresource),
                attachment.preserve_contents,
                &mut self.recorder.cache.graphics_command_cache.image_memory_barrier_cache,
            ).context_with(|| format_compact!(
                "image (id: {}) memory barrier failed",
                attachment.image_id,
            ))?;
            unsafe {
                scheduler::cmd_pipeline_barrier(
                    self.recorder.gpu().vk(),
                    command_buffer,
                    &[],
                    &[(handle, &memory_barriers)],
                    command_id.index(),
                    command_timeline_value,
                    &tmp_alloc,
                ).context_with(|| format_compact!(
                    "image (id: {}) memory barrier failed", attachment.image_id,
                ))?;
            }
            let mut attachment_info = vk::RenderingAttachmentInfo {
                s_type: vk::StructureType::RENDERING_ATTACHMENT_INFO,
                image_view: view,
                image_layout: dst_state.layout,
                load_op: attachment.load_op.into(),
                store_op: attachment.store_op.into(),
                clear_value: attachment.clear_value.into(),
                ..Default::default()
            };
            if let Some(resolve) = attachment.resolve {
                let image = self.recorder.register_image(resolve.image_id)?;
                let properties = image.properties();
                if properties.format != format {
                    return Err(Error::just_context(format_compact!(
                        "resolve image (id: {}) format {:?} must match the source format {:?} (this may change in the future)",
                        resolve.image_id, properties.format, format,
                    )))
                }
                if properties.dimensions != dimensions {
                    return Err(Error::just_context(format_compact!(
                        "resolve image (id: {}) dimensions {} must match source dimensions {}",
                        resolve.image_id, properties.dimensions, dimensions,
                    )))
                }
                if properties.samples != MSAA::X1 {
                    return Err(Error::just_context(format_compact!(
                        "resolve image (id: {}) sample count must be one but given sample count was {}",
                        resolve.image_id, properties.samples,
                    )))
                }
                let mode_bit = resolve.mode.as_raw();
                if properties.format_resolve_modes.resolve_modes(resolve_aspect).as_raw() & mode_bit != mode_bit {
                    return Err(Error::just_context(format_compact!(
                        "image format {:?} doesn't support resolve mode {} for resolve aspect {}",
                        format, resolve.mode, resolve_aspect,
                    )))
                }
                let view = if let Some(range) = resolve.range {
                    if range.subresource.level_count.get() > 1 {
                        return Err(Error::just_context(format_compact!(
                            "resolve image (id: {}) subresource range mip levels {} must be 1 when used as a rendering attachment",
                            resolve.image_id, range.subresource.level_count,
                        )))
                    }
                    if range.subresource.layer_count.get() != layer_count {
                        return Err(Error::just_context(format_compact!(
                            "resolve image (id: {}) subresource layer count {} must match source subresource layer count {}",
                            resolve.image_id, range.subresource.layer_count, layer_count,
                        )))
                    }
                    image.get_subview(range).context_with(|| format_compact!(
                        "failed to get image (id: {}) subview", resolve.image_id,
                    ))?
                } else {
                    if properties.array_layers != layer_count {
                        return Err(Error::just_context(format_compact!(
                            "resolve image (id: {}) layer count {} must match source layer count {} if no image range was provided",
                            resolve.image_id, properties.array_layers, layer_count,
                        )))
                    }
                    if properties.mip_levels > 1 {
                        let mut subresource = properties.whole_subresource();
                        subresource.level_count = 1.into();
                        image.get_subview(ImageRange::new(subresource, None)).context_with(|| format_compact!(
                            "failed to get image (id: {}) subview", attachment.image_id,
                        ))?
                    } else {
                        image.get_view()
                    }
                };
                let memory_barriers = image.memory_barrier(
                    dst_state,
                    resolve.range.map(|range| range.subresource),
                    attachment.preserve_contents,
                    &mut self.recorder.cache.graphics_command_cache.image_memory_barrier_cache
                ).context_with(|| format_compact!(
                    "image (id: {}) memory barrier failed",
                    resolve.image_id,
                ))?;
                unsafe {
                    scheduler::cmd_pipeline_barrier(
                        self.recorder.gpu().vk(),
                        command_buffer,
                        &[],
                        &[(handle, &memory_barriers)],
                        command_id.index(),
                        command_timeline_value,
                        &tmp_alloc,
                    ).context_with(|| format_compact!(
                        "image (id: {}) memory barrier failed", attachment.image_id,
                    ))?;
                }
                attachment_info.resolve_image_view = view;
                attachment_info.resolve_image_layout = dst_state.layout;
                attachment_info.resolve_mode = resolve.mode.into();
            }
            Ok((attachment_info, properties.format))
        };
        let color_attachment_state = ImageSubresourceState {
            stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            queue_family_index,
            command_index: command_id.index(),
            command_timeline_value,
        };
        let mut color_formats = unsafe { NonNullVec32
            ::with_capacity(pass.color_attachments.len(), &tmp_alloc)?
            .into_static()
            .into_clonable()
        };
        for attachment in &pass.color_attachments { 
            let (attachment, format) = process_attachment(
                attachment, color_attachment_state,
                ResolveAspect::Color,
                vk::ImageUsageFlags::COLOR_ATTACHMENT,
            )?;
            color_formats.push(format);
            vk_color_attachments.push(attachment);
        }
        let mut depth_attachment = None;
        let mut depth_format = vk::Format::UNDEFINED;
        let mut stencil_attachment = None;
        let mut stencil_format = vk::Format::UNDEFINED;
        match pass.depth_stencil_attachment {
            DepthStencilAttachment::None => {},
            DepthStencilAttachment::Depth(attachment) => {
                let (depth, format) = process_attachment(
                    &attachment,
                    ImageSubresourceState {
                        stage_mask:
                            vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS | 
                            vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                        access_mask:
                            vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ |
                            vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        layout: vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                        queue_family_index,
                        command_index: command_id.index(),
                        command_timeline_value
                    },
                    ResolveAspect::Depth,
                    vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                )?;
                depth_attachment = Some(depth);
            },
            DepthStencilAttachment::Stencil(attachment) => {
                let (stencil, format) = process_attachment(
                    &attachment,
                    ImageSubresourceState {
                        stage_mask:
                            vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS |
                            vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                        access_mask:
                            vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ |
                            vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        layout: vk::ImageLayout::STENCIL_ATTACHMENT_OPTIMAL,
                        queue_family_index,
                        command_index: command_id.index(),
                        command_timeline_value
                    },
                    ResolveAspect::Stencil,
                    vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                )?;
                stencil_format = format;
                stencil_attachment = Some(stencil);
            },
            DepthStencilAttachment::DepthStencil { depth, stencil } => {
                let (depth_layout, stencil_layout) = if depth.image_id == stencil.image_id {
                    (vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                } else {
                    (vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL, vk::ImageLayout::STENCIL_ATTACHMENT_OPTIMAL)
                };
                let (depth, format) = process_attachment(
                    &depth,
                    ImageSubresourceState {
                        stage_mask:
                            vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS | 
                            vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                        access_mask:
                            vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ |
                            vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        layout: depth_layout,
                        queue_family_index,
                        command_index: command_id.index(),
                        command_timeline_value
                    },
                    ResolveAspect::Depth,
                    vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                )?;
                depth_attachment = Some(depth);
                depth_format = format;
                let (stencil, format) = process_attachment(
                    &stencil,
                    ImageSubresourceState {
                        stage_mask:
                            vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS | 
                            vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                        access_mask:
                            vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ |
                            vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        layout: stencil_layout,
                        queue_family_index,
                        command_index: command_id.index(),
                        command_timeline_value
                    },
                    ResolveAspect::Stencil,
                    vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                )?;
                stencil_attachment = Some(stencil);
                stencil_format = format;
            },
        }
        let mut rendering_info = vk::RenderingInfo {
            s_type: vk::StructureType::RENDERING_INFO,
            flags: vk::RenderingFlags::CONTENTS_SECONDARY_COMMAND_BUFFERS,
            render_area: pass.render_area
                .unwrap_or(RenderArea {
                    width: min_extent.width,
                    height: min_extent.height,
                    ..Default::default()
                }).into(),
            layer_count: pass.layer_count,
            color_attachment_count: vk_color_attachments.len(),
            p_color_attachments: vk_color_attachments.as_ptr(),
            p_depth_attachment: depth_attachment.as_ref().as_ptr(),
            p_stencil_attachment: stencil_attachment.as_ref().as_ptr(),
            ..Default::default()
        };
        if !pass.color_attachments.is_empty() {
            self.signal_scope |= vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT;
        } else if
            !matches!(pass.depth_stencil_attachment, DepthStencilAttachment::None)
        {
            self.signal_scope |= vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS;
        }
        unsafe {
            self.gpu
                .vk()
                .device()
                .cmd_begin_rendering(command_buffer, &rendering_info);
            vk_color_attachments.drop_and_free(&tmp_alloc);
        }
        self.wait_scope |= vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT;
        Ok(ActiveRenderPass {
            cmd: self,
            current_pipeline_id: None,
            sample_count: msaa_samples,
            color_formats,
            depth_format,
            stencil_format,
            tmp_alloc,
        })
    }
    
    unsafe fn execute_draw_commands(
        &mut self,
        storage: &DrawCommandStorage,
    ) -> Result<()>
    {
        self.wait_scope |= storage.wait_scope;
        let tmp_alloc = self.gpu.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let pools = self.gpu.get_shader_resource_pools();
        let mut write_cache = FixedVec32::with_len_with(pools.capacity(), |_| None, &tmp_alloc)?;
        let command_id = self.command_id;
        let command_timeline_value = self.command_timeline_value;
        let primary_command_buffer = self.primary_command_buffer;
        let queue_family_index = self.gpu.vk().queue_family_indices().graphics_index();
        let cmd_cache = unsafe {
            self.recorder.cache().as_mut()
        };
        let cmd_cache = &mut cmd_cache.graphics_command_cache;
        let mut all_shader_stages = vk::ShaderStageFlags::empty();
        let current_frame = self.recorder.current_frame();
        for call in &storage.cache.shader_resource_binds {
            let mut barrier_info = FixedVec32::with_capacity(call.barriers.len(), &tmp_alloc)?;
            for (i, &resource) in call.sets.iter().enumerate() {
                let Some(resource) = resource else {
                    continue;
                };
                let set = i as u32;
                barrier_info.clear();
                barrier_info.extend(call.barriers.iter().copied().filter(|barrier| barrier.set == set));
                let pool_id = resource.pool_id();
                let pool = write_cache
                    .get_mut(pool_id.slot_index().index() as usize)
                    .ok_or_else(|| Error::just_context(format_compact!("invalid pool id {pool_id}")))?
                    .get_or_try_insert_with(|| {
                        Ok(pools.get(pool_id.slot_index())
                            .context_with(|| format_compact!(
                                "failed to find pool {pool_id}",
                            ))?.write())
                    })?;
                let mut resource = pool.get_shader_resource_for_submit(
                    resource,
                    current_frame,
                )?;
                let shader_stage_mask = resource.shader_stage_mask();
                let mut pipeline_stage = vk::PipelineStageFlags2::NONE;
                if shader_stage_mask.contains(vk::ShaderStageFlags::VERTEX) {
                    pipeline_stage |= vk::PipelineStageFlags2::VERTEX_SHADER;
                }
                if shader_stage_mask.contains(vk::ShaderStageFlags::FRAGMENT) {
                    pipeline_stage |= vk::PipelineStageFlags2::FRAGMENT_SHADER;
                }
                all_shader_stages |= shader_stage_mask;
                for binding in resource.binding_iter() {
                    let (ordering, access, explicit_access) = barrier_info
                        .iter()
                        .find_map(|bar| (bar.binding == binding.binding())
                            .then_some((bar.ordering, bar.access, Some(bar.access)))
                        ).unwrap_or((CommandOrdering::None, ShaderAccess::ReadWrite, None));
                    for buffer in binding.descriptor_buffers() {
                        if let Some((id, offset, size)) = buffer.buffer {
                            let cache = cmd_cache.buffer_cache.entry(id).or_default();
                            if cache.touch(
                                offset, size,
                                pipeline_stage,
                                vk::AccessFlags2::from_raw(access as u64),
                                ordering,
                            ) {
                                cmd_cache.buffer_id_cache.push(id);
                            }
                        }
                    }
                    for image in binding.descriptor_images() {
                        if let Some((id, layout, range)) = image.image {
                            let cache = cmd_cache.image_cache
                                .entry(id)
                                .or_default();
                            if cache.touch(layout, explicit_access, shader_stage_mask) {
                                cmd_cache.image_id_cache.push(id);
                            }
                            cache.subresource_ranges.insert(range);
                        }
                    }
                }
            }
        }
        for call in &storage.cache.draw_calls {
            if let Some(index_buffer) = call.index_buffer {
                let cache = cmd_cache.buffer_cache
                    .entry(index_buffer.id)
                    .or_default();
                if cache.touch(
                    index_buffer.offset,
                    index_buffer.size,
                    vk::PipelineStageFlags2::INDEX_INPUT,
                    vk::AccessFlags2::INDEX_READ,
                    CommandOrdering::None,
                ) {
                    cmd_cache.buffer_id_cache.push(index_buffer.id);
                }
            }
            for vertex_buffer in &call.vertex_buffers {
                let cache = cmd_cache.buffer_cache
                    .entry(vertex_buffer.id)
                    .or_default();
                if cache.touch(
                    vertex_buffer.offset,
                    vertex_buffer.size,
                    vk::PipelineStageFlags2::VERTEX_ATTRIBUTE_INPUT,
                    vk::AccessFlags2::VERTEX_ATTRIBUTE_READ,
                    CommandOrdering::None,
                ) {
                    cmd_cache.buffer_id_cache.push(vertex_buffer.id);
                }
            }
        }
        if !storage.cache.draw_calls.is_empty() {
            self.wait_scope |= vk::PipelineStageFlags2::VERTEX_INPUT;
        }
        else if all_shader_stages.contains(vk::ShaderStageFlags::VERTEX) {
            self.wait_scope |= vk::PipelineStageFlags2::VERTEX_SHADER;
        }
        else if all_shader_stages.contains(vk::ShaderStageFlags::FRAGMENT) {
            self.wait_scope |= vk::PipelineStageFlags2::FRAGMENT_SHADER;
        }
        for &id in &cmd_cache.buffer_id_cache {
            let buffer = self.recorder.register_buffer(id)?;
            let cache = cmd_cache.buffer_cache.get_mut(&id).unwrap();
            let handle = buffer.handle();
            let mut state = BufferState {
                access_mask: vk::AccessFlags2::NONE,
                stage_mask: vk::PipelineStageFlags2::NONE,
                queue_family_index,
                command_index: command_id.index(),
                command_timeline_value,
            };
            for &(offset, size, stage_mask, access_mask, ordering) in &cache.ranges {
                state.stage_mask = stage_mask;
                state.access_mask = access_mask;
                let memory_barriers = buffer.memory_barrier(
                    offset, size, state,
                    ordering, &mut cmd_cache.buffer_memory_barrier_cache,
                ).context_with(|| format_compact!(
                    "memory barrier for buffer with id {id} failed",
                ))?;
                unsafe {
                    scheduler::cmd_pipeline_barrier(
                        self.gpu.vk(),
                        primary_command_buffer,
                        &[(handle, memory_barriers)],
                        &[],
                        command_id.index(),
                        command_timeline_value,
                        &tmp_alloc,
                    ).context_with(|| format_compact!(
                        "memory barrier for buffer with id {id} failed"
                    ))?;
                }
            }
            cache.reset();
        }
        for &id in &cmd_cache.image_id_cache {
            let image = self.recorder.register_image(id)?;
            let cache = cmd_cache.image_cache.get_mut(&id).unwrap();
            let layout = cache.layout.unwrap();
            let mut stage_mask = vk::PipelineStageFlags2::NONE;
            if cache.shader_stage_mask.contains(vk::ShaderStageFlags::VERTEX) {
                stage_mask |= vk::PipelineStageFlags2::VERTEX_SHADER;
            }
            if cache.shader_stage_mask.contains(vk::ShaderStageFlags::FRAGMENT) {
                stage_mask |= vk::PipelineStageFlags2::FRAGMENT_SHADER;
            }
            let state = ImageSubresourceState {
                stage_mask,
                access_mask: cache.access
                    .map(|access| vk::AccessFlags2::from_raw(access as u64))
                    .unwrap_or(layout.access_mask()),
                layout: layout.into(),
                queue_family_index,
                command_index: command_id.index(),
                command_timeline_value,
            };
            let handle = image.handle();
            for &range in &cache.subresource_ranges {
                let memory_barriers = image
                    .memory_barrier(state, range, true, &mut cmd_cache.image_memory_barrier_cache)
                    .context_with(|| format_compact!(
                        "memory barrier for image with id {id} failed",
                    ))?;
                    unsafe {
                        scheduler::cmd_pipeline_barrier(
                            self.gpu.vk(),
                            primary_command_buffer,
                            &[], &[(handle, memory_barriers)],
                            command_id.index(),
                            command_timeline_value,
                            &tmp_alloc,
                        ).context_with(|| format_compact!(
                            "memory barrier for image with id {id} failed"
                        ))?;
                    }
            }
            cache.reset();
        }
        cmd_cache.image_id_cache.clear();
        cmd_cache.buffer_id_cache.clear();
        unsafe {
            self.gpu.vk().device()
            .cmd_execute_commands(primary_command_buffer, &[storage.command_buffer]);
        }
        Ok(())
    }

    pub(super) fn finish<'c, Alloc>(self, alloc: &'c Alloc) -> Result<CommandResult<'c>> 
        where Alloc: LocalAlloc<Error = Error>
    {
        unsafe {
            self.gpu.vk().device()
                .end_command_buffer(self.primary_command_buffer)
                .context("failed to end command buffer")?;
        }
        let mut primary_command_buffers = NonNullVec32::with_capacity(1, alloc)?;
        primary_command_buffers.push(self.primary_command_buffer);
        Ok(CommandResult {
            primary_command_buffers,
            timeline_value: self.command_timeline_value,
            wait_scope: self.wait_scope,
            signal_scope: self.signal_scope,
            queue: self.gpu.vk().graphics_queue(),
        })
    }
}

impl<'a, 'b, 'c, Alloc> ActiveRenderPass<'a, 'b, 'c, Alloc>
    where Alloc: Guard<True>
{

    pub fn render_dynamic<F>(
        &mut self,
        f: F,
    ) -> Result<()>
        where F: FnOnce(&mut DrawCommands) -> Result<()>
    {
        let command_buffer = self.cmd.recorder
            .get_current_worker()
            .allocate_secondaries(scheduler::CommandBufferKind::Graphics, 1)?[0];
        let mut rendering_inheritance_info = vk::CommandBufferInheritanceRenderingInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_INHERITANCE_RENDERING_INFO,
            color_attachment_count: self.color_formats.len(),
            p_color_attachment_formats: self.color_formats.as_ptr(),
            depth_attachment_format: self.depth_format,
            stencil_attachment_format: self.stencil_format,
            rasterization_samples: self.sample_count.into(),
            ..Default::default()
        };
        let inheritance_info = vk::CommandBufferInheritanceInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_INHERITANCE_INFO,
            ..Default::default()
        }.push_next(&mut rendering_inheritance_info);
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            p_inheritance_info: &inheritance_info,
            ..Default::default()
        };
        unsafe {
            self.cmd.gpu.vk().device()
            .begin_command_buffer(command_buffer, &begin_info)
            .context("failed to begin secondary command buffer")?;
        }
        let cache = unsafe {
            &mut *self.cmd.recorder.cache.graphics_command_cache.draw_cache.get()
        };
        let stack = self.cmd.recorder.stack().clone();
        let stack = stack.guard();
        let mut storage = unsafe { DrawCommandStorage::new(
            command_buffer,
            &self.color_formats,
            self.depth_format,
            self.stencil_format,
            self.sample_count,
            SizedCowMut::Borrowed(cache),
            DrawCommandAlloc::Borrowed(&stack),
        )? };
        let mut draw_commands = DrawCommands {
            gpu: &self.cmd.gpu,
            storage: &mut storage,
            buffers: DynResourceReadGuard::new(&self.cmd.recorder.buffers),
            images: DynResourceReadGuard::new(&self.cmd.recorder.images),
        };
        f(&mut draw_commands)?;
        unsafe {
            self.cmd.gpu.vk().device()
            .end_command_buffer(command_buffer)
            .context("failed to end secondary command buffer")?;
        }
        unsafe {
            self.cmd.execute_draw_commands(&storage)?;
        }
        Ok(())
    } 
}

impl<'a, 'b, 'c, Alloc> Drop for ActiveRenderPass<'a, 'b, 'c, Alloc>
    where Alloc: Guard<True>,
{

    fn drop(&mut self) {
        unsafe {
            self.cmd.gpu
                .vk()
                .device()
                .cmd_end_rendering(self.cmd.primary_command_buffer);
            self.color_formats.drop_and_free(&self.tmp_alloc);
        }
    }
}
