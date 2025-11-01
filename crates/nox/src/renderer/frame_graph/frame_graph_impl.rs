use std::rc::Rc;

use ash::vk;

use token_cell::prelude::TokenCell;

use nox_mem::{Allocator, vec_types::{GlobalVec, FixedVec, Vector}};

use crate::{
    has_bits, renderer::{
        frame_state::{FrameState, ResourceId},
        image::{
            Image, ImageBuilder, ImageRangeInfo, ImageState, ImageSubresourceRangeInfo
        },
        FrameToken,
        *
    },
};

use super::*;

pub(crate) struct FrameGraphImpl<'a, Alloc: Allocator> {
    frame_state: Rc<TokenCell<FrameState, FrameToken>>,
    frame_buffer_size: image::Dimensions,
    command_buffer: vk::CommandBuffer,
    passes: GlobalVec<Pass<'a, Alloc>>,
    signal_semaphore_count: u32,
    queue_family_indices: QueueFamilyIndices,
    next_pass_id: u32,
    alloc: &'a Alloc,
    frame_index: u32,
    token: &'a mut FrameToken,
}

impl<'a, Alloc: Allocator> FrameGraphImpl<'a, Alloc> {

    pub fn new(
        frame_state: Rc<TokenCell<FrameState, FrameToken>>,
        frame_buffer_size: image::Dimensions,
        command_buffer: vk::CommandBuffer,
        alloc: &'a Alloc,
        frame_index: u32,
        queue_family_indices: QueueFamilyIndices,
        token: &'a mut FrameToken,
    ) -> FrameGraphImpl<'a, Alloc>
    {
        frame_state.borrow_mut(token).init(command_buffer);
        FrameGraphImpl {
            frame_state,
            frame_buffer_size,
            command_buffer,
            passes: GlobalVec::with_capacity(4),
            signal_semaphore_count: 0,
            queue_family_indices,
            next_pass_id: 0,
            alloc,
            frame_index,
            token,
        }
    }
}

impl<'a, Alloc: Allocator> FrameGraph<'a> for FrameGraphImpl<'a, Alloc> {

    fn edit_resources(
        &mut self,
        f: &mut dyn FnMut(&mut GlobalResources) -> Result<(), Error>
    ) -> Result<(), Error> {
        f(&mut self.frame_state
            .borrow_mut(self.token).resource_pool.global_resources
            .write()
            .unwrap()
        )
    }

    fn frame_index(&self) -> u32 {
        self.frame_index
    }

    fn frame_buffer_size(&self) -> image::Dimensions {
        self.frame_buffer_size
    }

    fn set_render_image(&mut self, id: ResourceId, range_info: Option<ImageRangeInfo>) -> Result<(), Error>
    {
        assert!(self.frame_state.borrow(self.token).is_valid_resource_id(id), "invalid id");
        self.frame_state.borrow_mut(self.token).set_render_image(id, range_info)
    }

    fn add_image(&mut self, id: ImageId) -> Result<ResourceId, Error> {
        self.frame_state.borrow_mut(self.token).add_image(id)
    }

    fn add_transient_image(
        &mut self,
        f: &mut dyn FnMut(&mut ImageBuilder),
    ) -> Result<ResourceId, Error> {
        self.frame_state.borrow_mut(self.token).add_transient_image(f)
    }

    fn add_pass(
        &mut self,
        info: PassInfo,
        f: &mut dyn FnMut(&mut dyn PassAttachmentBuilder),
    ) -> Result<PassId, Error> {
        let alloc = self.alloc;
        let pass = self.passes.push(Pass::new(
            PassId(self.next_pass_id),
            info,
            alloc
        )?);
        self.next_pass_id += 1;
        f(pass);
        self.signal_semaphore_count += pass.signal_semaphores.len() as u32;
        assert!(pass.validate(alloc)?, "pass valiation error (Image subresource write overlaps)");
        Ok(pass.id)
    }
}

impl<'a, Alloc: Allocator> FrameGraphImpl<'a, Alloc> {

