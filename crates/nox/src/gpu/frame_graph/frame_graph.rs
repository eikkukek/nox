use std::sync::Arc;

use core::ops::{Deref, DerefMut};

use ash::vk;

use compact_str::format_compact;

use nox_mem::{vec_types::{GlobalVec, FixedVec, Vector}};

use nox_alloc::arena_alloc::ArenaAlloc;

use crate::dev::{
    export::*,
    error::{Error, Context, ErrorContext, caller, location},
    has_not_bits,
};

use crate::gpu::*;
use crate::gpu::frame_context::ImageSource;

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

    #[inline(always)]
    pub fn gpu(&self) -> &GpuContext<'a> {
        self.frame_context.gpu()
    }

    #[inline(always)]
    pub fn gpu_mut(&mut self) -> &mut GpuContext<'a> {
        self.frame_context.gpu_mut()
    }

    pub fn frame_index(&self) -> u32 {
        self.frame_index
    }

    #[track_caller]
    pub fn swapchain_image(&self) -> ResourceId {
        self.frame_context.swapchain_image(caller!())
    }

    #[track_caller]
    pub fn add_image(&mut self, id: ImageId) -> Result<ResourceId> {
        self.frame_context.add_image(id, caller!())
    }

    #[track_caller]
    pub fn add_transient_image(
        &mut self,
        f: impl FnMut(&mut ImageBuilder),
    ) -> Result<ResourceId> {
        self.frame_context.add_transient_image(f, caller!())
    }

    #[track_caller]
    pub fn add_pass(
        &mut self,
        info: PassInfo,
        mut f: impl FnMut(&mut PassBuilder) -> Result<()>,
    ) -> Result<PassId> {
        let alloc = self.alloc;
        let pass = self.passes.push(Pass::new(
            PassId(self.next_pass_id),
            info,
            alloc,
            caller!()
        )?);
        self.next_pass_id += 1;
        f(&mut PassBuilder { pass, }).context("failed to build pass")?;
        self.signal_semaphore_count += pass.signal_semaphores.len() as u32;
        self.wait_semaphore_count += pass.wait_semaphores.len() as u32;
        if let Some(err) = pass.validate(alloc)? {
            return Err(Error::new(
                "pass was invalid",
                err,
            ))
        }
        Ok(pass.id)
    }
}

impl<'a> FrameGraph<'a> {

