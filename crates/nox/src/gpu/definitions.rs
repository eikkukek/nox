use nox_ash::vk;

use nox_error::Display;

use nox_proc::FeatureStruct;

use super::{
    ext::MissingDeviceFeatureError,
    *
};

use core::num::NonZeroU32;

pub type DeviceSize = u64;

/// Sets which base device features to enable.
///
/// By default `sample_rate_shading`, `sampler_anisotropy` and `alpha_to_one` are enabled.
///
/// You can find the descriptions for each feature here:
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceFeatures.html>
#[derive(Clone, Copy, FeatureStruct)]
pub struct BaseDeviceFeatures {
    pub robust_buffer_access: bool,
    pub full_draw_index_uint32: bool,
    pub image_cube_array: bool,
    pub independent_blend: bool,
    pub geometry_shader: bool,
    pub tessellation_shader: bool,
    #[on]
    pub sample_rate_shading: bool,
    pub dual_src_blend: bool,
    pub logic_op: bool,
    pub multi_draw_indirect: bool,
    pub draw_indirect_first_instance: bool,
    pub depth_clamp: bool,
    pub depth_bias_clamp: bool,
    pub fill_mode_non_solid: bool,
    pub depth_bounds: bool,
    pub wide_lines: bool,
    pub large_points: bool,
    #[on]
    pub alpha_to_one: bool,
    pub multi_viewport: bool,
    #[on]
    pub sampler_anisotropy: bool,
    pub texture_compression_etc2: bool,
    pub texture_compression_astc_ldr: bool,
    pub texture_compression_bc: bool,
    pub occlusion_query_precise: bool,
    pub pipeline_statistics_query: bool,
    pub vertex_pipeline_stores_and_atomics: bool,
    pub fragment_stores_and_atomics: bool,
    pub shader_tessellation_and_geometry_point_size: bool,
    pub shader_image_gather_extended: bool,
    pub shader_storage_image_extended_formats: bool,
    pub shader_storage_image_multisample: bool,
    pub shader_storage_image_read_without_format: bool,
    pub shader_storage_image_write_without_format: bool,
    pub shader_uniform_buffer_array_dynamic_indexing: bool,
    pub shader_sampled_image_array_dynamic_indexing: bool,
    pub shader_storage_buffer_array_dynamic_indexing: bool,
    pub shader_storage_image_array_dynamic_indexing: bool,
    pub shader_clip_distance: bool,
    pub shader_cull_distance: bool,
    pub shader_float64: bool,
    pub shader_int64: bool,
    pub shader_int16: bool,
    pub shader_resource_residency: bool,
    pub shader_resource_min_lod: bool,
    pub sparse_binding: bool,
    pub sparse_residency_buffer: bool,
    pub sparse_residency_image_2d: bool,
    pub sparse_residency_image_3d: bool,
    pub sparse_residency_2_samples: bool,
    pub sparse_residency_4_samples: bool,
    pub sparse_residency_8_samples: bool,
    pub sparse_residency_16_samples: bool,
    pub sparse_residency_aliased: bool,
    pub variable_multisample_rate: bool,
    pub inherited_queries: bool,
}

impl BaseDeviceFeatures {