    pub fn render(&mut self, interface: &mut impl Interface, render_commands: &mut RenderCommands) -> Result<(), Error> {
        let alloc = self.alloc;
        let frame_state = self.frame_state.borrow_mut(self.token);
        let device = frame_state.device();
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
            let mut subresource_reset = FixedVec::with_capacity(pass.reads.len() + pass.writes.len(), alloc)?;
            let color_output_count = pass.writes.len();
            for read in pass.reads.iter() {
                let resource_id = read.resource_id;
                let image = frame_state.get_image(resource_id)?;
                let properties = image.properties;
                assert!(has_bits!(properties.usage, vk::ImageUsageFlags::SAMPLED),
                    "read image usage must contain ImageUsage::Sampled bit");
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
                    frame_state.cmd_memory_barrier(
                        resource_id,
                        dst_state,
                        None,
                    )?;
                }
                else {
                    frame_state.cmd_memory_barrier(
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
            let mut process_write = |write: &WriteInfo, ty: AttachmentType| -> Result<vk::RenderingAttachmentInfo<'static>, Error> {
                let resource_id = write.main_id;
                let image = frame_state.get_image(resource_id)?;
                let properties = image.properties;
                let (access, layout, stage) = match ty {
                        AttachmentType::Color => {
                            assert!(
                                has_bits!(properties.usage, vk::ImageUsageFlags::COLOR_ATTACHMENT),
                                "color write image usage must contain ImageUsage::ColorAttachment bit"
                            );
                            (
                                vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                            )
                        },
                        AttachmentType::Depth => {
                            assert!(
                                has_bits!(properties.usage, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
                                "depth/stencil write image usage must contain ImageUsage::DepthStencilAttachment bit"
                            );
                            (
                                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                                vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                            )
                        },
                        AttachmentType::DepthStencil => {
                            assert!(
                                has_bits!(properties.usage, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
                                "depth/stencil write image usage must contain ImageUsage::DepthStencilAttachment bit"
                            );
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
                        frame_state.cmd_memory_barrier(
                            resource_id,
                            dst_state,
                            None,
                        )?;
                    }
                    else {
                        frame_state.cmd_memory_barrier(
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
                    let resolve_image = frame_state.get_image(resolve_id)?;
                    let resolve_properties = resolve_image.properties;
                    assert!(properties.dimensions == resolve_properties.dimensions,
                        "resolve image dimensions must match main image dimensions, main dimensions {:?}, resolve dimensions {:?}",
                        properties.dimensions, resolve_properties.dimensions,
                    );
                    let state = resolve_image.state();
                    let range_info = write.resolve_range_info;
                    if state != dst_state {
                        if range_info.is_some() && state.layout == vk::ImageLayout::UNDEFINED {
                            frame_state.cmd_memory_barrier(
                                resolve_id,
                                dst_state,
                                None,
                            )?;
                        }
                        else {
                            frame_state.cmd_memory_barrier(
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
                            frame_state.create_image_view(resolve_id, info)?
                        } else {
                            frame_state.get_image_view(resolve_id)?
                        };
                    resolve_image_view = image_view;
                    resolve_image_layout = image_layout;
                }
                render_extent.width = render_extent.width.min(properties.dimensions.width);
                render_extent.height = render_extent.height.min(properties.dimensions.height);
                let (image_view, image_layout) =
                    if let Some(info) = range_info {
                        frame_state.create_image_view(resource_id, info)?
                    } else {
                        frame_state.get_image_view(resource_id)?
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
                color_outputs = FixedVec::<vk::RenderingAttachmentInfo, Alloc>::with_capacity(color_output_count, alloc)?;
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
            interface.render_commands(pass.id, render_commands)?;
            unsafe { device.cmd_end_rendering(command_buffer); }
        }
        Ok(())
    }

    pub fn signal_semaphore_count(&self) -> u32 {
        self.signal_semaphore_count
    }

    pub fn collect_signal_semaphores(
        &self,
        mut collect: impl FnMut(TimelineSemaphoreId, u64) -> Result<(), Error>
    ) -> Result<(), Error>
    {
        for pass in &self.passes {
            for &(id, value) in &pass.signal_semaphores {
                collect(id, value)?;
            }
        }
        Ok(())
    }
}
