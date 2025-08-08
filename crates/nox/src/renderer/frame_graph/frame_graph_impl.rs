use std::rc::Rc;

use core::cell::RefCell;

use ash::vk;

use nox_mem::{Allocator, CapacityError, vec_types::{FixedVec, Vector}};

use crate::{
    renderer::{
        *,
        global_resources::*,
        frame_state::SubresourceResetGuard,
        image::{
            ImageState,
            ImageBuilder,
        },
        frame_state::{FrameState, ResourceID},
    },
    has_bits,
};

use super::*;

pub(crate) struct FrameGraphImpl<'a, Alloc: Allocator> {
    frame_state: Rc<RefCell<FrameState>>,
    frame_buffer_size: image::Dimensions,
    command_buffer: vk::CommandBuffer,
    passes: FixedVec<'a, Pass<'a, Alloc>, Alloc>,
    queue_family_indices: QueueFamilyIndices,
    next_pass_id: u32,
    alloc: &'a Alloc,
    frame_index: u32,
}

impl<'a, Alloc: Allocator> FrameGraphImpl<'a, Alloc> {

    pub fn new(
        frame_state: Rc<RefCell<FrameState>>,
        frame_buffer_size: image::Dimensions,
        command_buffer: vk::CommandBuffer,
        alloc: &'a Alloc,
        frame_index: u32,
        queue_family_indices: QueueFamilyIndices,
    ) -> FrameGraphImpl<'a, Alloc>
    {
        FrameGraphImpl {
            frame_state,
            frame_buffer_size,
            command_buffer,
            passes: FixedVec::with_no_alloc(),
            queue_family_indices,
            next_pass_id: 0,
            alloc,
            frame_index,
        }
    }
}

impl<'a, Alloc: Allocator> FrameGraphInit<'a> for FrameGraphImpl<'a, Alloc> {

    fn init(
        &mut self,
        max_passes: u32,
    ) -> Result<&mut dyn FrameGraph<'a>, Error>
    {
        if max_passes != 0 {
            self.passes = FixedVec::with_capacity(max_passes as usize, self.alloc)?;
        }
        let command_buffer = self.command_buffer;
        self.frame_state.borrow_mut().init(command_buffer);
        Ok(self)
    }
}

impl<'a, Alloc: Allocator> FrameGraph<'a> for FrameGraphImpl<'a, Alloc> {

    fn frame_index(&self) -> u32 {
        self.frame_index
    }

    fn frame_buffer_size(&self) -> image::Dimensions {
        self.frame_buffer_size
    }

    fn set_render_image(&mut self, id: ResourceID)
    {
        assert!(self.frame_state.borrow().is_valid_resource_id(id), "invalid id");
        self.frame_state.borrow_mut().set_render_image(id);
    }

    fn add_image(&mut self, id: ImageSourceID) -> Result<ResourceID, Error> {
        self.frame_state.borrow_mut().add_image(id)
    }

    fn add_transient_image(
        &mut self,
        f: &mut dyn FnMut(&mut ImageBuilder),
    ) -> Result<ResourceID, Error> {
        self.frame_state.borrow_mut().add_transient_image(f)
    }

    fn add_transient_image_subresource(
        &mut self,
        resource_id: ResourceID,
        range_info: crate::renderer::image::ImageRangeInfo,
    ) -> Result<ResourceID, Error>
    {
        self.frame_state.borrow_mut().add_transient_image_subresource(resource_id, range_info)
    }

    fn add_pass(
        &mut self,
        info: PassInfo,
        f: &mut dyn FnMut(&mut dyn PassAttachmentBuilder),
    ) -> Result<PassID, Error> {
        let alloc = self.alloc;
        let pass = self.passes.push(Pass::new(
            PassID(self.next_pass_id),
            info,
            alloc
        )?).expect("pass capacity exceeded");
        self.next_pass_id += 1;
        f(pass);
        assert!(pass.validate(alloc)?, "pass valiation error (Image subresource write overlaps)");
        Ok(pass.id)
    }
}

impl<'a, Alloc: Allocator> FrameGraphImpl<'a, Alloc> {

