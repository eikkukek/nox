use core::{
    slice,
};
use std::num::NonZero;

use ash::vk;

use nox_mem::{Allocator, CapacityError, FixedVec, Vector};

use crate::string_types::ArrayString;

use super::{
    ImageState,
    pipeline::{PipelineID, PipelineCache},
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
pub struct ResourceID {
    index: u32,
}

pub struct ResourcePool<'a, Alloc>
    where
        Alloc: Allocator
{
    images: FixedVec<'a, Image, Alloc>,
}

impl<'a, Alloc> ResourcePool<'a, Alloc>
    where
        Alloc: Allocator
{

    #[inline(always)]
    fn new() -> Self
    {
        Self{
            images: FixedVec::with_no_alloc(),
        }
    }

    #[inline(always)]
    pub fn add_image(&mut self, image: Image) -> ResourceID
    {
        let index = self.images.len();
        self.images
            .push(image)
            .expect("resource capacity exceeded");
        ResourceID { index: index as u32 }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, id: ResourceID) -> &mut Image {
        &mut self.images[id.index as usize]
    }
}

#[derive(Clone)]
pub struct WriteInfo {
    resource_id: ResourceID,
    format: NonZero<i32>,
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
    ) -> Option<Self> {
        Some(Self {
            resource_id,
            format: NonZero::new(format.as_raw())?,
            dst_state,
            load_op,
            store_op,
            clear_value,
            msaa_samples,
        })
    }

    pub fn format(&self) -> vk::Format {
        vk::Format::from_raw(self.format.get())
    }
}

#[derive(Clone, Copy)]
pub struct PassInfo {
    pub max_reads: u32,
    pub max_writes: u32,
    pub max_dependencies: u32,
    pub max_pipelines: u32,
    pub msaa_samples: vk::SampleCountFlags,
}

impl Default for PassInfo {

    fn default() -> Self {
        Self {
            max_reads: 0,
            max_writes: 0,
            max_dependencies: 0,
            max_pipelines: 0,
            msaa_samples: vk::SampleCountFlags::TYPE_1,
        }
    }
}

struct Pass<'alloc, Alloc: Allocator, Alloc2: Allocator> {
    reads: Option<FixedVec<'alloc, (ResourceID, ImageState), Alloc>>,
    writes: Option<FixedVec<'alloc, WriteInfo, Alloc>>,
    depth_write: Option<WriteInfo>,
    stencil_write: Option<WriteInfo>,
    dependencies: Option<FixedVec<'alloc, usize, Alloc>>,
    render_area: Option<vk::Rect2D>,
    pipeline_cache: &'alloc PipelineCache<'alloc, Alloc2>,
    pipelines: Option<FixedVec<'alloc, PipelineID, Alloc>>,
    callback: Option<fn(usize)>,
    last_pipeline_type_index: Option<u32>,
    msaa_samples: vk::SampleCountFlags,
    pub _name: PassName,
}

impl<'alloc, Alloc: Allocator, Alloc2: Allocator> Pass<'alloc, Alloc, Alloc2> {
    
    fn new(
        name: PassName,
        info: PassInfo,
        pipeline_cache: &'alloc PipelineCache<'alloc, Alloc2>,
        alloc: &'alloc Alloc
    ) -> Result<Self, CapacityError> {
        let reads =
            if info.max_reads != 0 {
                Some(FixedVec::with_capacity(info.max_reads as usize, alloc)?)
            }
            else {
                None
            };
        let writes =
            if info.max_writes != 0 {
                Some(FixedVec::with_capacity(info.max_writes as usize, alloc)?)
            }
            else {
                None
            };
        let dependencies =
            if info.max_dependencies != 0 {
                Some(FixedVec::with_capacity(info.max_dependencies as usize, alloc)?)
            }
            else {
                None
            };
        let pipelines =
            if info.max_pipelines != 0 {
                Some(FixedVec::with_capacity(info.max_pipelines as usize, alloc)?)
            }
            else {
                None
            };
        Ok(Self {
            reads,
            writes,
            depth_write: None.into(),
            stencil_write: None.into(),
            dependencies,
            render_area: None,
            pipeline_cache,
            pipelines,
            callback: None,
            last_pipeline_type_index: None,
            msaa_samples: info.msaa_samples,
            _name: name,
        })
    }
}

pub trait PassPipelineBuilder<'a> {

    fn with_pipeline(&mut self, id: PipelineID) -> &mut dyn PassPipelineBuilder<'a>;
}

