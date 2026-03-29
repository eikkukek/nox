use std::fs;

use core::{
    ptr::NonNull,
    slice,
    time::Duration,
};

use ahash::AHashMap;

use ::image::EncodableLayout;

use nox::{
    gpu,
    error::*,
    threads::executor::block_on,
    expand_warn,
};

use gpu::MemoryBinder;

use nox_geom::*;

use crate::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ImageSource<'a> {
    Id(gpu::ImageViewId),
    Path(&'a str),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ImageSourceOwned {
    Id(gpu::ImageViewId),
    Path(CompactString),
}

#[derive(Clone, Copy)]
pub enum ImageSourceUnsafe {
    Id(gpu::ImageViewId),
    Path(NonNull<u8>, usize),
}

unsafe impl Send for ImageSourceUnsafe {}
unsafe impl Sync for ImageSourceUnsafe {}

impl ImageSourceUnsafe {
    
    /// Converts self into [`ImageSource`].
    ///
    /// # Safety
    /// If this is a [`path`][1], it has to be ensured that the pointer is a valid pointer up to
    /// size.
    ///
    /// [1]: Self::Path
    pub unsafe fn as_image_source<'a>(&'a self) -> ImageSource<'a> {
        match *self {
            Self::Path(data, len) => unsafe {
                ImageSource::Path(str
                    ::from_utf8(slice::from_raw_parts(data.as_ptr(), len))
                    .unwrap_or_default()
                )
            },
            Self::Id(id) => ImageSource::Id(id)
        }
    }
}

impl<'a> From<ImageSource<'a>> for ImageSourceOwned {

    fn from(value: ImageSource<'a>) -> Self {
        match value {
            ImageSource::Path(p) => Self::Path(p.into()),
            ImageSource::Id(id) => Self::Id(id),
        }
    }
}


#[derive(Clone, Copy,  Eq)]
pub struct ImageSourceInternal {
    pub view_id: gpu::ImageViewId,
}

impl PartialEq for ImageSourceInternal {

    fn eq(&self, other: &Self) -> bool {
        self.view_id == other.view_id
    }
}

#[macro_export]
macro_rules! image_source {
    ($path:tt) => {
        ImageSource::Path(&$path)
    };
    ($texture:expr) => {
        ImageSource::Id($texture)
    };
}

type ImageBuffer = ::image::ImageBuffer<::image::Rgba<u8>, Vec<u8>>;

struct LoadedImage {
    last_modified: std::time::SystemTime,
    last_updated: u64,
    source: ImageBuffer,
    staging_binder: gpu::LinearBinder,
    staging_id: gpu::BufferId,
    view_binder: gpu::LinearBinder,
    view_id: gpu::ImageViewId,
}

pub struct ImageLoader {
    gpu: gpu::Gpu,
    images: AHashMap<CompactString, LoadedImage>,
    semaphore: gpu::TimelineSemaphoreId,
    semaphore_value: u64,
    err_view_id: gpu::ImageViewId,
    err_staging_id: gpu::BufferId,
    err_image_loaded: bool,
}

impl ImageLoader {

    const ERR_BYTES: &[u8] =
    &[
        255, 0, 203, 255,
        0, 0, 0, 255,
        0, 0, 0, 255,
        255, 0, 203, 255,
    ];

    #[inline(always)]
    pub fn new(
        gpu: gpu::Gpu,
    ) -> Result<Self>
    {
        let (mut err_staging_id, mut err_image_id) = Default::default();
        let memory_binder = gpu::GlobalBinder::new(
            gpu.device().clone(),
            gpu::MemoryProperties::DEVICE_LOCAL,
            gpu::MemoryProperties::DEVICE_LOCAL
        );
        gpu.create_resources(
            [gpu::BufferCreateInfo::new(
                &mut err_staging_id,
                &memory_binder,
                size_of_val(Self::ERR_BYTES) as gpu::DeviceSize,
                gpu::BufferUsages::TRANSFER_SRC | gpu::BufferUsages::TRANSFER_DST
            ).unwrap()],
            [gpu::ImageCreateInfo
                ::new(
                    &mut err_image_id,
                    &memory_binder,
                ).with_dimensions((2, 2))
                .with_format(gpu::Format::R8g8b8a8Srgb, false)
                .with_usage(gpu::ImageUsages::TRANSFER_DST | gpu::ImageUsages::SAMPLED)
            ])?;
        let err_view_id = gpu.create_image_view(
            err_image_id,
            gpu::ImageRange::whole_range(gpu::ImageAspects::COLOR),
        )?;
        let mut semaphore = Default::default();
        gpu.create_timeline_semaphores([(&mut semaphore, 0)])?;
        Ok(Self {
            gpu,
            images: AHashMap::default(),
            semaphore,
            semaphore_value: 0,
            err_view_id,
            err_staging_id,
            err_image_loaded: false,
        })
    }

