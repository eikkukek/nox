use super::{
    ImageState,
    helpers::Handle,
};

use crate::{
    allocator_traits::AllocateExt,
    map_types::FixedMap,
    string::{String, SmallError},
    vec_types::{Vector, FixedVec}
};

use ash::vk;

use std::slice;

pub type UID = u64;
pub type PassName = String<64>;

#[derive(Clone)]
pub struct ImageResource<'r> {
    handle: Handle<'r, vk::Image>,
    view: Handle<'r, vk::ImageView>,
    subresource_range: vk::ImageSubresourceRange,
    state: ImageState,
    format: vk::Format,
    extent: vk::Extent2D,
}

impl<'r> ImageResource<'r> {

    pub fn new(
        handle: Handle<'r, vk::Image>,
        view: Handle<'r, vk::ImageView>,
        subresource_range: vk::ImageSubresourceRange,
        state: ImageState,
        format: vk::Format,
        extent: vk::Extent2D,
    ) -> Self {
        Self {
            handle,
            view,
            subresource_range,
            state,
            format,
            extent,
        }
    }

    pub fn handle(&self) -> Handle<'r, vk::Image> {
        self.handle.clone()
    }

    pub fn view(&self) -> Handle<'r, vk::ImageView> {
        self.view.clone()
    }

    pub fn subresource_range(&self) -> &vk::ImageSubresourceRange {
        &self.subresource_range
    }

    pub fn state(&self) -> ImageState {
        self.state
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }
}

pub struct ResourcePool<'a, 'r> {
    next_uid: UID,
    image_resources: FixedMap<'a, UID, ImageResource<'r>>,
}

impl<'a, 'r> ResourcePool<'a, 'r> {

    pub fn new<A: AllocateExt<'a>>(
        max_images: usize,
        allocator: &mut A,
    ) -> Option<Self> {
        Some(Self{
            image_resources: FixedMap::new(max_images, allocator)?,
            next_uid: 0,
        })
    }

    pub fn add_image_resource(&mut self, resource: ImageResource<'r>) -> Option<UID> {
        let uid = self.next_uid;
        self.image_resources.insert(uid, resource)?;
        self.next_uid = uid + 1;
        Some(uid)
    }
}

#[derive(Clone)]
pub struct WriteInfo {
    resource_uid: UID,
    image_state: ImageState,
    load_op: vk::AttachmentLoadOp,
    store_op: vk::AttachmentStoreOp,
    clear_value: vk::ClearValue,
}

impl WriteInfo {

    pub fn new(
        resource_uid: UID,
        image_state: ImageState,
        load_op: vk::AttachmentLoadOp,
        store_op: vk::AttachmentStoreOp,
        clear_value: vk::ClearValue,
    ) -> Self {
        Self {
            resource_uid,
            image_state,
            load_op,
            store_op,
            clear_value,
        }
    }
}

pub struct Pass<'a> {
    reads: Option<FixedVec<'a, (UID, ImageState)>>,
    color_writes: Option<FixedVec<'a, WriteInfo>>,
    depth_write: Option<WriteInfo>,
    stencil_write: Option<WriteInfo>,
    dependencies: Option<FixedVec<'a, UID>>,
    render_area: Option<vk::Rect2D>,
    pub name: PassName,
}

impl<'a> Pass<'a> {
    