pub trait PassAttachmentBuilder<'a> {

    fn with_read(&mut self, resource_id: ResourceID, dst_image_state: ImageState) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_depth_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_stencil_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_render_area(&mut self, offset: vk::Offset2D, extent: vk::Extent2D) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_dependency(&mut self, pass_index: usize) -> &mut dyn PassAttachmentBuilder<'a>;

    fn as_pipeline_builder(&mut self) -> &mut dyn PassPipelineBuilder<'a>;
}

impl<'a, Alloc: Allocator, Alloc2: Allocator> PassAttachmentBuilder<'a> for Pass<'a, Alloc, Alloc2> {

    fn as_pipeline_builder(&mut self) -> &mut dyn PassPipelineBuilder<'a> {
        self
    }

    fn with_read(&mut self, resource_id: ResourceID, dst_image_state: ImageState) -> &mut dyn PassAttachmentBuilder<'a> {
        self.reads
            .as_mut()
            .expect("read capacity exceeded")
            .push((resource_id, dst_image_state))
            .expect("read capacity exceeded");
        self
    }

    fn with_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a> {
        assert!(write.msaa_samples == self.msaa_samples,
            "write MSAA sample count must match pass sample count ( write: {:?}, pass: {:?} )",
            write.msaa_samples, self.msaa_samples);
        self.writes
            .as_mut()
            .expect("write capacity exceeded")
            .push(write)
            .expect("write capacity exceeded");
        self
    }

    fn with_depth_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a> {
        assert!(write.msaa_samples == self.msaa_samples, "write MSAA sample count must match pass sample count");
        self.depth_write = Some(write);
        self
    }

    fn with_stencil_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a> {
        assert!(write.msaa_samples == self.msaa_samples, "write MSAA sample count must match pass sample count");
        self.stencil_write = Some(write);
        self
    }

    fn with_render_area(&mut self, offset: vk::Offset2D, extent: vk::Extent2D) -> &mut dyn PassAttachmentBuilder<'a> {
        self.render_area = Some(vk::Rect2D { offset, extent });
        self
    }

    fn with_dependency(&mut self, pass_index: usize) -> &mut dyn PassAttachmentBuilder<'a> {
        self.dependencies
            .as_mut()
            .expect("dependency capacity exceeded")
            .push(pass_index)
            .expect("dependency capacity exceeded");
        self
    }
}

impl<'pass, Alloc: Allocator, Alloc2: Allocator> PassPipelineBuilder<'pass> for Pass<'pass, Alloc, Alloc2> {

    fn with_pipeline(&mut self, id: PipelineID) -> &mut dyn PassPipelineBuilder<'pass> {
        if self.last_pipeline_type_index.is_none_or(|v| v != id.type_index())
        {
            let type_info = self.pipeline_cache.get_type_info(id);
            assert!(type_info.msaa_samples() == self.msaa_samples, "pipeline MSAA sample count must match pass sample count");
            assert!(type_info.depth_format() == self.depth_write.as_ref().map_or(vk::Format::UNDEFINED, |r| r.format()),
                "pipeline depth format must match pass depth format");
            assert!(type_info.stencil_format() == self.stencil_write.as_ref().map_or(vk::Format::UNDEFINED, |r| r.format()),
                "pipeline stencil format must match pass stencil format");
            if let Some(writes) = &self.writes {
                let formats_len = type_info.color_formats().len();
                let writes_len = writes.len();
                assert!(formats_len <= writes_len, "pipeline color format count must be less or equal to pass color write count");
                for (i, format) in type_info.color_formats()[0..formats_len].iter().enumerate() {
                    assert!(writes[i].format() == *format, "pipeline color formats must match with pass color write formats");
                }
            }
            else {
                assert!(type_info.color_formats().len() == 0, "pipeline color format count must be less or equal to pass color write count");
            }
            self.last_pipeline_type_index = Some(id.type_index());
        } 
        self.pipelines
            .as_mut()
            .expect("pipeline capacity exceeded")
            .push(id)
            .expect("pipeline capacity exceeded");
        self
    }
}

pub trait FrameGraph<'a> {

    fn frame_index(&self) -> u32;

    fn set_render_image(&mut self, image: Image) -> ResourceID;

    fn add_resource(&mut self, image: Image) -> ResourceID;

    fn add_pass(
        &mut self,
        name: &str,
        info: PassInfo,
    ) -> Result<&mut dyn PassAttachmentBuilder<'a>, CapacityError>;

    fn with_pass(
        &mut self,
        name: &str,
        info: PassInfo,
        f: &mut dyn FnMut(&mut dyn PassAttachmentBuilder) -> Result<(), CapacityError>,
    ) -> Result<&mut dyn FrameGraph<'a>, CapacityError>;
}

