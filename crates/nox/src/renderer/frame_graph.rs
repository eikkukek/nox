use core::slice;

use ash::vk;

use nox_mem::{Allocator, CapacityError, FixedVec, Vector};

use crate::string_types::ArrayString;

use super::{
    ImageState,
    pipeline_cache::{PipelineID, PipelineCache},
    physical_device::QueueFamilyIndices,
    linear_device_alloc::{self, LinearDeviceAlloc},
};

pub type PassName = ArrayString<64>;

#[derive(Debug)]
pub enum RenderError {
    AllocError(CapacityError),
    DeviceAllocError(linear_device_alloc::Error),
}

impl From<CapacityError> for RenderError {

    fn from(value: CapacityError) -> Self {
        Self::AllocError(value)
    }
}

impl From<linear_device_alloc::Error> for RenderError {

    fn from(value: linear_device_alloc::Error) -> Self {
        Self::DeviceAllocError(value)
    }
}

#[derive(Clone, Copy)]
pub struct ImageResource {
    image: vk::Image,
    image_view: vk::ImageView,
    state: ImageState,
}

impl ImageResource {

    #[inline(always)]
    pub fn new(
        image: vk::Image,
        image_view: vk::ImageView,
        state: ImageState,
    ) -> Self
    {
        Self {
            image,
            image_view,
            state,
        }
    }

    #[inline(always)]
    pub fn state(&self) -> ImageState {
        self.state
    }
}

#[derive(Clone)]
pub struct Image {
    resource: Option<ImageResource>,
    device: Option<ash::Device>, // if Some, resource destroyed on drop
    format: vk::Format,
    extent: vk::Extent2D,
    usage: vk::ImageUsageFlags,
    samples: vk::SampleCountFlags,
    subresource_range: vk::ImageSubresourceRange,
}

impl Image {

    #[inline(always)]
    pub fn new(
        format: vk::Format,
        extent: vk::Extent2D,
        usage: vk::ImageUsageFlags,
        samples: vk::SampleCountFlags,
        subresource_range: vk::ImageSubresourceRange,
    ) -> Self {
        Self {
            resource: None,
            device: None,
            format,
            extent,
            usage,
            samples,
            subresource_range,
        }
    }

    #[inline(always)]
    pub fn with_resource(
        resource: ImageResource,
        format: vk::Format,
        extent: vk::Extent2D,
        usage: vk::ImageUsageFlags,
        samples: vk::SampleCountFlags,
        subresource_range: vk::ImageSubresourceRange,
    ) -> Self {
        Self {
            resource: Some(resource),
            device: None,
            format,
            extent,
            usage,
            samples,
            subresource_range,
        }
    }

    #[inline(always)]
    pub fn create_resource<'alloc>(&mut self, device_alloc: &'alloc LinearDeviceAlloc) -> linear_device_alloc::Result<()> {
        assert!(self.resource.is_none(), "attempting to override resource");
        let image_create_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            image_type: vk::ImageType::TYPE_2D,
            format: self.format,
            extent: vk::Extent3D {
                width: self.extent.width,
                height: self.extent.height,
                depth: 1,
                },
            mip_levels: 1,
            array_layers: 1,
            samples: self.samples,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: self.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            ..Default::default()
        };
        let device = device_alloc.device();
        let image = unsafe { device
            .create_image(&image_create_info, None)?
        };
        device_alloc
            .bind_image_memory(image)
            .map_err(|e| {
                unsafe {
                    device.destroy_image(image, None);
                }
                e
            })?;
        let view_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: self.format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
                },
            subresource_range: self.subresource_range,
            ..Default::default()
        };
        let view = unsafe { device
            .create_image_view(&view_create_info, None)
            .map_err(|e| {
                device.destroy_image(image, None);
                e
            })?
        };
        self.resource = Some(ImageResource::new(
            image,
            view,
            ImageState::new(
                vk::AccessFlags::NONE,
                vk::ImageLayout::UNDEFINED,
                vk::QUEUE_FAMILY_IGNORED,
                vk::PipelineStageFlags::TOP_OF_PIPE,
            ),
        ));
        self.device = Some(device);
        Ok(())
    }

    #[inline(always)]
    pub fn resource(&self) -> Option<ImageResource> {
        self.resource
    }

    #[inline(always)]
    pub fn format(&self) -> vk::Format {
        self.format
    }

    #[inline(always)]
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    #[inline(always)]
    pub fn usage(&self) -> vk::ImageUsageFlags {
        self.usage
    }

    #[inline(always)]
    pub fn samples(&self) -> vk::SampleCountFlags {
        self.samples
    }

    #[inline(always)]
    pub fn subresource_range(&self) -> &vk::ImageSubresourceRange {
        &self.subresource_range
    }
}

