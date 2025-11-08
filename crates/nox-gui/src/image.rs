use std::rc::Rc;

use core::marker::PhantomData;

use ::image::EncodableLayout;

use nox::{
    alloc::arena_alloc::ArenaGuard,
    mem::vec_types::GlobalVec,
    *
};

use nox_geom::*;

use crate::*;

#[derive(Clone, PartialEq, Eq)]
pub enum ImageSource {
    Path(std::path::PathBuf),
    Texture(ImageId),
}

#[derive(Clone, Eq)]
pub enum ImageSourceInternal {
    Err,
    Path(Rc< ::image::ImageBuffer<::image::Rgba<u8>, Vec<u8>>>),
    Texture(ImageId),
}

impl PartialEq for ImageSourceInternal {

    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Err => matches!(other, Self::Err),
            Self::Path(this) =>
                match other {
                    Self::Err => false,
                    Self::Path(other) => Rc::ptr_eq(this, other),
                    Self::Texture(_) => false,
                },
            Self::Texture(this) => 
                match other {
                    Self::Err => false,
                    Self::Path(_) => false,
                    Self::Texture(other) => this == other,
                }

        }
    }
}

#[macro_export]
macro_rules! image_source {
    ($path:tt) => {
        ImageSource::Path(<std::path::PathBuf as std::str::FromStr>::from_str($path).unwrap_or_default())
    };
    ($texture:expr) => {
        ImageSource::Texture($texture)
    };
}

pub struct Image<I, Style> {
    offset: Vec2,
    size: Vec2,
    source: Option<ImageSourceInternal>,
    render_format: ColorFormat,
    image: Option<ImageId>,
    shader_resource: core::cell::RefCell<Option<ShaderResourceId>>,
    flags: u32,
    _marker: PhantomData<(I, Style)>
}

impl<I, Style> Image<I, Style> {

    const SOURCE_RESET: u32 = 0x1;

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            offset: Default::default(),
            size: Default::default(),
            source: None,
            render_format: Default::default(),
            image: None,
            shader_resource: core::cell::RefCell::new(None),
            flags: 0,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn update_source(
        &mut self,
        source: &ImageSourceInternal,
        size: Vec2,
    )
    {
        if let Some(cur) = &self.source {
            if cur != source {
                self.flags |= Self::SOURCE_RESET;
                self.source = Some(source.clone());
            }
        } else {
            self.flags |= Self::SOURCE_RESET;
            self.source = Some(source.clone());
        }
        self.size = size;
    }

    #[inline(always)]
    fn source_reset(&self) -> bool {
        self.flags & Self::SOURCE_RESET == Self::SOURCE_RESET
    }
}

