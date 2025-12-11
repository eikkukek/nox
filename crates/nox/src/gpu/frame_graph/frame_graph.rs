use std::sync::Arc;
use core::panic::Location;

use ash::vk;

use compact_str::format_compact;

use nox_mem::{vec_types::{GlobalVec, FixedVec, Vector}};

use nox_alloc::arena_alloc::ArenaAlloc;

use crate::dev::{
    export::*,
    error::{Error, Context, Tracked, ErrorContext},
    has_not_bits,
};

use crate::gpu::*;

use super::*;

pub struct FrameGraph<'a> {
    frame_context: FrameContext<'a>,
    command_buffer: vk::CommandBuffer,
    passes: GlobalVec<Pass<'a>>,
    signal_semaphore_count: u32,
    wait_semaphore_count: u32,
    queue_family_indices: QueueFamilyIndices,
    next_pass_id: u32,
    alloc: &'a ArenaAlloc,
    frame_index: u32,
}

impl<'a> FrameGraph<'a> {

    pub(crate) fn new(
        frame_context: FrameContext<'a>,
        command_buffer: vk::CommandBuffer,
        alloc: &'a ArenaAlloc,
        frame_index: u32,
        queue_family_indices: QueueFamilyIndices,
    ) -> Self
    {
        Self {
            frame_context,
            command_buffer,
            passes: GlobalVec::with_capacity(4),
            signal_semaphore_count: 0,
            wait_semaphore_count: 0,
            queue_family_indices,
            next_pass_id: 0,
            alloc,
            frame_index,
        }
    }
}

impl<'a> FrameGraph<'a> {

    pub fn gpu(&mut self) -> &mut GpuContext<'a> {
        self.frame_context.gpu()
    }

    pub fn frame_index(&self) -> u32 {
        self.frame_index
    }

    #[track_caller]
    pub fn set_render_image(&mut self, id: ResourceId, range_info: Option<ImageRangeInfo>) -> Result<()>
    {
        self.frame_context.set_render_image(id, range_info, Location::caller())?;
    }

    #[track_caller]
    pub fn add_image(&mut self, id: ImageId) -> Result<ResourceId> {
        self.frame_context.add_image(id, Location::caller())?
    }

    #[track_caller]
    pub fn add_transient_image(
        &mut self,
        f: &mut dyn FnMut(&mut ImageBuilder),
    ) -> Result<ResourceId> {
        self.frame_context.add_transient_image(f, Location::caller())?
    }

    #[track_caller]
    pub fn add_pass(
        &mut self,
        info: PassInfo,
        f: &mut dyn FnMut(&mut Pass),
    ) -> Result<PassId> {
        let alloc = self.alloc;
        let pass = self.passes.push(Pass::new(
            PassId(self.next_pass_id),
            info,
            alloc,
            Location::caller(),
        )?);
        self.next_pass_id += 1;
        f(pass);
        self.signal_semaphore_count += pass.signal_semaphores.len() as u32;
        self.wait_semaphore_count += pass.wait_semaphores.len() as u32;
        if !pass.validate(alloc)? {
            //assert!(pass.validate(alloc)?, "pass valiation error (Image subresource write overlaps)");
            todo!("pass validation error message not done")
        }
        Ok(pass.id)
    }
}

impl<'a> FrameGraph<'a> {