impl Drop for Image {

    fn drop(&mut self) {
        if let Some(device) = self.device.take() {
            unsafe {
                let resource = self.resource.unwrap_unchecked();
                device.destroy_image_view(resource.image_view, None);
                device.destroy_image(resource.image, None);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct ResourceID(u32);

impl ResourceID {

    pub const fn render_image_id() -> Self {
        Self(0)
    }
}

pub struct ResourcePool<'alloc, Alloc>
    where
        Alloc: Allocator
{
    images: FixedVec<'alloc, Image, Alloc>,
    render_image: &'alloc mut Image,
}

impl<'alloc, Alloc> ResourcePool<'alloc, Alloc>
    where
        Alloc: Allocator
{

    #[inline(always)]
    fn new(render_image: &'alloc mut Image) -> Self
    {
        Self{
            images: FixedVec::with_no_alloc(),
            render_image,
        }
    }

    #[inline(always)]
    pub fn add_image(&mut self, image: Image) -> Result<ResourceID, CapacityError>
    {
        let index = self.images.len();
        self.images.push(image)?;
        Ok(ResourceID(index as u32 + 1))
    }

    #[inline(always)]
    pub fn get_mut(&mut self, id: ResourceID) -> &mut Image {
        if id.0 == 0 {
            self.render_image
        }
        else {
            &mut self.images[id.0 as usize - 1]
        }
    }
}

#[derive(Clone)]
pub struct WriteInfo {
    resource_id: ResourceID,
    format: vk::Format,
    dst_state: ImageState,
    load_op: vk::AttachmentLoadOp,
    store_op: vk::AttachmentStoreOp,
    clear_value: vk::ClearValue,
    msaa_samples: vk::SampleCountFlags
}

impl WriteInfo {

    pub fn new(
        resource_id: ResourceID,
        format: vk::Format,
        dst_state: ImageState,
        load_op: vk::AttachmentLoadOp,
        store_op: vk::AttachmentStoreOp,
        clear_value: vk::ClearValue,
        msaa_samples: vk::SampleCountFlags
    ) -> Self {
        Self {
            resource_id,
            format,
            dst_state,
            load_op,
            store_op,
            clear_value,
            msaa_samples,
        }
    }
}

pub struct Pass<'alloc, Alloc: Allocator> {
    reads: Option<FixedVec<'alloc, (ResourceID, ImageState), Alloc>>,
    writes: Option<FixedVec<'alloc, WriteInfo, Alloc>>,
    depth_write: Option<WriteInfo>,
    stencil_write: Option<WriteInfo>,
    dependencies: Option<FixedVec<'alloc, usize, Alloc>>,
    render_area: Option<vk::Rect2D>,
    pipelines: Option<FixedVec<'alloc, PipelineID, Alloc>>,
    callback: Option<fn(usize)>,
    msaa_samples: vk::SampleCountFlags,
    pub name: PassName,
}

impl<'alloc, Alloc: Allocator> Pass<'alloc, Alloc> {
    
    fn new(
        name: PassName,
        max_reads: u32,
        max_color_writes: u32,
        max_dependencies: u32,
        msaa_samples: vk::SampleCountFlags,
        alloc: &'alloc Alloc
    ) -> Result<Self, CapacityError> {
        let reads =
            if max_reads != 0 {
                Some(FixedVec::with_capacity(max_reads as usize, alloc)?)
            }
            else {
                None
            };
        let writes =
            if max_color_writes != 0 {
                Some(FixedVec::with_capacity(max_color_writes as usize, alloc)?)
            }
            else {
                None
            };
        let dependencies =
            if max_dependencies != 0 {
                Some(FixedVec::with_capacity(max_color_writes as usize, alloc)?)
            }
            else {
                None
            };
        Ok(Self {
            reads,
            writes,
            depth_write: None,
            stencil_write: None,
            dependencies,
            render_area: None,
            pipelines: None,
            callback: None,
            msaa_samples,
            name,
        })
    }
}

pub struct PassPipelineBuilder<'pass, Alloc: Allocator> {
    pass: &'pass mut Pass<'pass, Alloc>,
    last_pipeline_type_id: Option<PipelineID>,
}

impl<'pass, Alloc: Allocator> PassPipelineBuilder<'pass, Alloc> {

    pub fn finish(self) {}
}

pub struct PassAttachmentBuilder<'pass, Alloc: Allocator> {
    pass: &'pass mut Pass<'pass, Alloc>,
    pipeline_cache: &'pass PipelineCache<'pass>,
}

impl<'pass, Alloc: Allocator> PassAttachmentBuilder<'pass, Alloc> {

    pub fn finish(self) -> PassPipelineBuilder<'pass, Alloc> {
        PassPipelineBuilder {
            pass: self.pass,
            last_pipeline_type_id: None,
        }
    }

    pub fn with_read(&mut self, resource_id: ResourceID, dst_image_state: ImageState) -> &mut Self {
        self.pass.reads
            .as_mut()
            .expect("no reads set")
            .push((resource_id, dst_image_state))
            .expect("read capacity exceeded");
        self
    }

    pub fn with_write(&mut self, write: WriteInfo) -> &mut Self {
        assert!(write.msaa_samples == self.pass.msaa_samples, "write MSAA sample count must match pass sample count");
        self.pass.writes
            .as_mut()
            .expect("no writes set")
            .push(write)
            .expect("write capacity exceeded");
        self
    }

    pub fn with_depth_write(&mut self, write: WriteInfo) -> &mut Self {
        assert!(write.msaa_samples == self.pass.msaa_samples, "write MSAA sample count must match pass sample count");
        self.pass.depth_write = Some(write);
        self
    }

    pub fn with_stencil_write(&mut self, write: WriteInfo) -> &mut Self {
        assert!(write.msaa_samples == self.pass.msaa_samples, "write MSAA sample count must match pass sample count");
        self.pass.stencil_write = Some(write);
        self
    }

    pub fn with_render_area(&mut self, offset: vk::Offset2D, extent: vk::Extent2D) -> &mut Self {
        self.pass.render_area = Some(vk::Rect2D { offset, extent });
        self
    }

    pub fn with_dependency(&mut self, pass_index: usize) -> &mut Self {
        self.pass.dependencies
            .as_mut()
            .expect("no dependencies set")
            .push(pass_index)
            .expect("dependency capacity exceeded");
        self
    }
}

pub struct FrameGraph<'alloc, Alloc: Allocator> {
    device: ash::Device,
    command_buffer: vk::CommandBuffer,
    alloc: &'alloc Alloc,
    pipeline_cache: &'alloc PipelineCache<'alloc, Alloc>,
    device_alloc: &'alloc LinearDeviceAlloc,
    resource_pool: ResourcePool<'alloc, Alloc>,
    queue_family_indices: QueueFamilyIndices,
    passes: FixedVec<'alloc, Pass<'alloc, Alloc>, Alloc>,
    frame_index: u32,
}

