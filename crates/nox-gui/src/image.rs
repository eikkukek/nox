use std::{
    fs,
    rc::Rc,
};

use core::{
    ptr::NonNull,
    slice,
};

use rustc_hash::FxHashMap;

use ::image::EncodableLayout;

use nox::{
    mem::Allocator,
    *
};

use nox_geom::*;

use crate::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ImageSource<'a> {
    Path(&'a str),
    Id(ImageId),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ImageSourceOwned {
    Path(CompactString),
    Id(ImageId),
}

#[derive(Clone, Copy)]
pub enum ImageSourceUnsafe {
    Path(NonNull<u8>, usize),
    Id(ImageId),
}

impl ImageSourceUnsafe {

    pub unsafe fn as_image_source<'a>(&'a self) -> ImageSource<'a> {
        match self {
            &Self::Path(data, len) => unsafe {
                ImageSource::Path(str
                    ::from_utf8(slice::from_raw_parts(data.as_ptr(), len))
                    .unwrap_or_default()
                )
            },
            &Self::Id(id) => ImageSource::Id(id)
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

#[derive(Clone, Eq)]
pub enum ImageSourceInternal {
    Err,
    Path(Rc<::image::ImageBuffer<::image::Rgba<u8>, Vec<u8>>>),
    Id(ImageId),
}

impl PartialEq for ImageSourceInternal {

    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Err => matches!(other, Self::Err),
            Self::Path(this) =>
                match other {
                    Self::Err => false,
                    Self::Path(other) => Rc::ptr_eq(this, other),
                    Self::Id(_) => false,
                },
            Self::Id(this) => 
                match other {
                    Self::Err => false,
                    Self::Path(_) => false,
                    Self::Id(other) => this == other,
                }

        }
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


pub struct ImageLoader {
    images: FxHashMap<CompactString, (std::time::SystemTime, Rc<::image::ImageBuffer<::image::Rgba<u8>, Vec<u8>>>)>,
}

impl ImageLoader {

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            images: FxHashMap::default(),
        }
    }

    #[inline(always)]
    pub fn load_image(&mut self, path: &str) -> ImageSourceInternal {
        if let Some((last_modified, source)) = self.images.get_mut(path) {
            if let Ok(meta) = fs::metadata(path) {
                if let Ok(modified) = meta.modified() {
                    if modified == *last_modified {
                        return ImageSourceInternal::Path(source.clone())
                    }
                    if let Ok(new_img) = load_rgba_image(path) {
                        *source = Rc::new(new_img);
                        *last_modified = modified;
                    } else {
                        return ImageSourceInternal::Err
                    }
                }
            }
            return ImageSourceInternal::Err
        }
        if let Ok(meta) = fs::metadata(path) {
            if let Ok(modified) = meta.modified() {
                if let Ok(new_img) = load_rgba_image(path) {
                    return ImageSourceInternal::Path(
                        self.images
                            .entry(path.into())
                            .or_insert((modified, Rc::new(new_img)))
                            .1
                            .clone()
                    )
                }
            }
        }
        ImageSourceInternal::Err
    }
}

#[derive(Default)]
pub struct ImageData {
    offset: Vec2,
    size: Vec2,
    source: Option<ImageSourceInternal>,
    render_format: ColorFormat,
    image: Option<ImageId>,
    shader_resource: Option<ShaderResourceId>,
    flags: u32,
}

impl ImageData {

    const SOURCE_RESET: u32 = 0x1;

    #[inline(always)]
    pub fn requires_transfer_commands(&self) -> bool {
        self.flags & Self::SOURCE_RESET == Self::SOURCE_RESET
    }

    #[inline(always)]
    fn source_reset(&self) -> bool {
        self.flags & Self::SOURCE_RESET == Self::SOURCE_RESET
    }

    #[inline(always)]
    pub fn update_source(
        &mut self,
        source: ImageSourceInternal,
        offset: Vec2,
        size: Vec2,
    )
    {
        if let Some(cur) = &self.source {
            if cur != &source {
                self.flags |= Self::SOURCE_RESET;
                self.source = Some(source.clone());
            }
        } else {
            self.flags |= Self::SOURCE_RESET;
            self.source = Some(source.clone());
        }
        self.offset = offset;
        self.size = size;
    }

    pub fn render(
        &mut self,
        frame_graph: &mut dyn FrameGraph,
        render_format: ColorFormat,
        add_read: &mut dyn FnMut(ReadInfo),
    ) -> Result<(), Error> {
        self.render_format = render_format;
        if !self.source_reset() && let Some(image) = self.image {
            let resource_id = frame_graph.add_image(image)?;
            add_read(ReadInfo::new(resource_id, None));
        }
        Ok(())
    }