    pub(crate) fn render<Token: CellToken>(
        mut self,
        token: &mut Token,
        process: &mut impl ProcessEvent<Token>,
        frame_semaphore: vk::Semaphore,
        frame_semaphore_value: u64,
        buffered_frames: u32,
    ) -> Result<FrameGraphResult<'a>>
    {
        let alloc = self.alloc;
        let device = self.frame_context.device();
        let command_buffer = self.command_buffer;
        let queue_family_indices = self.queue_family_indices;
        let graphics_queue_index = queue_family_indices.graphics_index();

        let mut passes = FixedVec
            ::with_capacity(self.passes.len(), alloc)
            .context_with(|| ErrorContext::VecError(location!()))?;
        passes
            .move_from_vec(&mut self.passes)
            .context_with(|| ErrorContext::VecError(location!()))?;

        let mut render_commands = RenderCommands::new(
            command_buffer, &mut self, frame_semaphore, frame_semaphore_value,
            alloc, queue_family_indices, buffered_frames
        ).context("failed to initialize render commands")?;

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
                .context_with(|| ErrorContext::VecError(location!()))?;
            let color_output_count = pass.writes.len();
            for read in pass.reads.iter() {
                let resource_id = read.id;
                let image = match render_commands.frame_graph.frame_context.get_image(resource_id)? {
                    ImageSource::Owned(image) => image,
                    ImageSource::Swapchain(_, _, _,) => return Err(Error::just_context(
                        "swapchain images are write only"
                    )).context(ErrorContext::EventError(read.location_or_this()))
                        .context(format_compact!("error while processing pass at {}", pass.location_or_this())),
                };
                let properties = image.properties;
                if has_not_bits!(properties.usage, vk::ImageUsageFlags::SAMPLED) {
                    return Err(Error::just_context("image read must be sampleable"
                    )).context(ErrorContext::EventError(read.location_or_this()))
                        .context(format_compact!("error while processing pass at {}", pass.location_or_this()))
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
                let range_info = read.range;
                if range_info.is_some() && state.layout == vk::ImageLayout::UNDEFINED {
                    render_commands.frame_graph.frame_context.cmd_memory_barrier(
                        resource_id,
                        dst_state,
                        None,
                    )?;
                }
                else {
                    render_commands.frame_graph.frame_context.cmd_memory_barrier(
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
            let mut process_write = |write: &WriteInfo, ty: AttachmentType| -> Result<vk::RenderingAttachmentInfo<'static>>
            {
                let resource_id = write.id;
                let image = render_commands.frame_graph.frame_context.get_image(resource_id)?;
                match image {
                    ImageSource::Owned(image) => {
                        let properties = image.properties;
                        let (access, layout, stage) = match ty {
                            AttachmentType::Color => {
                                if has_not_bits!(properties.usage, vk::ImageUsageFlags::COLOR_ATTACHMENT) {
                                    return Err(Error::just_context(
                                        "color write image must be usable as an color attachment"
                                    )).context(ErrorContext::EventError(write.location_or_this()))
                                }
                                (
                                    vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                                )
                            },
                            AttachmentType::Depth => {
                                if has_not_bits!(properties.usage, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
                                    return Err(Error::just_context(
                                        "depth/stencil write image must be usable as an depth/stencil attachment"
                                    )).context(ErrorContext::EventError(write.location_or_this()))
                                }
                                (
                                    vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                                    vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                                    vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS |
                                    vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                                )
                            },
                            AttachmentType::DepthStencil => {
                                if has_not_bits!(properties.usage, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
                                    return Err(Error::just_context(
                                        "depth/stencil write image must be usable as an depth/stencil attachment"
                                    )).context(ErrorContext::EventError(write.location_or_this()))
                                }
                                (
                                    vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                                    vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                                    vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS |
                                    vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
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
                        let range_info = write.range;
                        if state != dst_state {
                            if range_info.is_some() && state.layout == vk::ImageLayout::UNDEFINED {
                                render_commands.frame_graph.frame_context.cmd_memory_barrier(
                                    resource_id,
                                    dst_state,
                                    None,
                                )?;
                            }
                            else {
                                render_commands.frame_graph.frame_context.cmd_memory_barrier(
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
                        if let Some(resolve) = write.resolve {
                            resolve_mode = resolve.mode.into();
                            let resolve_image = render_commands.frame_graph.frame_context.get_image(resolve.id)?;
                            match resolve_image {
                                ImageSource::Owned(resolve_image) => {
                                    let resolve_properties = resolve_image.properties;
                                    if properties.dimensions != resolve_properties.dimensions {
                                        return Err(Error::just_context(format_compact!(
                                            "resolve image dimensions {} must match main image dimensions {}",
                                            resolve_properties.dimensions, properties.dimensions,
                                        ))).context(ErrorContext::EventError(resource_id.location_or_this()))
                                    }
                                    let state = resolve_image.state();
                                    let range_info = resolve.range;
                                    if state != dst_state {
                                        if range_info.is_some() && state.layout == vk::ImageLayout::UNDEFINED {
                                            render_commands.frame_graph.frame_context.cmd_memory_barrier(
                                                resolve.id,
                                                dst_state,
                                                None,
                                            )?;
                                        }
                                        else {
                                            render_commands.frame_graph.frame_context.cmd_memory_barrier(
                                                resolve.id,
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
                                            render_commands.frame_graph.frame_context.create_image_view(resolve.id, info)?
                                        } else {
                                            render_commands.frame_graph.frame_context.get_image_view(resolve.id)?
                                        };
                                    resolve_image_view = image_view;
                                    resolve_image_layout = image_layout;
                                },
                                ImageSource::Swapchain(image, view, state) => {
                                    let dimensions = render_commands.frame_graph.frame_context
                                        .gpu()
                                        .frame_buffer_size();
                                    if properties.dimensions != dimensions {
                                        return Err(Error::just_context(format_compact!(
                                            "resolve (swapchain) image dimensions {} must match main image dimensions {}",
                                            dimensions, properties.dimensions,
                                        ))).context(ErrorContext::EventError(resource_id.location_or_this()))
                                    }
                                    if resolve.range.is_some() {
                                        return Err(Error::just_context("swapchain images don't support image ranges"
                                        )).context(ErrorContext::EventError(resource_id.location_or_this()))
                                            .context(format_compact!("error while processing pass at {}", pass.location_or_this()))
                                    }
                                    if state != dst_state {
                                        let memory_barrier = state.to_memory_barrier(
                                            image, dst_state,
                                            SwapchainContext::subresource_range_info(),
                                        );
                                        unsafe {
                                            device.cmd_pipeline_barrier(
                                                command_buffer,
                                                state.pipeline_stage,
                                                dst_state.pipeline_stage,
                                                Default::default(),
                                                Default::default(),
                                                Default::default(),
                                                &[memory_barrier]);
                                        }
                                        render_commands.frame_graph.frame_context.set_swapchain_image_state(dst_state);
                                    }
                                    resolve_image_view = view;
                                    resolve_image_layout = dst_state.layout;
                                },
                            }
                        }
                        render_extent.width = render_extent.width.min(properties.dimensions.width);
                        render_extent.height = render_extent.height.min(properties.dimensions.height);
                        let (image_view, image_layout) =
                            if let Some(info) = range_info {
                                render_commands.frame_graph.frame_context.create_image_view(resource_id, info)?
                            } else {
                                render_commands.frame_graph.frame_context.get_image_view(resource_id)?
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
                    },
                    ImageSource::Swapchain(image, view, state) => {
                        let (access, layout, stage) = match ty {
                            AttachmentType::Color => {
                                (
                                    vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                                )
                            },
                            _ => return Err(Error::just_context("swapchain images can only be used as color attachments"
                            )).context(ErrorContext::EventError(resource_id.location_or_this()))
                            .context(format_compact!("error while processing pass at {}", pass.location_or_this())),
                        };
                        let dst_state = ImageState::new(
                            access,
                            layout,
                            graphics_queue_index,
                            stage,
                        );
                        if write.resolve.is_some() {
                            return Err(Error::just_context("swapchain images can't be resolved"
                            )).context(ErrorContext::EventError(resource_id.location_or_this()))
                            .context(format_compact!("error while processing pass at {}", pass.location_or_this()))
                        }
                        if state != dst_state {
                            let memory_barrier = state.to_memory_barrier(
                                image, dst_state,
                                SwapchainContext::subresource_range_info(),
                            );
                            unsafe {
                                device.cmd_pipeline_barrier(
                                    command_buffer,
                                    state.pipeline_stage,
                                    dst_state.pipeline_stage,
                                    Default::default(),
                                    Default::default(),
                                    Default::default(),
                                    &[memory_barrier]);
                            }
                            render_commands.frame_graph.frame_context.set_swapchain_image_state(dst_state);
                        }
                        let dimensions = render_commands.frame_graph.frame_context
                            .gpu()
                            .frame_buffer_size();
                        render_extent.width = render_extent.width.min(dimensions.width);
                        render_extent.height = render_extent.height.min(dimensions.height);
                        Ok(vk::RenderingAttachmentInfo {
                            s_type: vk::StructureType::RENDERING_ATTACHMENT_INFO,
                            image_view: view,
                            image_layout: dst_state.layout,
                            load_op: write.load_op.into(),
                            store_op: write.store_op.into(),
                            clear_value: write.clear_value.into(),
                            ..Default::default()
                        })
                    },
                }
            };
            let mut color_outputs = FixedVec::with_no_alloc();
            let writes = &pass.writes;
            if color_output_count != 0 {
                color_outputs = FixedVec::<vk::RenderingAttachmentInfo, ArenaAlloc>
                    ::with_capacity(color_output_count, alloc)
                    .context_with(|| ErrorContext::VecError(location!()))?;
                for write in writes {
                    color_outputs
                        .push(process_write(write, AttachmentType::Color)
                            .context_with(||
                                format_compact!("error while processing pass at {}", pass.location_or_this())
                            )?
                        ).unwrap();
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
                p_stencil_attachment: if let Some(attachment) = &stencil_output {
                    attachment
                } else {
                    0 as _
                },
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
            (process)(token, Event::RenderWork {
                pass_id: pass.id,
                commands: &mut render_commands,
            }).context_from_tracked(|orig| ErrorContext::EventError(orig.or_this()))?;
            unsafe { device.cmd_end_rendering(command_buffer); }
        }
        Ok(FrameGraphResult {
            render_commands: render_commands.finish(),
            frame_graph: self, 
        })
    }

    pub fn signal_semaphore_count(&self) -> u32 {
        self.signal_semaphore_count
    }

    pub fn wait_semaphore_count(&self) -> u32 {
        self.signal_semaphore_count
    }

    pub(crate) fn collect_semaphores(
        &self,
        mut collect_signal: impl FnMut(&Self, TimelineSemaphoreId, u64) -> Result<()>,
        mut collect_wait: impl FnMut(&Self, TimelineSemaphoreId, u64, PipelineStage) -> Result<()>,
    ) -> Result<()>
    {
        for pass in &self.passes {
            for &(id, value) in &pass.signal_semaphores {
                collect_signal(self, id, value)?;
            }
            for &(id, value, stage) in &pass.wait_semaphores {
                collect_wait(self, id, value, stage)?;
            }
        }
        Ok(())
    }

    pub(crate) fn finalize(self) -> FrameContext<'a> {
        self.frame_context
    }
}

pub(crate) struct FrameGraphResult<'a> {
   pub frame_graph: FrameGraph<'a>,
   pub render_commands: RenderCommandsStorage,
}

impl<'a> FrameGraphResult<'a> {

    pub fn device(&self) -> Arc<ash::Device> {
        self.frame_graph.frame_context.device()
    }
}

impl<'a> Deref for FrameGraphResult<'a> {

    type Target = FrameGraph<'a>;

    fn deref(&self) -> &Self::Target {
        &self.frame_graph
    }
}

impl<'a> DerefMut for FrameGraphResult<'a> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frame_graph
    }
}