pub struct FrameGraphInit<'alloc, Alloc: Allocator> {
    frame_graph: FrameGraph<'alloc, Alloc>,
}

pub fn new<'alloc, Alloc: Allocator>(
    device: ash::Device,
    command_buffer: vk::CommandBuffer,
    render_image: &'alloc mut Image,
    alloc: &'alloc Alloc,
    pipeline_cache: &'alloc PipelineCache<'alloc, Alloc>,
    device_alloc: &'alloc LinearDeviceAlloc,
    queue_family_indices: QueueFamilyIndices,
    frame_index: u32,
) -> FrameGraphInit<'alloc, Alloc>
{
    FrameGraphInit {
        frame_graph: FrameGraph
        {
            device,
            command_buffer,
            alloc,
            pipeline_cache,
            device_alloc,
            resource_pool: ResourcePool::new(render_image),
            queue_family_indices,
            passes: FixedVec::with_no_alloc(),
            frame_index,
        }
    }
}

impl<'alloc, Alloc: Allocator> FrameGraphInit<'alloc, Alloc> {

    pub fn init(mut self, max_passes: u32, max_resources: u32) -> Result<FrameGraph<'alloc, Alloc>, CapacityError> {
        if max_passes != 0 {
            self.frame_graph.passes = FixedVec::with_capacity(max_passes as usize, self.frame_graph.alloc)?;
        }
        if max_resources != 0 {
            self.frame_graph.resource_pool.images = FixedVec::with_capacity(max_resources as usize, self.frame_graph.alloc)?;
        }
        Ok(self.frame_graph)
    }
}