    pub fn new<A: AllocateExt<'a>>(
        name: PassName,
        max_reads: usize,
        max_color_writes: usize,
        max_dependencies: usize,
        allocator: &mut A,
    ) -> Option<Self> {
        Some(Self {
            reads: if max_reads != 0 { Some(FixedVec::new(max_reads, allocator)?) } else { None },
            color_writes: if max_color_writes != 0 { Some(FixedVec::new(max_color_writes, allocator)?) } else { None },
            depth_write: None,
            stencil_write: None,
            dependencies: if max_dependencies != 0 { Some(FixedVec::new(max_dependencies, allocator)?) } else { None },
            render_area: None,
            name,
        })
    }

    pub fn name(&mut self, name: &str) {
        self.name = String::from_str(name);
    }

    pub fn add_read(&mut self, uid: UID, dst_image_state: ImageState) -> Option<&mut Self> {
        if let Some(reads) = &mut self.reads {
            reads.push_back((uid, dst_image_state))?;
        }
        return Some(self)
    }

    pub fn add_color_write(&mut self, write: WriteInfo) -> Option<&mut Self> {
        if let Some(writes) = &mut self.color_writes {
            writes.push_back(write)?;
        }
        Some(self)
    }

    pub fn depth_write(&mut self, write: WriteInfo) -> &mut Self {
        self.depth_write = Some(write);
        self
    }

    pub fn stencil_write(&mut self, write: WriteInfo) -> &mut Self {
        self.stencil_write = Some(write);
        self
    }

    pub fn render_area(&mut self, offset: vk::Offset2D, extent: vk::Extent2D) -> &mut Self {
        self.render_area = Some(vk::Rect2D { offset, extent });
        self
    }

    pub fn add_dependency(&mut self, uid: UID) -> Option<&mut Self> {
        if let Some(dependencies) = &mut self.dependencies {
            dependencies.push_back(uid)?;
        }
        Some(self)
    }
}

pub trait Exec<'a> {

    fn execute<'r, 'b, B: AllocateExt<'b>>(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        resource_pool: &mut ResourcePool<'a, 'r>,
        swapchain_image_resource: &mut ImageResource<'r>,
        funs: Option<&FixedMap<'a, UID, fn(UID)>>,
        temp_allocator: &mut B,
    ) -> Option<()>;
}

pub struct FrameGraph<'a> {
    passes: FixedMap<'a, UID, Pass<'a>>, // ordered map
    sorted: FixedVec<'a, UID>,
}

impl<'a> FrameGraph<'a> {

    pub fn new<'b, A: AllocateExt<'a>, B: AllocateExt<'b>>(
        passes: FixedMap<'a, UID, Pass<'a>>,
        allocator: &mut A,
        temp_allocator: &mut B,
    ) -> Result<Self, SmallError> {
        let Some(mut in_degree) = FixedMap::<UID, usize>::new(passes.size(), temp_allocator) else {
            return Err(String::from_str("allocation failed"))
        };
        let Some(mut dependents) = FixedMap::<UID, FixedVec<UID>>::new(passes.size(), temp_allocator) else {
            return Err(String::from_str("allocation failed"))
        };
        for (uid, pass) in passes.iter() {
            if let Some(deps) = &pass.dependencies {
                for dep in deps {
                    in_degree.insert_or_modify(*uid, || 1, |v| *v += 1);
                    let Some(dependent_list) = &mut dependents
                        .insert(
                            *dep,
                            match FixedVec::new(passes.size(), temp_allocator) {
                                Some(r) => r,
                                None => return Err(String::from_str("allocation failed"))
                        })
                        else {
                            return Err(String::from_str("allocation failed"))
                        };
                    dependent_list.push_back(*uid);
                }
            }
            
        }
        if in_degree.len() == 0 {
            let Some(mut sorted) = FixedVec::new(passes.size(), allocator) else {
                return Err(String::from_str("allocation failed"))
            };
            for pass in passes.iter() {
                sorted.push_back(pass.0);
            }
            return Ok(Self {
                passes,
                sorted,
            })
        }
        let Some(mut pending) = FixedVec::<UID>::new(passes.size(), temp_allocator) else {
            return Err(String::from_str("allocation failed"))
        };
        for (uid, deg) in in_degree.iter() {
            if *deg == 0 {
                pending.push_back(*uid);
            }
        }
        let Some(mut sorted) = FixedVec::new(passes.size(), allocator) else {
            return Err(String::from_str("allocation failed"))
        };
        while let Some(uid) = pending.pop_back() {
            //let pass = passes.get(uid).expect("UID not found");
            sorted.push_back(uid);
            if let Some(dependents) = dependents.get(&uid) {
                for dep_uid in dependents {
                    let count = in_degree.get_mut(dep_uid).unwrap();
                    *count -= 1;
                    if *count == 0 {
                        pending.push_back(*dep_uid);
                    }
                }
            }
        }
        if sorted.len() != passes.len() {
            Err(String::from_str("cycle detected"))
        }
        else {
            Ok(Self {
                passes,
                sorted,
            })
        }
    } 
}