    pub(crate) fn missing_features(self, available: &vk::PhysicalDeviceFeatures) -> Option<MissingDeviceFeatureError> {
        macro_rules! check {
            ($($field:ident),+ $(,)?) => {
                $(
                    if self.$field && (available.$field == 0) {
                        return Some(MissingDeviceFeatureError::new(stringify!($field)))
                    }
                )+
            };
        }
        check!(
            robust_buffer_access,
            full_draw_index_uint32,
            image_cube_array,
            independent_blend,
            geometry_shader,
            tessellation_shader,
            sample_rate_shading,
            dual_src_blend,
            logic_op,
            multi_draw_indirect,
            draw_indirect_first_instance,
            depth_clamp,
            depth_bias_clamp,
            fill_mode_non_solid,
            depth_bounds,
            wide_lines,
            large_points,
            alpha_to_one,
            multi_viewport,
            sampler_anisotropy,
            texture_compression_etc2,
            texture_compression_astc_ldr,
            texture_compression_bc,
            occlusion_query_precise,
            pipeline_statistics_query,
            vertex_pipeline_stores_and_atomics,
            fragment_stores_and_atomics,
            shader_tessellation_and_geometry_point_size,
            shader_image_gather_extended,
            shader_storage_image_extended_formats,
            shader_storage_image_multisample,
            shader_storage_image_read_without_format,
            shader_storage_image_write_without_format,
            shader_uniform_buffer_array_dynamic_indexing,
            shader_sampled_image_array_dynamic_indexing,
            shader_storage_buffer_array_dynamic_indexing,
            shader_storage_image_array_dynamic_indexing,
            shader_clip_distance,
            shader_cull_distance,
            shader_float64,
            shader_int64,
            shader_int16,
            shader_resource_residency,
            shader_resource_min_lod,
            sparse_binding,
            sparse_residency_buffer,
            sparse_residency_aliased,
            variable_multisample_rate,
            inherited_queries,
        );
        if self.sparse_residency_image_2d && (available.sparse_residency_image2_d == 0) {
            return Some(MissingDeviceFeatureError::new("sparse_residency_image_2d"))
        }
        if self.sparse_residency_image_3d && (available.sparse_residency_image2_d == 0) {
            return Some(MissingDeviceFeatureError::new("sparse_residency_image_3d"))
        }
        if self.sparse_residency_2_samples && (available.sparse_residency2_samples == 0) {
            return Some(MissingDeviceFeatureError::new("sparse_residency_2_samples"))
        }
        if self.sparse_residency_4_samples && (available.sparse_residency4_samples == 0) {
            return Some(MissingDeviceFeatureError::new("sparse_residency_4_samples"))
        }
        if self.sparse_residency_8_samples && (available.sparse_residency8_samples == 0) {
            return Some(MissingDeviceFeatureError::new("sparse_residency_8_samples"))
        }
        if self.sparse_residency_16_samples && (available.sparse_residency16_samples == 0) {
            return Some(MissingDeviceFeatureError::new("sparse_residency_16_samples"))
        }
        None
    }
}

impl From<BaseDeviceFeatures> for vk::PhysicalDeviceFeatures {