impl<'alloc, Alloc: Allocator> FrameGraph<'alloc, Alloc> {

    pub fn frame_index(&self) -> u32 {
        self.frame_index
    }

    pub fn queue_family_indices(&self) -> &QueueFamilyIndices {
        &self.queue_family_indices
    }

    pub fn add_pass(
        &mut self,
        name: &str,
        max_reads: u32,
        max_color_writes: u32,
        max_dependencies: u32,
        msaa_samples: vk::SampleCountFlags
    ) -> Result<PassAttachmentBuilder<'_, 'alloc, Alloc>, CapacityError> {
        let pass = Pass::new(
            ArrayString::from_str(name),
            max_reads,
            max_color_writes,
            max_dependencies,
            msaa_samples,
            self.alloc
        )?;
        Ok(PassAttachmentBuilder { pass: self.passes
            .push(pass)
            .expect("pass capacity exceeded")
        })
    }

    pub fn add_pass_fn<F>(
        &mut self,
        max_reads: u32,
        max_color_writes: u32,
        max_dependencies: u32,
        msaa_samples: vk::SampleCountFlags,

    ) -> Result<(), CapacityError>
        where
            F: FnMut(PassAttachmentBuilder<'_, 'alloc, Alloc>) -> Result<(), CapacityError>
    {
        todo!()
    }

