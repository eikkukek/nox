use super::{
    ImageState,
    handle::Handle,
};

use crate::{
    allocator_traits::{Allocate, Free},
    array_format,
    map_types::FixedMap,
    string_types::{ArrayString, LargeError},
    vec_types::{CapacityError, FixedVec, Vector}
};

use ash::vk;

use core::{
    slice,
    cell::RefCell,
};

pub type UID = u64;
pub type PassName = ArrayString<64>;

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

pub struct ResourcePool<'mem, 'r, A>
    where
        A: Allocate + Free,
        'mem: 'r,
{
    next_uid: UID,
    image_resources: FixedMap<'mem, UID, ImageResource<'r>, A>,
}

impl<'mem, 'r, A> ResourcePool<'mem, 'r, A>
    where
        A: Allocate + Free,
        'mem: 'r,
{

    pub fn new(
        max_images: usize,
        allocator: &'mem RefCell<A>,
    ) -> Result<Self, CapacityError>
    {
        Ok(Self{
            image_resources: FixedMap::with_capacity(max_images, allocator)?,
            next_uid: 0,
        })
    }

    pub fn add_image_resource(
        &mut self,
        resource: ImageResource<'r>
    ) -> Result<UID, CapacityError>
    {
        let uid = self.next_uid;
        self.image_resources.insert(uid, resource)?;
        self.next_uid = uid + 1;
        Ok(uid)
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

pub struct Pass<'mem, A: Allocate + Free> {
    reads: Option<FixedVec<'mem, (UID, ImageState), A>>,
    color_writes: Option<FixedVec<'mem, WriteInfo, A>>,
    depth_write: Option<WriteInfo>,
    stencil_write: Option<WriteInfo>,
    dependencies: Option<FixedVec<'mem, UID, A>>,
    render_area: Option<vk::Rect2D>,
    pub name: PassName,
}

impl<'mem, A: Allocate + Free> Pass<'mem, A> {
    
    pub fn new(
        name: PassName,
        max_reads: usize,
        max_color_writes: usize,
        max_dependencies: usize,
        allocator: &'mem RefCell<A>,
    ) -> Result<Self, CapacityError> {
        let reads =
            if max_reads != 0 {
                Some(FixedVec::with_capacity(max_reads, allocator)?)
            }
            else {
                None
            };
        let color_writes =
            if max_color_writes != 0 {
                Some(FixedVec::with_capacity(max_color_writes, allocator)?)
            }
            else {
                None
            };
        let dependencies =
            if max_dependencies != 0 {
                Some(FixedVec::with_capacity(max_color_writes, allocator)?)
            }
            else {
                None
            };
        Ok(Self {
            reads,
            color_writes,
            depth_write: None,
            stencil_write: None,
            dependencies,
            render_area: None,
            name,
        })
    }

    pub fn name(&mut self, name: &str) {
        self.name = ArrayString::from_str(name);
    }

    pub fn add_read(&mut self, uid: UID, dst_image_state: ImageState) -> Result<&mut Self, CapacityError> {
        if let Some(reads) = &mut self.reads {
            reads.push((uid, dst_image_state))?;
        }
        Ok(self)
    }

    pub fn add_color_write(&mut self, write: WriteInfo) -> Result<&mut Self, CapacityError> {
        if let Some(writes) = &mut self.color_writes {
            writes.push(write)?;
        }
        Ok(self)
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

    pub fn add_dependency(&mut self, uid: UID) -> Result<&mut Self, CapacityError> {
        if let Some(dependencies) = &mut self.dependencies {
            dependencies.push(uid)?;
        }
        Ok(self)
    }
}

pub trait Execute<'mem, 'r, A: Allocate + Free, B: Allocate + Free>
    where
        'mem: 'r,
{

    fn execute(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        resource_pool: &mut ResourcePool<'mem, 'r, A>,
        swapchain_image_resource: &RefCell<ImageResource<'r>>,
        funs: Option<&FixedMap<'mem, UID, fn(UID), A>>,
        temp_allocator: &RefCell<B>,
    ) -> Result<(), CapacityError>;
}

pub struct FrameGraph<'mem, A: Allocate + Free> {
    passes: FixedMap<'mem, UID, Pass<'mem, A>, A>,
    sorted: FixedVec<'mem, UID, A>,
}

impl<'mem, A: Allocate + Free> FrameGraph<'mem, A> {

    pub fn new<B: Allocate + Free>(
        passes: FixedMap<'mem, UID, Pass<'mem, A>, A>,
        allocator: &'mem RefCell<A>,
        temp_allocator: &'mem RefCell<B>,
    ) -> Result<Self, LargeError> {
        let mut in_degree = FixedMap::<UID, usize, B>
            ::with_capacity(passes.capacity(), temp_allocator)
            .map_err(|e| array_format!("failed to create 'in degree' ( {:?} )", e))?;
        let mut dependents = FixedMap::<UID, FixedVec<UID, B>, B>
            ::with_capacity(passes.len(), temp_allocator)
            .map_err(|e| array_format!("failed to create 'dependents' ( {:?} )", e))?;
        for (uid, pass) in passes.iter() {
            if let Some(deps) = &pass.dependencies {
                for dep in deps {
                    in_degree
                        .insert_or_modify(*uid, || 1, |v| *v += 1)
                        .map_err(|e| array_format!("failed to insert to 'in degree' ( {:?} )", e))?;
                    let dependent_list = &mut dependents
                        .insert(
                            *dep,
                            FixedVec
                                ::with_capacity(passes.capacity(), temp_allocator)
                                .map_err(|e| array_format!("failed to create 'dependent list' ( {:?} )", e))?
                        )
                        .map_err(|e| array_format!("failed to insert to 'dependents' ( {:?} )", e))?
                        .unwrap();
                    dependent_list
                        .push(*uid)
                        .map_err(|e| array_format!("failed to push to 'dependent list' ( {:?} )", e))?;
                }
            }
            
        }
        if in_degree.len() == 0 {
            let mut sorted = FixedVec
                ::with_capacity(passes.capacity(), allocator)
                .map_err(|e| array_format!("failed to create 'sorted' ( {:?} )", e))?;
            for pass in passes.iter() {
                sorted
                    .push(*pass.0)
                    .map_err(|e| array_format!("failed to push to 'sorted' ( {:?} )", e))?;
            }
            return Ok(Self {
                passes,
                sorted,
            })
        }
        let mut pending = FixedVec::<UID, B>
            ::with_capacity(passes.capacity(), temp_allocator)
            .map_err(|e| array_format!("failed to create 'pending' ( {:?} )", e))?;
        for (uid, deg) in in_degree.iter() {
            if *deg == 0 {
                pending
                    .push(*uid)
                    .map_err(|e| array_format!("failed to push to 'pending' ( {:?} )", e))?;
            }
        }
        let mut sorted = FixedVec
            ::with_capacity(passes.capacity(), allocator)
            .map_err(|e| array_format!("failed to create 'sorted' ( {:?} )", e))?;
        while let Some(uid) = pending.pop() {
            //let pass = passes.get(uid).expect("UID not found");
            sorted
                .push(uid)
                .map_err(|e| array_format!("failed to push to 'sorted' ( {:?} )", e))?;
            if let Some(dependents) = dependents.get(&uid) {
                for dep_uid in dependents {
                    let count = in_degree.get_mut(dep_uid).unwrap();
                    *count -= 1;
                    if *count == 0 {
                        pending
                            .push(*dep_uid)
                            .map_err(|e| array_format!("failed to push to 'pending' ( {:?} )", e))?;
                    }
                }
            }
        }
        if sorted.len() != passes.len() {
            Err(ArrayString::from_str("cycle detected"))
        }
        else {
            Ok(Self {
                passes,
                sorted,
            })
        }
    } 
}

impl<'mem, 'r, A, B> Execute<'mem, 'r, A, B> for FrameGraph<'mem, A>
    where
        A: Allocate + Free,
        B: Allocate + Free,
        'mem: 'r,
{

    fn execute(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        resource_pool: &mut ResourcePool<'mem, 'r, A>,
        swapchain_image_resource: &RefCell<ImageResource<'r>>,
        funs: Option<&FixedMap<'mem, UID, fn(UID), A>>,
        temp_allocator: &RefCell<B>,
    ) -> Result<(), CapacityError> 
    {
        for uid in &self.sorted {
            let pass = self.passes.get(uid).expect("couldn't find pass");
            if let Some(reads) = &pass.reads {
                for read in reads {
                    let image_resource = resource_pool.image_resources.get_mut(&read.0).expect("couldn't find resource");
                    let memory_barrier = image_resource.state
                        .to_memory_barrier(
                            image_resource.handle.clone(),
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
                        image_resource.handle.clone(),
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
                if *image_resource.handle == *swapchain_image_resource.borrow().handle {
                    *swapchain_image_resource.borrow_mut() = image_resource.clone();
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
                    Some(FixedVec::<vk::RenderingAttachmentInfo, B>
                        ::with_capacity(writes.len(), temp_allocator)
                        .map_err(|e| e)?
                    )
                }
                else {
                    None
                };
            if let Some(writes) = &pass.color_writes {
                let attachments = color_outputs.as_mut().unwrap();
                for write in writes {
                    attachments
                        .push(process_write(write))?;
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
        Ok(())
    }
}