pub trait FrameGraphInit<'a> {
    
    fn init(&mut self, max_passes: u32, max_resources: u32) -> Result<&mut dyn FrameGraph<'a>, CapacityError>;
}

pub struct FrameGraphImpl<'a, Alloc: Allocator, Alloc2: Allocator> {
    device: ash::Device,
    command_buffer: vk::CommandBuffer,
    alloc: &'a Alloc,
    pipeline_cache: &'a PipelineCache<'a, Alloc2>,
    device_alloc: &'a LinearDeviceAlloc,
    resource_pool: ResourcePool<'a, Alloc>,
    passes: FixedVec<'a, Pass<'a, Alloc, Alloc2>, Alloc>,
    render_image_id: Option<ResourceID>,
    frame_index: u32,
}

impl<'a, Alloc: Allocator, Alloc2: Allocator> FrameGraphImpl<'a, Alloc, Alloc2> {

    pub fn new(
        device: ash::Device,
        command_buffer: vk::CommandBuffer,
        alloc: &'a Alloc,
        pipeline_cache: &'a PipelineCache<'a, Alloc2>,
        device_alloc: &'a LinearDeviceAlloc,
        frame_index: u32,
    ) -> FrameGraphImpl<'a, Alloc, Alloc2>
    {
        FrameGraphImpl {
            device,
            command_buffer,
            alloc,
            pipeline_cache,
            device_alloc,
            resource_pool: ResourcePool::new(),
            passes: FixedVec::with_no_alloc(),
            render_image_id: None,
            frame_index,
        }
    }
}

impl<'a, Alloc: Allocator, Alloc2: Allocator> FrameGraphInit<'a> for FrameGraphImpl<'a, Alloc, Alloc2> {

    fn init(&mut self, max_passes: u32, max_resources: u32) -> Result<&mut dyn FrameGraph<'a>, CapacityError> {
        if max_passes != 0 {
            self.passes = FixedVec::with_capacity(max_passes as usize, self.alloc)?;
        }
        if max_resources != 0 {
            self.resource_pool.images = FixedVec::with_capacity(max_resources as usize, self.alloc)?;
        }
        Ok(self)
    }
}

impl<'a, Alloc: Allocator, Alloc2: Allocator> FrameGraph<'a> for FrameGraphImpl<'a, Alloc, Alloc2> {

    fn frame_index(&self) -> u32 {
        self.frame_index
    }

    fn set_render_image(&mut self, image: Image) -> ResourceID {
        assert!(self.render_image_id.is_none(), "render image already set");
        let id = self.resource_pool.add_image(image);
        self.render_image_id = Some(id);
        id
    }

    fn add_resource(&mut self, image: Image) -> ResourceID {
        self.resource_pool.add_image(image)
    }

    fn add_pass(
        &mut self,
        name: &str,
        info: PassInfo,
    ) -> Result<&mut dyn PassAttachmentBuilder<'a>, CapacityError>
    {
        let pass = Pass::new(
            ArrayString::from_str(name),
            info,
            self.pipeline_cache,
            self.alloc
        )?;
        Ok(self.passes
            .push(pass)
            .expect("pass capacity exceeded")
        )
    }

    fn with_pass(
        &mut self,
        name: &str,
        info: PassInfo,
        f: &mut dyn FnMut(&mut dyn PassAttachmentBuilder) -> Result<(), CapacityError>,
    ) -> Result<&mut dyn FrameGraph<'a>, CapacityError> {
        let pass = self.passes.push(Pass::new(
            ArrayString::from_str(name),
            info,
            self.pipeline_cache,
            self.alloc
        )?).expect("pass capacity exceeded");
        f(pass)?;
        Ok(self)
    }
}

impl<'alloc, Alloc: Allocator, Alloc2: Allocator> FrameGraphImpl<'alloc, Alloc, Alloc2> {

    pub fn render(&mut self) -> Result<Option<&mut Image>, RenderError> {
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
                    Some(process_write(&write)?)
                } else {
                    None
                };
            let stencil_output =
                if let Some(write) = &pass.stencil_write {
                    Some(process_write(write)?)
                } else {
                    None
                };
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
        Ok(self.render_image_id.map(|v| self.resource_pool.get_mut(v)))
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