    fn load_err_image(&mut self) -> (ImageSourceInternal, Option<gpu::CommandDependency>) {
        let mut command_id = None;
        if !self.err_image_loaded {
            let (err_staging_id, err_view_id) = (self.err_staging_id, self.err_view_id);
            let mut scheduler = self.gpu.schedule_commands();
            let mut cmd = scheduler.new_commands::<gpu::NewCopyCommands>(
                self.gpu.any_device_queue(gpu::QueueFlags::GRAPHICS).unwrap(),
                move |cmd| {
                    cmd.update_buffer(err_staging_id, 0, Self::ERR_BYTES, gpu::CommandOrdering::Lenient)?;
                    cmd.copy_buffer_to_image(
                        err_staging_id,
                        err_view_id.image_id(),
                        &[gpu::BufferImageCopy
                            ::default()
                            .image_subresource(gpu::ImageSubresourceLayers
                                ::default()
                                .aspect_mask(gpu::ImageAspects::COLOR)
                            ).image_extent((2, 2))
                        ],
                        gpu::CommandOrdering::Strict
                    )?;
                    Ok(())
                }
            ).ok();
            command_id = cmd.take().map(|cmd| {
                let cmd = cmd.with_signal_semaphore(self.semaphore, self.semaphore_value + 1);
                cmd.id()
            });
            self.semaphore_value += 1;
        }
        self.err_image_loaded = true;
        (
            ImageSourceInternal { view_id: self.err_view_id, },
            command_id
                .map(|id| gpu::CommandDependency::new(id, gpu::MemoryDependencyHint::FRAGMENT_SHADER))
        )
    }