    pub fn render(&mut self) -> Result<(), RenderError> {
        let sorted = self.sort()?;
        for index in sorted.iter().map(|i| *i) {
            let pass = &self.passes[index];
            if let Some(reads) = &pass.reads {
                for read in reads {
                    let image = self.resource_pool.get_mut(read.0);
                    let Some(mut image_resource) = image.resource.take() else {
                        panic!("read image resource was none")
                    };
                    let memory_barrier = image_resource.state
                        .to_memory_barrier(
                            image_resource.image,
                            &read.1,
                            image.subresource_range,
                        );
                    unsafe {
                        self.device.cmd_pipeline_barrier(
                            self.command_buffer,
                            image_resource.state.pipeline_stage,
                            read.1.pipeline_stage,
                            Default::default(),
                            Default::default(),
                            Default::default(),
                            slice::from_ref(&memory_barrier),
                        );
                    }
                    image_resource.state = read.1;
                    image.resource = Some(image_resource);
                }
            }
            let mut render_extent = vk::Extent2D { width: u32::MAX, height: u32::MAX };
            let mut process_write = |write: &WriteInfo| -> Result<vk::RenderingAttachmentInfo, linear_device_alloc::Error> {
                let image = self.resource_pool.get_mut(write.resource_id);
                if image.resource.is_none() {
                    image.create_resource(self.device_alloc)?;
                }
                let dst_state = write.dst_state;
                let mut image_resource = image.resource.unwrap();
                let memory_barrier = image_resource.state
                    .to_memory_barrier(
                        image_resource.image,
                        &dst_state,
                        image.subresource_range,
                    );
                unsafe {
                    self.device.cmd_pipeline_barrier(
                        self.command_buffer,
                        image_resource.state.pipeline_stage,
                        dst_state.pipeline_stage,
                        Default::default(),
                        Default::default(),
                        Default::default(),
                        slice::from_ref(&memory_barrier),
                    );
                }
                render_extent.width = render_extent.width.min(image.extent.width);
                render_extent.height = render_extent.height.min(image.extent.height);
                image_resource.state = dst_state;
                image.resource = Some(image_resource);
                Ok(vk::RenderingAttachmentInfo {
                    s_type: vk::StructureType::RENDERING_ATTACHMENT_INFO,
                    image_view: image_resource.image_view,
                    image_layout: image_resource.state.layout,
                    load_op: write.load_op,
                    store_op: write.store_op,
                    clear_value: write.clear_value,
                    ..Default::default()
                })
            };
            let mut color_outputs =
                if let Some(writes) = &pass.writes {
                    Some(FixedVec::<vk::RenderingAttachmentInfo, Alloc>
                        ::with_capacity(writes.len(), self.alloc)?
                    )
                }
                else {
                    None
                };
            if let Some(writes) = &pass.writes {
                let attachments = color_outputs.as_mut().unwrap();
                for write in writes {
                    attachments
                        .push(process_write(write)?)?;
                }
            }
            let depth_output =
                if let Some(write) = &pass.depth_write {
                    Some(process_write(write)?)
                } else { None };
            let stencil_output =
                if let Some(write) = &pass.stencil_write {
                    Some(process_write(write)?)
                } else { None };
            let (color_attachment_count, p_color_attachments) = match &color_outputs {
                Some(r) => {
                    (r.len() as u32, r.as_ptr())
                },
                None => {
                    (0, core::ptr::null())
                },
            };
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
                    color_attachment_count,
                    p_color_attachments,
                    p_depth_attachment: if let Some(attachment) = &depth_output { attachment } else { 0 as _ },
                    p_stencil_attachment: if let Some(attachment) = &stencil_output { attachment } else { 0 as _ },
                    ..Default::default()
                };
                unsafe {
                    self.device.cmd_begin_rendering(self.command_buffer, &rendering_info);
                }
            }
            if let Some(callback) = pass.callback {
                callback(index)
            }
            unsafe { self.device.cmd_end_rendering(self.command_buffer); }
        }
        Ok(())
    }

    #[inline(always)]
    fn sort(&self) -> Result<FixedVec<'alloc, usize, Alloc>, CapacityError> {
        if self.passes.len() == 0 {
            return Ok(FixedVec::with_no_alloc())
        }
        let mut in_degree = FixedVec::<usize, Alloc>
            ::with_len(self.passes.len(), 0, self.alloc)?;
        let mut dependents = FixedVec::<FixedVec<usize, Alloc>, Alloc>
            ::with_len_with(self.passes.len(), FixedVec::with_no_alloc, self.alloc)?;
        for (i, pass) in self.passes.iter().enumerate() {
            if let Some(deps) = &pass.dependencies {
                in_degree[i] = deps.len();
                for index in deps {
                    let list = &mut dependents[*index];
                    if list.capacity() == 0 {
                        *list =
                            FixedVec::with_capacity(self.passes.len(), self.alloc)?;
                    }
                    list.push(i)?;
                }
            }
            
        }
        if in_degree.len() == 0 {
            let mut i = 0;
            return
                FixedVec
                ::with_len_with(self.passes.capacity(),
                    || {
                        let index = i;
                        i += 1;
                        index
                    },
                    self.alloc
                )
        }
        let mut pending = FixedVec::<usize, Alloc>
            ::with_capacity(self.passes.capacity(), self.alloc)?;
        for (i, deg) in in_degree.iter().enumerate() {
            if *deg == 0 {
                pending
                    .push(i)?;
            }
        }
        let mut sorted = FixedVec
            ::with_capacity(self.passes.capacity(), self.alloc)?;
        while let Some(index) = pending.pop() {
            //let pass = passes.get(uid).expect("UID not found");
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
        if sorted.len() != self.passes.len() {
            panic!("cycle detected")
        }
        else {
            Ok(sorted)
        }
    }
}