    fn from(value: BaseDeviceFeatures) -> Self {
        Self {
            robust_buffer_access: value.robust_buffer_access as u32,
            full_draw_index_uint32: value.full_draw_index_uint32 as u32,
            image_cube_array: value.image_cube_array as u32,
            independent_blend: value.independent_blend as u32,
            geometry_shader: value.geometry_shader as u32,
            tessellation_shader: value.tessellation_shader as u32,
            sample_rate_shading: value.sample_rate_shading as u32,
            dual_src_blend: value.dual_src_blend as u32,
            logic_op: value.logic_op as u32,
            multi_draw_indirect: value.multi_draw_indirect as u32,
            draw_indirect_first_instance: value.draw_indirect_first_instance as u32,
            depth_clamp: value.depth_clamp as u32,
            depth_bias_clamp: value.depth_bias_clamp as u32,
            fill_mode_non_solid: value.fill_mode_non_solid as u32,
            depth_bounds: value.depth_bounds as u32,
            wide_lines: value.wide_lines as u32,
            large_points: value.large_points as u32,
            alpha_to_one: value.alpha_to_one as u32,
            multi_viewport: value.multi_viewport as u32,
            sampler_anisotropy: value.sampler_anisotropy as u32,
            texture_compression_etc2: value.texture_compression_etc2 as u32,
            texture_compression_astc_ldr: value.texture_compression_astc_ldr as u32,
            texture_compression_bc: value.texture_compression_bc as u32,
            occlusion_query_precise: value.occlusion_query_precise as u32,
            pipeline_statistics_query: value.pipeline_statistics_query as u32,
            vertex_pipeline_stores_and_atomics: value.vertex_pipeline_stores_and_atomics as u32,
            fragment_stores_and_atomics: value.fragment_stores_and_atomics as u32,
            shader_tessellation_and_geometry_point_size: value.shader_tessellation_and_geometry_point_size as u32,
            shader_image_gather_extended: value.shader_image_gather_extended as u32,
            shader_storage_image_extended_formats: value.shader_storage_image_extended_formats as u32,
            shader_storage_image_multisample: value.shader_storage_image_multisample as u32,
            shader_storage_image_read_without_format: value.shader_storage_image_read_without_format as u32,
            shader_storage_image_write_without_format: value.shader_storage_image_write_without_format as u32,
            shader_uniform_buffer_array_dynamic_indexing: value.shader_uniform_buffer_array_dynamic_indexing as u32,
            shader_sampled_image_array_dynamic_indexing: value.shader_sampled_image_array_dynamic_indexing as u32,
            shader_storage_buffer_array_dynamic_indexing: value.shader_storage_buffer_array_dynamic_indexing as u32,
            shader_storage_image_array_dynamic_indexing: value.shader_storage_image_array_dynamic_indexing as u32,
            shader_clip_distance: value.shader_clip_distance as u32,
            shader_cull_distance: value.shader_cull_distance as u32,
            shader_float64: value.shader_float64 as u32,
            shader_int64: value.shader_int64 as u32,
            shader_int16: value.shader_int16 as u32,
            sparse_binding: value.sparse_binding as u32,
            shader_resource_residency: value.shader_resource_residency as u32,
            shader_resource_min_lod: value.shader_resource_min_lod as u32,
            sparse_residency_buffer: value.sparse_residency_buffer as u32,
            sparse_residency_image2_d: value.sparse_residency_image_2d as u32,
            sparse_residency_image3_d: value.sparse_residency_image_3d as u32,
            sparse_residency2_samples: value.sparse_residency_2_samples as u32,
            sparse_residency4_samples: value.sparse_residency_4_samples as u32,
            sparse_residency8_samples: value.sparse_residency_8_samples as u32,
            sparse_residency16_samples: value.sparse_residency_16_samples as u32,
            sparse_residency_aliased: value.sparse_residency_aliased as u32,
            variable_multisample_rate: value.variable_multisample_rate as u32,
            inherited_queries: value.inherited_queries as u32,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
#[display("(x: {x}, y: {y})")]
pub struct Offset2D {
    pub x: i32,
    pub y: i32
}

impl From<Offset2D> for vk::Offset2D {

    fn from(value: Offset2D) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
#[display("(x: {x}, y: {y}, z: {z})")]
pub struct Offset3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Offset3D {

    pub fn new(x: i32, y: i32, z: i32) -> Self
    {
        Self {x, y, z}
    }
}

impl From<Offset3D> for vk::Offset3D {

    fn from(value: Offset3D) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

/// Used for image dimensions.
///
/// Depth is included for 3D images.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
#[display("({width}, {height}, {depth})")]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Dimensions {

    #[inline(always)]
    pub fn new(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }

    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.width == 0 ||
        self.height == 0 ||
        self.depth == 0
    }

    #[inline(always)]
    pub fn texel_count(&self) -> vk::DeviceSize {
        self.width as vk::DeviceSize *
        self.height as vk::DeviceSize *
        self.depth as vk::DeviceSize
    }
}

impl From<Dimensions> for vk::Extent3D {

    fn from(value: Dimensions) -> Self {
        vk::Extent3D {
            width: value.width,
            height: value.height,
            depth: value.depth,
        }
    }
}

impl From<vk::Extent2D> for Dimensions {

    fn from(value: vk::Extent2D) -> Self {
        Dimensions::new(
            value.width,
            value.height,
            1,
        )
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentMapping {
    pub r: ComponentSwizzle,
    pub g: ComponentSwizzle,
    pub b: ComponentSwizzle,
    pub a: ComponentSwizzle,
}

impl From<ComponentMapping> for vk::ComponentMapping {

    fn from(value: ComponentMapping) -> Self {
        Self {
            r: value.r.into(),
            g: value.g.into(),
            b: value.b.into(),
            a: value.a.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSubresourceRange {
    pub aspect_mask: ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: NonZeroU32,
    pub base_array_layer: u32,
    pub layer_count: NonZeroU32,
}

impl ImageSubresourceRange {

    pub fn new(
        aspect_mask: ImageAspectFlags,
        base_mip_level: u32,
        level_count: u32,
        base_array_layer: u32,
        layer_count: u32,
    ) -> Option<Self>
    {
        Some(Self {
            aspect_mask,
            base_mip_level,
            level_count: NonZeroU32::new(level_count)?,
            base_array_layer,
            layer_count: NonZeroU32::new(layer_count)?,
        })
    }

    pub fn overlaps(self, other: Self) -> bool {
        if self.aspect_mask.intersects(other.aspect_mask) {
            self.base_mip_level < other.base_mip_level + other.level_count.get() &&
            other.base_mip_level < self.base_mip_level + self.level_count.get() &&
            self.base_array_layer < other.base_array_layer + other.layer_count.get() &&
            other.base_array_layer < self.base_array_layer + self.layer_count.get()
        } else {
            false
        }
    }
}

impl Default for ImageSubresourceRange {

    fn default() -> Self {
        Self {
            aspect_mask: ImageAspectFlags::empty(),
            base_mip_level: 0,
            level_count: NonZeroU32::new(1).unwrap(),
            base_array_layer: 0,
            layer_count: NonZeroU32::new(1).unwrap(),
        }
    }
}

impl From<ImageSubresourceRange> for vk::ImageSubresourceRange {

    fn from(value: ImageSubresourceRange) -> Self {
        Self {
            aspect_mask: vk::ImageAspectFlags::from_raw(value.aspect_mask.as_raw()),
            base_mip_level: value.base_mip_level,
            level_count: value.level_count.get(),
            base_array_layer: value.base_array_layer,
            layer_count: value.layer_count.get(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSubresourceLayers {
    pub aspect_mask: ImageAspectFlags,
    pub mip_level: u32,
    pub base_array_layer: u32,
    pub layer_count: NonZeroU32,
}

impl ImageSubresourceLayers {

    pub fn new(
        aspect_mask: ImageAspectFlags,
        mip_level: u32,
        base_array_layer: u32,
        layer_count: u32,
    ) -> Option<Self> {
        Some(Self {
            aspect_mask: aspect_mask,
            mip_level,
            base_array_layer,
            layer_count: NonZeroU32::new(layer_count)?,
        })
    }
}

impl From<ImageSubresourceLayers> for vk::ImageSubresourceLayers {

    fn from(value: ImageSubresourceLayers) -> Self {
        Self {
            aspect_mask: value.aspect_mask.into(),
            mip_level: value.mip_level,
            base_array_layer: value.base_array_layer,
            layer_count: value.layer_count.get(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentInfo {
    pub component_mapping: ComponentMapping,
    pub(crate) format: vk::Format,
}

impl ComponentInfo {

    pub fn new<F: Format>(
        component_mapping: ComponentMapping,
        format: F,
    ) -> Self
    {
        Self {
            component_mapping,
            format: format.as_vk_format(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ImageRange {
    pub subresource_range: ImageSubresourceRange,
    pub component_info: Option<ComponentInfo>,
    pub cube_map: bool,
}

impl ImageRange {

    #[inline(always)]
    pub fn new(
        subresource_range: ImageSubresourceRange,
        component_info: Option<ComponentInfo>,
    ) -> Self
    {
        Self {
            subresource_range,
            component_info,
            cube_map: false,
        }
    }

    pub fn with_cube_map(mut self, is: bool) -> Self {
        self.cube_map = is;
        self
    }
}

/// Specifies a viewport.
/// 
/// # Vulkan docs
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkViewport.html>**
pub type Viewport = vk::Viewport;

/// Specifies a scissor.
///
/// This is used instead of `VkRect2D`, to enforce that x >= 0 and y >= 0.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Scissor {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Controls the robustness of pipelines.
///
/// # Vulkan docs
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRobustnessCreateInfo.html>**
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct PipelineRobustnessInfo {
    pub storage_buffer_behavior: PipelineRobustnessBufferBehavior,
    pub uniform_buffer_behavior: PipelineRobustnessBufferBehavior,
    pub vertex_input_behavior: PipelineRobustnessBufferBehavior,
    pub image_behavior: PipelineRobustnessImageBehavior,
}

impl From<PipelineRobustnessInfo> for vk::PipelineRobustnessCreateInfo<'_> {

    #[inline(always)]
    fn from(value: PipelineRobustnessInfo) -> Self {
        Self {
            storage_buffers: value.storage_buffer_behavior.into(),
            uniform_buffers: value.uniform_buffer_behavior.into(),
            vertex_input: value.vertex_input_behavior.into(),
            images: value.image_behavior.into(),
            ..Default::default()
        }
    }
}