impl<I, Style> Widget<I, Style> for Image<I, Style>
    where 
        I: Interface,
        Style: WindowStyle,
{

    fn get_offset(&self) -> Vec2 {
        self.offset
    }

    fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
    }

    fn set_scroll_offset(&mut self, offset: Vec2) {
        self.offset += offset;
    }

    fn calc_size(
        &mut self,
        _style: &Style,
        _text_renderer: &mut TextRenderer,
    ) -> Vec2 {
        self.size
    }

    fn status<'a>(
        &'a self,
        _nox: &Nox<I>,
        _style: &Style,
        _window_pos: Vec2,
        _cursor_pos: Vec2,
    ) -> WidgetStatus<'a> {
        WidgetStatus::Inactive
    }

    fn update(
        &mut self,
        _nox: &mut Nox<I>,
        _style: &Style,
        _text_renderer: &mut TextRenderer,
        _window_size: Vec2,
        _window_pos: Vec2,
        _content_offset: Vec2,
        _cursor_pos: Vec2,
        _delta_cursor_pos: Vec2,
        _cursor_in_this_window: bool,
        _other_widget_active: bool,
        _cursor_in_other_widget: bool,
        _window_moving: bool,
        _hover_blocked: bool,
        _collect_text: &mut dyn FnMut(&nox_font::RenderedText, Vec2, BoundedTextInstance),
    ) -> UpdateResult {
        UpdateResult {
            requires_triangulation: false,
            requires_transfer_commands: self.source_reset(),
            cursor_in_widget: false,
        }
    }

    fn render(
        &mut self,
        frame_graph: &mut dyn FrameGraph,
        _msaa_samples: MSAA, 
        render_format: ColorFormat,
        _resolve_mode: Option<ResolveMode>,
        add_read: &mut dyn FnMut(ReadInfo),
        _add_signal_semaphore: &mut dyn FnMut(TimelineSemaphoreId, u64),
        _collect_pass: &mut dyn FnMut(PassId),
    ) -> Result<(), Error> {
        self.render_format = render_format;
        if !self.source_reset() && let Some(image) = self.image {
            let resource_id = frame_graph.add_image(image)?;
            add_read(ReadInfo::new(resource_id, None));
        }
        Ok(())
    }

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        _style: &Style,
        sampler: SamplerId,
        _base_pipeline: GraphicsPipelineId,
        _text_pipeline: GraphicsPipelineId,
        texture_pipeline: GraphicsPipelineId,
        texture_pipeline_layout: PipelineLayoutId,
        _vertex_buffer: &mut RingBuf,
        _index_buffer: &mut RingBuf,
        window_pos: Vec2,
        content_area: BoundingRect,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        tmp_alloc: &ArenaGuard,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, Style>>, Error> {
        let mut shader_resource = self.shader_resource.borrow_mut();
        if shader_resource.is_none() {
            render_commands.edit_resources(|r| {
                r.allocate_shader_resources(
                    &[
                        ShaderResourceInfo::new(texture_pipeline_layout, 0)
                    ],
                    |_, id| { *shader_resource = Some(id); },
                    tmp_alloc
                )?;
                r.update_shader_resources(
                    &[
                        ShaderResourceImageUpdate {
                            resource: shader_resource.unwrap(),
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
            shader_resource.unwrap()
        })?;
        let pc_vertex = calc_texture_push_constants_vertex(
            window_pos + self.offset,
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
        Ok(None)
    }

    fn transfer_commands(
        &mut self,
        transfer_commands: &mut TransferCommands,
        window_semaphore: Option<(TimelineSemaphoreId, u64)>,
        sampler: SamplerId,
        texture_pipeline_layout: PipelineLayoutId,
        tmp_alloc: &ArenaGuard,
    ) -> Result<(), Error> {
        if self.source_reset() {
            let source = self.source.as_ref().unwrap();
            transfer_commands.edit_resources(|mut cmd, r| {
                if let Some((semaphore, value)) = window_semaphore {
                    if r.wait_for_semaphores(
                            &[(semaphore, value)],
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
                            &ImageSourceInternal::Texture(t) => {
                                *self.image.insert(t)
                            }
                        };
                        let mut shader_resource = self.shader_resource.borrow_mut();
                        let resource =
                            if let Some(resource) = *shader_resource {
                                resource
                            } else {
                                r.allocate_shader_resources(
                                    &[
                                        ShaderResourceInfo::new(texture_pipeline_layout, 0)
                                    ],
                                    |_, id| { *shader_resource = Some(id); },
                                    tmp_alloc,
                                )?;
                                shader_resource.unwrap()
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
                }
                Ok(())
            })?;
        }
        Ok(())
    }

    fn triangulate(
        &mut self,
        _points: &mut GlobalVec<[f32; 2]>,
        _helper_points: &mut GlobalVec<[f32; 2]>,
        _tri: &mut dyn FnMut(&[[f32; 2]]) -> Option<VertexRange>,
    ) {}

    fn set_vertex_params(
        &mut self,
        _style: &Style,
        _vertices: &mut [Vertex],
    ) {}

    fn hide(
        &mut self,
        _vertices: &mut [Vertex],
        window_semaphore: (TimelineSemaphoreId, u64),
        global_resources: &mut GlobalResources,
        tmp_alloc: &ArenaGuard,
    ) -> Result<(), Error> {
        if let Some(resource) = self.shader_resource.take() {
            global_resources.wait_for_semaphores_and_then(
                &[(window_semaphore.0, window_semaphore.1)],
                u64::MAX,
                tmp_alloc,
                |r| {
                    r.free_shader_resources(&[resource], &GlobalAlloc)?;
                    Ok(())
                }
            )?;
        }
        Ok(())
    }
}