    pub fn render(&mut self, interface: &mut impl Interface, render_commands: &mut RenderCommands) -> Result<(), Error> {
        let alloc = self.alloc;
        let frame_state = self.frame_state.borrow_mut();
        let device = frame_state.device();
        let passes = &mut self.passes;
        let sorted = Self::sort(passes, alloc)?;
        let command_buffer = self.command_buffer;
        let graphics_queue_index = self.queue_family_indices.graphics_index();
        for index in sorted.iter().map(|i| *i) {
            let pass = &mut passes[index];
            let color_output_count = pass.writes.len();
            let mut reset_guards = FixedVec::<SubresourceResetGuard, Alloc>::with_capacity(pass.reads.len() + color_output_count + 2, alloc)?;
            for read in pass.reads.iter() {
                let resource_id = read.resource_id;
                let image_properties = frame_state.get_image_properties(resource_id)?;
                assert!(has_bits!(image_properties.usage, vk::ImageUsageFlags::SAMPLED),
                    "read image usage must contain ImageUsage::Sampled bit");
                let reset_guard = frame_state.cmd_memory_barrier(
                    resource_id,
                    ImageState::new(
                        vk::AccessFlags::SHADER_READ,
                        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                        graphics_queue_index,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                    ),
                )?;
                if let Some(guard) = reset_guard {
                    reset_guards.push(guard).unwrap();
                }
            }
            let mut render_extent = vk::Extent2D { width: u32::MAX, height: u32::MAX };
            enum AttachmentType {
                Color,
                Depth,
                Stencil,
            }
            let mut process_write = |write: &WriteInfo, ty: AttachmentType| -> Result<vk::RenderingAttachmentInfo<'static>, Error> {
                let resource_id = write.resource_id;
                let image_properties = frame_state.get_image_properties(resource_id)?;
                let layout = match ty {
                        AttachmentType::Color => {
                            assert!(
                                has_bits!(image_properties.usage, vk::ImageUsageFlags::COLOR_ATTACHMENT),
                                "color write image usage must contain ImageUsage::ColorAttachment bit"
                            );
                            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
                        },
                        AttachmentType::Depth => {
                            assert!(
                                has_bits!(image_properties.usage, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
                                "depth/stencil write image usage must contain ImageUsage::DepthStencilAttachment bit"
                            );
                            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL
                        },
                        AttachmentType::Stencil => {
                            assert!(
                                has_bits!(image_properties.usage, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
                                "depth/stencil write image usage must contain ImageUsage::DepthStencilAttachment bit"
                            );
                            vk::ImageLayout::STENCIL_ATTACHMENT_OPTIMAL
                        }
                };
                let reset_guard = frame_state.cmd_memory_barrier(
                    resource_id,
                    ImageState::new(
                        vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                        layout,
                        graphics_queue_index,
                        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    ),
                )?;
                render_extent.width = render_extent.width.min(image_properties.dimensions.width);
                render_extent.height = render_extent.height.min(image_properties.dimensions.height);
                let (image_view, image_layout) = frame_state.get_image_view(resource_id)?;
                if let Some(guard) = reset_guard {
                    reset_guards.push(guard).unwrap();
                }
                Ok(vk::RenderingAttachmentInfo {
                    s_type: vk::StructureType::RENDERING_ATTACHMENT_INFO,
                    image_view,
                    image_layout,
                    load_op: write.load_op.into(),
                    store_op: write.store_op.into(),
                    clear_value: write.clear_value.into(),
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
            let depth_output =
                if let Some(write) = &pass.depth_write {
                    Some(process_write(&write, AttachmentType::Depth)?)
                } else {
                    None
                };
            let stencil_output =
                if let Some(write) = &pass.stencil_write {
                    Some(process_write(write, AttachmentType::Stencil)?)
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
            interface.render_commands(pass.id, render_commands)?;
            unsafe { device.cmd_end_rendering(command_buffer); }
        }
        Ok(())
    }

    #[inline(always)]
    fn sort(passes: &FixedVec<Pass<Alloc>, Alloc>, alloc: &'a Alloc) -> Result<FixedVec<'a, usize, Alloc>, CapacityError> {
        if passes.len() == 0 {
            return Ok(FixedVec::with_no_alloc())
        }
        let pass_count = passes.len();
        if pass_count == 0 {
            return Ok(FixedVec::with_no_alloc())
        }
        let mut in_degree = FixedVec::<usize, Alloc>
            ::with_len(pass_count, 0, alloc)?;
        let mut dependents = FixedVec::<FixedVec<usize, Alloc>, Alloc>
            ::with_len_with(pass_count, FixedVec::with_no_alloc, alloc)?;
        for (i, pass) in passes.iter().enumerate() {
            let deps = &pass.dependencies;
            in_degree[i] = deps.len();
            for index in deps {
                let list = &mut dependents[index.0 as usize];
                if list.capacity() == 0 {
                    *list =
                        FixedVec::with_capacity(pass_count, alloc)?;
                }
                list.push(i)?;
            }
            
        }
        if in_degree.len() == 0 {
            let mut i = 0;
            return
                FixedVec
                ::with_len_with(pass_count,
                    || {
                        let index = i;
                        i += 1;
                        index
                    },
                    alloc
                )
        }
        let mut pending = FixedVec::<usize, Alloc>
            ::with_capacity(pass_count, alloc)?;
        for (i, deg) in in_degree.iter().enumerate() {
            if *deg == 0 {
                pending
                    .push(i)?;
            }
        }
        let mut sorted = FixedVec
            ::with_capacity(pass_count, alloc)?;
        while let Some(index) = pending.pop() {
            sorted.push(index)?;
            let list = &dependents[index];
            if list.capacity() != 0 {
                for index in list.iter().map(|i| *i) {
                    let count = &mut in_degree[index];
                    *count -= 1;
                    if *count == 0 {
                        pending.push(index)?;
                    }
                }
            }
        }
        if sorted.len() != pass_count {
            panic!("cycle detected")
        }
        else {
            Ok(sorted)
        }
    }
}