impl<'a> Exec<'a> for FrameGraph<'a> {

    fn execute<'r, 'b, B: AllocateExt<'b>>(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        resource_pool: &mut ResourcePool<'a, 'r>,
        swapchain_image_resource: &mut ImageResource<'r>,
        funs: Option<&FixedMap<'a, UID, fn(UID)>>,
        temp_allocator: &mut B,
    ) -> Option<()> {
        for uid in &self.sorted {
            let pass = self.passes.get(uid).expect("couldn't find pass");
            if let Some(reads) = &pass.reads {
                for read in reads {
                    let image_resource = resource_pool.image_resources.get_mut(&read.0).expect("couldn't find resource");
                    let memory_barrier = image_resource.state
                        .to_memory_barrier(
                            *image_resource.handle,
                            &read.1,
                            image_resource.subresource_range,
                        );
                    unsafe {
                        device.cmd_pipeline_barrier(
                            command_buffer,
                            image_resource.state.pipeline_stage,
                            read.1.pipeline_stage,
                            Default::default(),
                            Default::default(),
                            Default::default(),
                            slice::from_ref(&memory_barrier),
                        );
                    }
                    image_resource.state = read.1;
                }
            }
            let mut render_extent = vk::Extent2D { width: u32::MAX, height: u32::MAX };
            let mut process_write = |write: &WriteInfo| {
                let image_resource = resource_pool.image_resources.get_mut(&write.resource_uid).expect("couldn't find resource");
                let memory_barrier = image_resource.state
                    .to_memory_barrier(
                        *image_resource.handle,
                        &write.image_state,
                        image_resource.subresource_range,
                    );
                unsafe {
                    device.cmd_pipeline_barrier(
                        command_buffer,
                        image_resource.state.pipeline_stage,
                        write.image_state.pipeline_stage,
                        Default::default(),
                        Default::default(),
                        Default::default(),
                        slice::from_ref(&memory_barrier),
                    );
                }
                render_extent.width = render_extent.width.min(image_resource.extent.width);
                render_extent.height = render_extent.height.min(image_resource.extent.height);
                image_resource.state = write.image_state;
                if *image_resource.handle == *swapchain_image_resource.handle {
                    *swapchain_image_resource = image_resource.clone();
                }
                vk::RenderingAttachmentInfo {
                    s_type: vk::StructureType::RENDERING_ATTACHMENT_INFO,
                    image_view: *image_resource.view,
                    image_layout: write.image_state.layout,
                    load_op: write.load_op,
                    store_op: write.store_op,
                    clear_value: write.clear_value,
                    ..Default::default()
                }
            };
            let mut color_outputs =
                if let Some(writes) = &pass.color_writes {
                    Some(FixedVec::<vk::RenderingAttachmentInfo>::new(writes.len(), temp_allocator)?)
                }
                else {
                    None
                };
            if let Some(writes) = &pass.color_writes {
                let attachments = color_outputs.as_mut().unwrap();
                for write in writes {
                    attachments.push_back(
                        process_write(write)
                    );
                }
            }
            let depth_output =
                if let Some(write) = &pass.depth_write {
                    Some(process_write(write))
                } else { None };
            let stencil_output =
                if let Some(write) = &pass.stencil_write {
                    Some(process_write(write))
                } else { None };
            if color_outputs.is_some() || depth_output.is_some() || stencil_output.is_some() {
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
                    color_attachment_count: if let Some(attachments) = &color_outputs { attachments.len() as u32 } else { 0 },
                    p_color_attachments: if let Some(attachments) = &color_outputs { attachments.as_ptr() as _ } else { 0 as _ },
                    p_depth_attachment: if let Some(attachment) = &depth_output { attachment } else { 0 as _ },
                    p_stencil_attachment: if let Some(attachment) = &stencil_output { attachment } else { 0 as _ },
                    ..Default::default()
                };
                unsafe {
                    device.cmd_begin_rendering(command_buffer, &rendering_info);
                }
            }
            if let Some(funs) = funs {
                if let Some(fun) = funs.get(uid) {
                    fun(*uid);
                }
            }
            unsafe { device.cmd_end_rendering(command_buffer); }
        }
        Some(())
    }
}