    #[inline(always)]
    pub fn load_image(&mut self, path: &str) -> (ImageSourceInternal, Option<gpu::CommandDependency>) {
        if let Some(img) = self.images.get_mut(path) &&
            let Ok(meta) = fs::metadata(path) &&
            let Ok(modified) = meta.modified()
        {
            if modified == img.last_modified {
                return (ImageSourceInternal {
                    view_id: img.view_id,
                }, None)
            }
            if let Ok(new_img) = load_rgba_image(path) {
                let mut command_id = Default::default();
                match block_on(async {
                    if !self.gpu.wait_for_semaphores(&[(self.semaphore, img.last_updated)],
                        Duration::from_secs(2)
                    )? {
                        return Err(Error::just_context("semaphore wait timeout"))
                    }
                    unsafe {
                        img.staging_binder.release_resources();
                        img.view_binder.release_resources();
                    }
                    self.gpu.destroy_resources([img.staging_id], [img.view_id.image_id()])?;
                    let (width, height) = new_img.dimensions();
                    let mem_size = (width * height) as gpu::DeviceSize * 4;
                    let mip_levels = 32 - (width | height).leading_zeros();
                    let (mut staging_id, mut image_id) = Default::default();
                    self.gpu.create_resources(
                        [gpu::BufferCreateInfo::new(
                            &mut staging_id,
                            &img.staging_binder,
                            mem_size,
                            gpu::BufferUsages::TRANSFER_SRC,
                        ).unwrap()],
                        [gpu::ImageCreateInfo
                            ::new(&mut image_id, &img.view_binder)
                            .with_dimensions((width, height))
                            .with_format(gpu::Format::R8g8b8a8Srgb, false)
                            .with_usage(gpu::ImageUsages::TRANSFER_DST | gpu::ImageUsages::SAMPLED)
                            .with_mip_levels(mip_levels)
                        ])?;
                    let mut map = self.gpu.map_buffer(staging_id)?;
                    unsafe {
                        map.write_bytes(new_img.as_bytes());
                    }
                    if !map.is_coherent {
                        self.gpu.flush_mapped_memory_ranges(&[
                            gpu::MappedBufferMemoryRange {
                                buffer_id: staging_id,
                                offset: 0,
                                size: mem_size,
                            },
                        ])?;
                    }
                    let view_id = self.gpu.create_image_view(
                        image_id,
                        gpu::ImageRange::whole_range(gpu::ImageAspects::COLOR),
                    )?;
                    command_id = self.gpu
                        .schedule_commands()
                        .new_commands::<gpu::NewCopyCommands>(
                            self.gpu.any_device_queue(gpu::QueueFlags::GRAPHICS).unwrap(),
                            move |cmd| {
                                cmd.copy_buffer_to_image(
                                    staging_id,
                                    image_id,
                                    &[gpu::BufferImageCopy
                                        ::default()
                                        .image_subresource(gpu::ImageSubresourceLayers
                                            ::default()
                                            .aspect_mask(gpu::ImageAspects::COLOR)
                                        ).image_extent((width, height))
                                    ],
                                    gpu::CommandOrdering::Lenient,
                                )?;
                                Ok(())
                            }
                        )?.with_signal_semaphore(self.semaphore, self.semaphore_value + 1).id();
                    self.semaphore_value += 1;
                    img.staging_id = staging_id;
                    img.view_id = view_id;
                    img.last_updated = self.semaphore_value;
                    img.source = new_img;
                    img.last_modified = modified;
                    Result::Ok(())
                }) {
                    Ok(()) => {
                        return (ImageSourceInternal {
                            view_id: img.view_id,
                        }, Some(gpu::CommandDependency::new(
                            command_id, gpu::MemoryDependencyHint::FRAGMENT_SHADER,
                        )))
                    },
                    Err(err) => {
                        expand_warn!(Error::new(
                            err, "(nox_gui) failed to load image to gpu memory",
                        ));
                    },
                }
            }
            return self.load_err_image()
        }
        if let Ok(meta) = fs::metadata(path) &&
            let Ok(last_modified) = meta.modified() &&
            let Ok(new_img) = load_rgba_image(path)
        {
            let mut command_id = Default::default();
            match block_on(async {
                let (width, height) = new_img.dimensions();
                let buf_size = new_img.as_bytes().len() as gpu::DeviceSize;
                let mip_levels = 32 - (width | height).leading_zeros();
                let img_size: gpu::DeviceSize = (0..mip_levels).map(|i| buf_size >> i).sum();
                let staging_binder = gpu::LinearBinder
                    ::new(
                        self.gpu.device().clone(),
                        buf_size,
                        gpu::MemoryProperties::HOST_VISIBLE,
                        gpu::MemoryProperties::HOST_VISIBLE | gpu::MemoryProperties::HOST_COHERENT,
                    )?;
                let view_binder = gpu::LinearBinder
                    ::new(
                        self.gpu.device().clone(),
                        img_size,
                        gpu::MemoryProperties::DEVICE_LOCAL,
                        gpu::MemoryProperties::DEVICE_LOCAL
                    )?;
                let (mut staging_id, mut image_id) = Default::default();
                self.gpu.create_resources(
                    [gpu::BufferCreateInfo::new(
                        &mut staging_id,
                        &staging_binder,
                        buf_size,
                        gpu::BufferUsages::TRANSFER_SRC
                    ).unwrap()],
                    [gpu::ImageCreateInfo
                        ::new(&mut image_id, &view_binder)
                        .with_dimensions((width, height))
                        .with_format(gpu::Format::R8g8b8a8Srgb, false)
                        .with_usage(
                            gpu::ImageUsages::TRANSFER_SRC |
                            gpu::ImageUsages::TRANSFER_DST |
                            gpu::ImageUsages::SAMPLED
                        ).with_mip_levels(mip_levels)
                    ])?;
                let mut map = self.gpu.map_buffer(staging_id)?;
                unsafe {
                    map.write_bytes(new_img.as_bytes());
                }
                if !map.is_coherent {
                    self.gpu.flush_mapped_memory_ranges(&[
                        gpu::MappedBufferMemoryRange {
                            buffer_id: staging_id,
                            offset: 0,
                            size: buf_size,
                        },
                    ])?;
                }
                let view_id = self.gpu.create_image_view(
                    image_id,
                    gpu::ImageRange::whole_range(gpu::ImageAspects::COLOR),
                )?;
                command_id = self.gpu.schedule_commands()
                    .new_commands::<gpu::NewCopyCommands>(
                        self.gpu.any_device_queue(gpu::QueueFlags::GRAPHICS).unwrap(),
                        move |cmd| {
                            cmd.copy_buffer_to_image(
                                staging_id,
                                image_id,
                                &[gpu::BufferImageCopy
                                    ::default()
                                    .image_subresource(gpu::ImageSubresourceLayers
                                        ::default()
                                        .aspect_mask(gpu::ImageAspects::COLOR)
                                    ).image_extent((width, height))
                                ],
                                gpu::CommandOrdering::Lenient,
                            )?;
                            cmd.gen_mip_map(image_id, gpu::Filter::Linear)?;
                            Ok(())
                        }
                    )?.with_signal_semaphore(self.semaphore, self.semaphore_value + 1).id();
                self.semaphore_value += 1;
                Result::Ok(LoadedImage {
                    last_modified,
                    last_updated: self.semaphore_value,
                    source: new_img,
                    staging_binder,
                    staging_id,
                    view_binder,
                    view_id,
                })
            }) {
                Ok(img) => {
                    let view_id = img.view_id;
                    self.images.insert(path.into(), img);
                    return (ImageSourceInternal {
                        view_id,
                    }, Some(gpu::CommandDependency::new(
                        command_id, gpu::MemoryDependencyHint::FRAGMENT_SHADER,
                    )))
                },
                Err(err) => {
                    expand_warn!(Error::new(
                        err, "(nox_gui) failed to load image to gpu memory",
                    ));
                },
            }
        }
        self.load_err_image()
    }
}