    pub fn transfer_commands(
        &mut self,
        transfer_commands: &mut TransferCommands,
        window_semaphore: (TimelineSemaphoreId, u64),
        sampler: SamplerId,
        texture_pipeline_layout: PipelineLayoutId,
        tmp_alloc: &impl Allocator,
    ) -> Result<(), Error> {
        if self.source_reset() {
            let source = self.source.as_ref().unwrap();
            transfer_commands.edit_resources(|mut cmd, r| {
                if r.wait_for_semaphores(
                        &[(window_semaphore.0, window_semaphore.1)],
                        u64::MAX,
                        tmp_alloc,
                    )?
                {
                    self.flags &= !Self::SOURCE_RESET;
                    if let Some(image) = self.image {
                        r.destroy_image(image);
                    }
                    let image = match source {
                        ImageSourceInternal::Err => {
                            let bytes =
                            [
                                255, 0, 203, 255,
                                0, 0, 0, 255,
                                0, 0, 0, 255,
                                255, 0, 203, 255,
                            ];
                            let &mut image = self.image.insert(r.create_image(ResourceBinderImage::DefaultBinder, |builder| {
                                builder
                                    .with_dimensions(Dimensions::new(2, 2, 1))
                                    .with_format(self.render_format, false)
                                    .with_usage(ImageUsage::Sampled)
                                    .with_usage(ImageUsage::TransferDst);
                            })?);
                            cmd.copy_data_to_image(
                                r, image, &bytes, None, None, None
                            )?;
                            image
                        }
                        ImageSourceInternal::Path(buf) => {
                            let (bytes, dim) = (buf.as_bytes(), buf.dimensions());
                            let &mut image = self.image.insert(r.create_image(ResourceBinderImage::DefaultBinder, |builder| {
                                builder
                                    .with_dimensions(Dimensions::new(dim.0, dim.1, 1))
                                    .with_format(self.render_format, false)
                                    .with_usage(ImageUsage::Sampled)
                                    .with_usage(ImageUsage::TransferDst)
                                    .with_usage(ImageUsage::TransferSrc)
                                    .with_mip_levels((dim.0.max(dim.1) as f32).log2().floor() as u32);
                            })?);
                            cmd.copy_data_to_image(
                                r, image, bytes, None, None, None
                            )?;
                            cmd.gen_mip_maps(r, image, Filter::Linear)?;
                            image
                        },
                        &ImageSourceInternal::Id(t) => {
                            *self.image.insert(t)
                        }
                    };
                    let resource =
                        if let Some(resource) = self.shader_resource {
                            resource
                        } else {
                            r.allocate_shader_resources(
                                &[
                                    ShaderResourceInfo::new(texture_pipeline_layout, 0)
                                ],
                                |_, id| { self.shader_resource = Some(id); },
                                tmp_alloc,
                            )?;
                            self.shader_resource.unwrap()
                        };
                    r.update_shader_resources(
                        &[
                            ShaderResourceImageUpdate {
                                resource,
                                binding: 0,
                                starting_index: 0,
                                infos: &[
                                    ShaderResourceImageInfo {
                                        sampler,
                                        image_source: (image, None),
                                        storage_image: false,
                                    }
                                ]
                            }
                        ], &[], &[], tmp_alloc
                    )?;
                }
                Ok(())
            })?;
        }
        Ok(())
    }

    pub fn render_commands(
        &mut self,
        render_commands: &mut RenderCommands,
        sampler: SamplerId,
        texture_pipeline: GraphicsPipelineId,
        texture_pipeline_layout: PipelineLayoutId,
        content_off: Vec2,
        content_area: BoundingRect,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        tmp_alloc: &impl Allocator,
    ) -> Result<(), Error> {
        if self.shader_resource.is_none() {
            render_commands.edit_resources(|r| {
                r.allocate_shader_resources(
                    &[
                        ShaderResourceInfo::new(texture_pipeline_layout, 0)
                    ],
                    |_, id| { self.shader_resource = Some(id); },
                    tmp_alloc
                )?;
                r.update_shader_resources(
                    &[
                        ShaderResourceImageUpdate {
                            resource: self.shader_resource.unwrap(),
                            binding: 0,
                            starting_index: 0,
                            infos: &[
                                ShaderResourceImageInfo {
                                    sampler,
                                    image_source: (self.image.unwrap(), None),
                                    storage_image: false,
                                }
                        ]
                        }
                    ], &[], &[], tmp_alloc
                )?;
                Ok(())
            })?;
        }
        render_commands.bind_pipeline(texture_pipeline)?;
        render_commands.bind_shader_resources(|_| {
            self.shader_resource.unwrap()
        })?;
        let pc_vertex = calc_texture_push_constants_vertex(
            content_off + self.offset,
            self.size,
            inv_aspect_ratio,
            unit_scale
        );
        let pc_fragment = base_push_constants_fragment(
            content_area.min, content_area.max
        );
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_commands.draw_bufferless(6, 1);
        Ok(())
    }

    pub fn hide(
        &mut self,
        window_semaphore: (TimelineSemaphoreId, u64),
        global_resources: &mut GlobalResources,
        tmp_alloc: &impl Allocator,
    ) -> Result<(), Error> {
        if let Some(resource) = self.shader_resource.take() {
            if global_resources.wait_for_semaphores(
                &[(window_semaphore.0, window_semaphore.1)],
                u64::MAX,
                tmp_alloc,
            )? {
                global_resources.free_shader_resources(&[resource], &GlobalAlloc)?;
            }
        }
        Ok(())
    }
}