    pub(crate) fn render(
        &mut self,
        interface: &mut impl Interface,
        render_commands: &mut RenderCommands,
    ) -> Result<()>
    {
        let alloc = self.alloc;
        let frame_context = self.frame_context;
        let device = frame_context.device();
        let passes = &mut self.passes;
        let command_buffer = self.command_buffer;
        let graphics_queue_index = self.queue_family_indices.graphics_index();

        struct SubresourceReset {
            image: Arc<Image>,
            command_buffer: vk::CommandBuffer,
            old_state: ImageState,
            subresource: ImageSubresourceRangeInfo,
        }

        impl Drop for SubresourceReset {

            fn drop(&mut self) {
                self.image.cmd_memory_barrier(
                    self.old_state,
                    self.command_buffer,
                    Some(self.subresource),
                    false,
                ).unwrap();
            }
        }

        for pass in passes.iter() {
            let mut subresource_reset = FixedVec
                ::with_capacity(pass.reads.len() + pass.writes.len(), alloc)
                .context_with(|loc| ErrorContext::VecError(loc))?;
            let color_output_count = pass.writes.len();
            for read in pass.reads.iter() {
                let resource_id = read.resource_id;
                let image = frame_context.get_image(resource_id)?;
                let properties = image.properties;
                if has_not_bits!(properties.usage, vk::ImageUsageFlags::SAMPLED) {
                    return Err(Error::just_context("image read must be sampleable"
                    )).context(ErrorContext::EventError(read.loc()))
                }
                let state = image.state();
                let dst_state = ImageState::new(
                    vk::AccessFlags::SHADER_READ,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    graphics_queue_index,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                );
                if state == dst_state {
                    continue
                }
                let range_info = read.range_info;
                if range_info.is_some() && state.layout == vk::ImageLayout::UNDEFINED {
                    frame_context.cmd_memory_barrier(
                        resource_id,
                        dst_state,
                        None,
                    )?;
                }
                else {
                    frame_context.cmd_memory_barrier(
                        resource_id,
                        dst_state,
                        range_info.map(|v| v.subresource_info)
                    )?;
                    if let Some(info) = range_info {
                        subresource_reset.push(SubresourceReset {
                            image,
                            command_buffer,
                            old_state: state,
                            subresource: info.subresource_info,
                        }).unwrap();
                    }
                }
            }
            let mut render_extent = vk::Extent2D { width: u32::MAX, height: u32::MAX };
            enum AttachmentType {
                Color,
                Depth,
                DepthStencil,
            }
            let mut process_write = |write: &WriteInfo, ty: AttachmentType| -> Result<vk::RenderingAttachmentInfo<'static>> {
                let resource_id = write.main_id;
                let image = frame_context.get_image(resource_id)?;
                let properties = image.properties;
                let (access, layout, stage) = match ty {
                    AttachmentType::Color => {
                        if has_not_bits!(properties.usage, vk::ImageUsageFlags::COLOR_ATTACHMENT) {
                            return Err(Error::just_context("color write image must be usable as an color attachment"
                            )).context(ErrorContext::EventError(write.loc()))
                        }
                        (
                            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                        )
                    },
                    AttachmentType::Depth => {
                        if has_not_bits!(properties.usage, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
                            return Err(Error::just_context("depth/stencil write image must be usable as an depth/stencil attachment"
                            )).context(ErrorContext::EventError(write.loc()))
                        }
                        (
                            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        )
                    },
                    AttachmentType::DepthStencil => {
                        if has_not_bits!(properties.usage, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
                            return Err(Error::just_context("depth/stencil write image must be usable as an depth/stencil attachment"
                            )).context(ErrorContext::EventError(write.loc()))
                        }
                        (
                            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        )
                    }
                };
                let dst_state = ImageState::new(
                    access,
                    layout,
                    graphics_queue_index,
                    stage,
                );
                let state = image.state();
                let range_info = write.range_info;
                if state != dst_state {
                    if range_info.is_some() && state.layout == vk::ImageLayout::UNDEFINED {
                        frame_context.cmd_memory_barrier(
                            resource_id,
                            dst_state,
                            None,
                        )?;
                    }
                    else {
                        frame_context.cmd_memory_barrier(
                            resource_id,
                            dst_state,
                            range_info.map(|v| v.subresource_info)
                        )?;
                        if let Some(info) = range_info {
                            subresource_reset.push(SubresourceReset {
                                image,
                                command_buffer,
                                old_state: state,
                                subresource: info.subresource_info,
                            }).unwrap();
                        }
                    }
                }
                let mut resolve_image_view = Default::default();
                let mut resolve_image_layout = Default::default();
                let mut resolve_mode = Default::default();
                if let Some((resolve_id, mode)) = write.resolve {
                    resolve_mode = mode.into();
                    let resolve_image = frame_context.get_image(resolve_id)?;
                    let resolve_properties = resolve_image.properties;
                    if properties.dimensions != resolve_properties.dimensions {
                        return Err(Error::just_context(format_compact!("resolve image dimensions {} must match main image dimensions {}",
                            resolve_properties, properties.dimensions,
                        ))).context(ErrorContext::EventError(resource_id.loc()))
                    }
                    let state = resolve_image.state();
                    let range_info = write.resolve_range_info;
                    if state != dst_state {
                        if range_info.is_some() && state.layout == vk::ImageLayout::UNDEFINED {
                            frame_context.cmd_memory_barrier(
                                resolve_id,
                                dst_state,
                                None,
                            )?;
                        }
                        else {
                            frame_context.cmd_memory_barrier(
                                resolve_id,
                                dst_state,
                                range_info.map(|v| v.subresource_info)
                            )?;
                            if let Some(info) = range_info {
                                subresource_reset.push(SubresourceReset {
                                    image: resolve_image,
                                    command_buffer,
                                    old_state: state,
                                    subresource: info.subresource_info,
                                }
                                ).unwrap();
                            }
                        }
                    }
                    let (image_view, image_layout) =
                        if let Some(info) = range_info {
                            frame_context.create_image_view(resolve_id, info)?
                        } else {
                            frame_context.get_image_view(resolve_id)?
                        };
                    resolve_image_view = image_view;
                    resolve_image_layout = image_layout;
                }
                render_extent.width = render_extent.width.min(properties.dimensions.width);
                render_extent.height = render_extent.height.min(properties.dimensions.height);
                let (image_view, image_layout) =
                    if let Some(info) = range_info {
                        frame_context.create_image_view(resource_id, info)?
                    } else {
                        frame_context.get_image_view(resource_id)?
                    };
                Ok(vk::RenderingAttachmentInfo {
                    s_type: vk::StructureType::RENDERING_ATTACHMENT_INFO,
                    image_view,
                    image_layout,
                    load_op: write.load_op.into(),
                    store_op: write.store_op.into(),
                    clear_value: write.clear_value.into(),
                    resolve_image_view,
                    resolve_image_layout,
                    resolve_mode,
                    ..Default::default()
                })
            };
            let mut color_outputs = FixedVec::with_no_alloc();
            let writes = &pass.writes;
            if color_output_count != 0 {
                color_outputs = FixedVec::<vk::RenderingAttachmentInfo, ArenaAlloc>
                    ::with_capacity(color_output_count, alloc)
                    .context_with(|loc| ErrorContext::VecError(loc))?;
                for write in writes {
                    color_outputs
                        .push(process_write(write, AttachmentType::Color)?).unwrap();
                }
            }
            let mut stencil_output = None;
            let depth_output =
                if let Some((stencil, write)) = &pass.depth_write {
                    let v = process_write(
                        &write,
                        if *stencil {
                            AttachmentType::DepthStencil
                        } else {
                            AttachmentType::Depth
                        }
                    )?;
                    if *stencil {
                        stencil_output = Some(v);
                    }
                    Some(v)
                } else {
                    None
                };
            let rendering_info = vk::RenderingInfo {
                s_type: vk::StructureType::RENDERING_INFO,
                render_area:
                    if pass.render_area.is_some() {
                        pass.render_area.unwrap()
                    } else {
                        vk::Rect2D {
                            offset: Default::default(),
                            extent: render_extent,
                        }
                    },
                layer_count: 1,
                color_attachment_count: color_output_count as u32,
                p_color_attachments: color_outputs.as_ptr(),
                p_depth_attachment: if let Some(attachment) = &depth_output { attachment } else { 0 as _ },
                p_stencil_attachment: if let Some(attachment) = &stencil_output { attachment } else { 0 as _ },
                ..Default::default()
            };
            unsafe {
                device.cmd_begin_rendering(command_buffer, &rendering_info);
            }
            let view_port = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: rendering_info.render_area.extent.width as f32,
                height: rendering_info.render_area.extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            let scissor = rendering_info.render_area;
            unsafe {
                device.cmd_set_viewport(command_buffer, 0, &[view_port]);
                device.cmd_set_scissor(command_buffer, 0, &[scissor]);
            }
            render_commands.set_current_sample_count(pass.msaa_samples);
            interface.event(Event::RenderWork {
                pass_id: pass.id,
                commands: render_commands,
            }).context_from_origin(|orig| ErrorContext::EventError(orig))?;
            unsafe { device.cmd_end_rendering(command_buffer); }
        }
        Ok(())
    }

    pub fn signal_semaphore_count(&self) -> u32 {
        self.signal_semaphore_count
    }

    pub fn wait_semaphore_count(&self) -> u32 {
        self.signal_semaphore_count
    }

    pub(crate) fn collect_semaphores(
        &self,
        mut collect_signal: impl FnMut(TimelineSemaphoreId, u64) -> Result<()>,
        mut collect_wait: impl FnMut(TimelineSemaphoreId, u64, PipelineStage) -> Result<()>,
    ) -> Result<()>
    {
        for pass in &self.passes {
            for &(id, value) in &pass.signal_semaphores {
                collect_signal(id, value)?;
            }
            for &(id, value, stage) in &pass.wait_semaphores {
                collect_wait(id, value, stage)?;
            }
        }
        Ok(())
    }

    pub(crate) fn finalize(self) -> FrameContext<'a> {
        self.frame_context
    }
}