impl Drop for ImageLoader {

    fn drop(&mut self) {
        self.gpu.destroy_resources([self.err_staging_id], [self.err_view_id.image_id()]).ok();
        self.gpu.destroy_timeline_semaphores(&[self.semaphore]);
        for image in self.images.values() {
            self.gpu.destroy_resources([image.staging_id], [image.view_id.image_id()]).ok();
        }
    }
}

#[derive(Default)]
pub struct ImageData {
    offset: Vec2,
    size: Vec2,
    source: Option<ImageSourceInternal>,
    loc: Option<Location>,
}

impl Tracked for ImageData {

    fn location(&self) -> Option<Location> {
        self.loc
    }
}

impl ImageData {

    #[inline]
    pub fn update_source(
        &mut self,
        source: ImageSourceInternal,
        loc: Location,
        offset: Vec2,
        size: Vec2,
    )
    {
        self.source = Some(source);
        self.loc = Some(loc);
        self.offset = offset;
        self.size = size;
    } 

    pub fn draw(
        &mut self,
        cmd: &mut gpu::DrawCommands,
        rec: &mut RecordCmd<'_>,
        sampler: gpu::Sampler,
        cached_data: &CachedUiData,
        content_off: Vec2,
        content_area: BoundingRect,
    ) -> Result<()> {
        let Some(source) = self.source else {
            return Ok(())
        };
        let (viewport, scissor) = cached_data.viewport_and_scissor();
        let mut pipeline_cmd = cmd.bind_pipeline(
            rec.texture_pipeline(),
            &[viewport], &[scissor],
        ).context("failed to bind texture pipeline")?;
        pipeline_cmd.push_descriptor_bindings(&[
            gpu::PushDescriptorBinding::new(
                "tex",
                0,
                gpu::DescriptorInfos::images(&[
                    gpu::DescriptorImageInfo {
                        sampler: Some(sampler),
                        image_view: Some(source.view_id),
                    },
                ]),
                gpu::CommandBarrierInfo::new(
                    gpu::CommandOrdering::Lenient,
                    gpu::ExplicitAccess::SHADER_READ,
                ),
            )?,
        ]).context("failed to push descriptor binding")?;
        let pc_vertex = calc_texture_push_constants_vertex(
            content_off + self.offset,
            self.size,
            cached_data.inv_aspect_ratio,
            cached_data.unit_scale
        );
        let pc_fragment = base_push_constants_fragment(
            content_area.min, content_area.max
        );
        pipeline_cmd
            .push_constants(pc_vertex.0, &[pc_vertex.1])
            .context("failed to push constants")?;
        pipeline_cmd
            .push_constants(pc_fragment.0, &[pc_fragment.1])
            .context("failed to push constants")?;
        pipeline_cmd.begin_drawing(
            gpu::DrawInfo
                ::default()
                .vertex_count(6),
            &[],
            None,
            |cmd| {
                cmd.draw()?;
                Ok(())
            },
        ).context("failed to draw image")?;
        Ok(())
    }
}
