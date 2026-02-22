use super::*;

use core::{
    ffi::{c_void, CStr},
    marker::PhantomData,
    ptr,
    fmt,
};

/// *Extracted from [`ash`] source code.*
///
/// Iterates through the pointer chain. Includes the item that is passed into the function.
/// Stops at the last [`BaseOutStructure`] that has a null [`BaseOutStructure::p_next`] field.
pub unsafe fn ptr_chain_iter<T: ?Sized>(
    ptr: &mut T,
) -> impl Iterator<Item = *mut BaseOutStructure<'_>> {
    unsafe {
        let ptr = <*mut T>::cast::<BaseOutStructure<'_>>(ptr);
        (0..).scan(ptr, |p_ptr, _| {
            if p_ptr.is_null() {
                return None;
            }
            let n_ptr = (**p_ptr).p_next;
            let old = *p_ptr;
            *p_ptr = n_ptr;
            Some(old)
        })
    }
}

macro_rules! extend_structure_type {
    (
        $name:ident[$(#[doc = $doc:literal] $s_type:ident = $lit:literal),* $(,)?]
    ) => {
        pub trait $name {
            $(
                #[doc = $doc]
                const $s_type: Self;
            )*
        }
        
        impl $name for StructureType {
            $(const $s_type: Self = Self::from_raw($lit);)*
        }
    };
}

pub const API_VERSION_1_4: u32 = make_api_version(0, 1, 4, 0);

pub trait ObjectTypeKHR {
    const PIPELINE_BINARY_KHR: Self;
}

impl ObjectTypeKHR for ObjectType {

    const PIPELINE_BINARY_KHR: Self = Self::from_raw(1000483000);
}

pub trait ResultKHR {

    /// Provided by VK_KHR_pipeline_binary
    const PIPELINE_BINARY_MISSING_KHR: Self;
    /// Provided by VK_KHR_pipeline_binary
    const ERROR_NOT_ENOUGH_SPACE_KHR: Self;
}

impl ResultKHR for Result {

    const PIPELINE_BINARY_MISSING_KHR: Self = Self::from_raw(1000483000);
    const ERROR_NOT_ENOUGH_SPACE_KHR: Self = Self::from_raw(-1000483000);
}

extend_structure_type! { StructureTypeKHR [
    /// Provided by VK_KHR_present_id2
    SURFACE_CAPABILITIES_PRESENT_ID_2_KHR = 1000479000,
    /// Provided by VK_KHR_present_id2
    PRESENT_ID_2_KHR = 1000479001,
    /// Provided by VK_KHR_present_id2
    PHYSICAL_DEVICE_PRESENT_ID_2_FEATURES_KHR = 1000479002,
    /// Provided by VK_KHR_present_wait2
    SURFACE_CAPABILITIES_PRESENT_WAIT_2_KHR = 1000480000,
    /// Provided by VK_KHR_present_wait2
    PHYSICAL_DEVICE_PRESENT_WAIT_2_FEATURES_KHR = 1000480001,
    /// Provided by VK_KHR_present_wait2
    PRESENT_WAIT_INFO_2_KHR = 1000480002,
    /// Provided by VK_KHR_pipeline_binary
    PHYSICAL_DEVICE_PIPELINE_BINARY_FEATURES_KHR = 1000483000,
    /// Provided by VK_KHR_pipeline_binary
    PIPELINE_BINARY_CREATE_INFO_KHR = 1000483001,
    /// Provided by VK_KHR_pipeline_binary
    PIPELINE_BINARY_INFO_KHR = 1000483002,
    /// Provided by VK_KHR_pipeline_binary
    PIPELINE_BINARY_KEY_KHR = 1000483003,
    /// Provided by VK_KHR_pipeline_binary
    PHYSICAL_DEVICE_PIPELINE_BINARY_PROPERTIES_KHR = 1000483004,
    /// Provided by VK_KHR_pipeline_binary
    RELEASE_CAPTURED_PIPELINE_DATA_INFO_KHR = 1000483005,
    /// Provided by VK_KHR_pipeline_binary
    PIPELINE_BINARY_DATA_INFO_KHR = 1000483006,
    /// Provided by VK_KHR_pipeline_binary
    PIPELINE_CREATE_INFO_KHR = 1000483007,
    /// Provided by VK_KHR_pipeline_binary
    DEVICE_PIPELINE_BINARY_INTERNAL_CACHE_CONTROL_KHR = 1000483008,
    /// Provided by VK_KHR_pipeline_binary
    PIPELINE_BINARY_HANDLES_INFO_KHR = 1000483009,
]}

extend_structure_type!{ StructureType14 [
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_VULKAN_1_4_FEATURES = 55,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_VULKAN_1_4_PROPERTIES = 56,
    /// Provided by VK_VERSION_1_4
    DEVICE_QUEUE_GLOBAL_PRIORITY_CREATE_INFO = 1000174000,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_GLOBAL_PRIORITY_QUERY_FEATURES = 1000388000,
    /// Provided by VK_VERSION_1_4
    QUEUE_FAMILY_GLOBAL_PRIORITY_PROPERTIES = 1000388001,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_INDEX_TYPE_UINT8_FEATURES = 1000265000,
    /// Provided by VK_VERSION_1_4
    MEMORY_MAP_INFO = 1000271000,
    /// Provided by VK_VERSION_1_4
    MEMORY_UNMAP_INFO = 1000271001,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_MAINTENANCE_5_FEATURES = 1000470000,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_MAINTENANCE_5_PROPERTIES = 1000470001,
    /// Provided by VK_VERSION_1_4
    DEVICE_IMAGE_SUBRESOURCE_INFO = 1000470004,
    /// Provided by VK_VERSION_1_4
    SUBRESOURCE_LAYOUT_2 = 1000338002,
    /// Provided by VK_VERSION_1_4
    IMAGE_SUBRESOURCE_2 = 1000338003,
    /// Provided by VK_VERSION_1_4
    BUFFER_USAGE_FLAGS_2_CREATE_INFO = 1000470006,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_MAINTENANCE_6_FEATURES = 1000545000,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_MAINTENANCE_6_PROPERTIES = 1000545001,
    /// Provided by VK_VERSION_1_4
    BIND_MEMORY_STATUS = 1000545002,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_HOST_IMAGE_COPY_FEATURES = 1000270000,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_HOST_IMAGE_COPY_PROPERTIES = 1000270001,
    /// Provided by VK_VERSION_1_4
    MEMORY_TO_IMAGE_COPY = 1000270002,
    /// Provided by VK_VERSION_1_4
    IMAGE_TO_MEMORY_COPY = 1000270003,
    /// Provided by VK_VERSION_1_4
    COPY_IMAGE_TO_MEMORY_INFO = 1000270004,
    /// Provided by VK_VERSION_1_4
    COPY_MEMORY_TO_IMAGE_INFO = 1000270005,
    /// Provided by VK_VERSION_1_4
    HOST_IMAGE_LAYOUT_TRANSITION_INFO = 1000270006,
    /// Provided by VK_VERSION_1_4
    COPY_IMAGE_TO_IMAGE_INFO = 1000270007,
    /// Provided by VK_VERSION_1_4
    SUBRESOURCE_HOST_MEMCPY_SIZE = 1000270008,
    /// Provided by VK_VERSION_1_4
    HOST_IMAGE_COPY_DEVICE_PERFORMANCE_QUERY = 1000270009,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_SHADER_SUBGROUP_ROTATE_FEATURES = 1000416000,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_SHADER_FLOAT_CONTROLS_2_FEATURES = 1000528000,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_SHADER_EXPECT_ASSUME_FEATURES = 1000544000,
    /// Provided by VK_VERSION_1_4
    PIPELINE_CREATE_FLAGS_2_CREATE_INFO = 1000470005,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_PUSH_DESCRIPTOR_PROPERTIES = 1000080000,
    /// Provided by VK_VERSION_1_4
    BIND_DESCRIPTOR_SETS_INFO = 1000545003,
    /// Provided by VK_VERSION_1_4
    PUSH_CONSTANTS_INFO = 1000545004,
    /// Provided by VK_VERSION_1_4
    PUSH_DESCRIPTOR_SET_INFO = 1000545005,
    /// Provided by VK_VERSION_1_4
    PUSH_DESCRIPTOR_SET_WITH_TEMPLATE_INFO = 1000545006,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_PIPELINE_PROTECTED_ACCESS_FEATURES = 1000466000,
    /// Provided by VK_VERSION_1_4
    PIPELINE_ROBUSTNESS_CREATE_INFO = 1000068000,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_PIPELINE_ROBUSTNESS_FEATURES = 1000068001,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_PIPELINE_ROBUSTNESS_PROPERTIES = 1000068002,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_LINE_RASTERIZATION_FEATURES = 1000259000,
    /// Provided by VK_VERSION_1_4
    PIPELINE_RASTERIZATION_LINE_STATE_CREATE_INFO = 1000259001,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_LINE_RASTERIZATION_PROPERTIES = 1000259002,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_PROPERTIES = 1000525000,
    /// Provided by VK_VERSION_1_4
    PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO = 1000190001,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES = 1000190002,
    /// Provided by VK_VERSION_1_4
    RENDERING_AREA_INFO = 1000470003,
    /// Provided by VK_VERSION_1_4
    PHYSICAL_DEVICE_DYNAMIC_RENDERING_LOCAL_READ_FEATURES = 1000232000,
    /// Provided by VK_VERSION_1_4
    RENDERING_ATTACHMENT_LOCATION_INFO = 1000232001,
    /// Provided by VK_VERSION_1_4
    RENDERING_INPUT_ATTACHMENT_INDEX_INFO = 1000232002,
]}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceVulkan14Features.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PhysicalDeviceVulkan14Features<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub global_priority_query: Bool32,
    pub shader_subgroup_rotate: Bool32,
    pub shader_subgroup_rotate_clustered: Bool32,
    pub shader_float_controls2: Bool32,
    pub shader_expect_assume: Bool32,
    pub rectangular_lines: Bool32,
    pub bresenham_lines: Bool32,
    pub smooth_lines: Bool32,
    pub stippled_rectangular_lines: Bool32,
    pub stippled_bresenham_lines: Bool32,
    pub stippled_smooth_lines: Bool32,
    pub vertex_attribute_instance_rate_divisor: Bool32,
    pub vertex_attribute_instance_rate_zero_divisor: Bool32,
    pub index_type_uint8: Bool32,
    pub dynamic_rendering_local_read: Bool32,
    pub maintenance5: Bool32,
    pub maintenance6: Bool32,
    pub pipeline_protected_access: Bool32,
    pub pipeline_robustness: Bool32,
    pub host_image_copy: Bool32,
    pub push_descriptor: Bool32,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for PhysicalDeviceVulkan14Features<'_> {

    const STRUCTURE_TYPE: StructureType = StructureType::PHYSICAL_DEVICE_VULKAN_1_4_FEATURES;
}

impl Default for PhysicalDeviceVulkan14Features<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            global_priority_query: 0,
            shader_subgroup_rotate: 0,
            shader_subgroup_rotate_clustered: 0,
            shader_float_controls2: 0,
            shader_expect_assume: 0,
            rectangular_lines: 0,
            bresenham_lines: 0,
            smooth_lines: 0,
            stippled_rectangular_lines: 0,
            stippled_bresenham_lines: 0,
            stippled_smooth_lines: 0,
            vertex_attribute_instance_rate_divisor: 0,
            vertex_attribute_instance_rate_zero_divisor: 0,
            index_type_uint8: 0,
            dynamic_rendering_local_read: 0,
            maintenance5: 0,
            maintenance6: 0,
            pipeline_protected_access: 0,
            pipeline_robustness: 0,
            host_image_copy: 0,
            push_descriptor: 0,
            _marker: PhantomData,
        }
    }
}

unsafe impl Send for PhysicalDeviceVulkan14Features<'_> {}
unsafe impl Sync for PhysicalDeviceVulkan14Features<'_> {}
unsafe impl ExtendsPhysicalDeviceFeatures2 for PhysicalDeviceVulkan14Features<'_> {}
unsafe impl ExtendsDeviceCreateInfo for PhysicalDeviceVulkan14Features<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceDynamicRenderingLocalReadFeatures.html>
pub type PhysicalDeviceDynamicRenderingLocalReadFeatures<'a> = PhysicalDeviceDynamicRenderingLocalReadFeaturesKHR<'a>;

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceIndexTypeUint8Features.html>
pub type PhysicalDeviceIndexTypeUint8Features<'a> = PhysicalDeviceIndexTypeUint8FeaturesKHR<'a>;

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkRenderingAttachmentLocationInfo.html>
pub type RenderingAttachmentLocationInfo<'a> = RenderingAttachmentLocationInfoKHR<'a>;

crate::macros::ash_style_enum_internal!(
    
    /// Part of Vulkan 1.4 core.
    ///
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineCreateFlags2CreateInfo.html>
    #[flags(u64)]
    pub enum PipelineCreateFlags2 {
        #[display("Disable Optimiziation")]
        DISABLE_OPTIMIZATION = 0x00000001,
        #[display("Alloc Derivatives")]
        ALLOW_DERIVATIVES = 0x00000002,
        #[display("Derivative")]
        DERIVATIVE = 0x00000004,
        #[display("View Index From Device Index")]
        VIEW_INDEX_FROM_DEVICE_INDEX = 0x00000008,
        #[display("Dispatch Base")]
        DISPATCH_BASE = 0x00000010,
        #[display("Fail On Pipeline Compile Required")]
        FAIL_ON_PIPELINE_COMPILE_REQUIRED = 0x00000100,
        #[display("Early Return On Failure")]
        EARLY_RETURN_ON_FAILURE = 0x00000200,
        #[display("No Protected Access")]
        NO_PROTECTED_ACCESS = 0x08000000,
        #[display("Protected Access Only")]
        PROTECTED_ACCESS_ONLY = 0x40000000,
        /// Provided by VK_EXT_descriptor_heap
        #[display("DESCRIPTOR_HEAP_EXT")]
        DESCRIPTOR_HEAP_EXT = 0x1000000000,
        /// Provided by VK_KHR_ray_tracing_pipeline
        #[display("RAY_TRACING_SKIP_BUILT_IN_PRIMITIVES_KHR")]
        RAY_TRACING_SKIP_BUILT_IN_PRIMITIVES_KHR = 0x00001000,
        /// Provided by VK_NV_ray_tracing_linear_swept_spheres
        #[display("RAY_TRACING_ALLOW_SPHERES_AND_LINEAR_SWEPT_SPHERES_NV")]
        RAY_TRACING_ALLOW_SPHERES_AND_LINEAR_SWEPT_SPHERES_NV = 0x200000000,
        /// Provided by VK_EXT_legacy_dithering with (VK_KHR_dynamic_rendering or VK_VERSION_1_3) and (VK_KHR_maintenance5 or VK_VERSION_1_4)
        #[display("ENABLE_LEGACY_DITHERING_EXT")]
        ENABLE_LEGACY_DITHERING_EXT = 0x400000000,
        /// Provided by VK_KHR_maintenance5
        #[display("DISABLE_OPTIMIZATION_KHR")]
        DISABLE_OPTIMIZATION_KHR = 0x00000001,
        /// Provided by VK_KHR_maintenance5
        #[display("ALLOW_DERIVATIVES_KHR")]
        ALLOW_DERIVATIVES_KHR = 0x00000002,
        /// Provided by VK_KHR_maintenance5
        #[display("DERIVATIVE_KHR")]
        DERIVATIVE_KHR = 0x00000004,
        /// Provided by VK_KHR_maintenance5
        #[display("VIEW_INDEX_FROM_DEVICE_INDEX_KHR")]
        VIEW_INDEX_FROM_DEVICE_INDEX_KHR = 0x00000008,
        /// Provided by VK_KHR_maintenance5
        #[display("DISPATCH_BASE_KHR")]
        DISPATCH_BASE_KHR = 0x00000010,
        /// Provided by VK_KHR_maintenance5 with VK_NV_ray_tracing
        #[display("DEFER_COMPILE_NV")]
        DEFER_COMPILE_NV = 0x00000020,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_pipeline_executable_properties
        #[display("CAPTURE_STATISTICS_KHR")]
        CAPTURE_STATISTICS_KHR = 0x00000040,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_pipeline_executable_properties
        #[display("CAPTURE_INTERNAL_REPRESENTATIONS_KHR")]
        CAPTURE_INTERNAL_REPRESENTATIONS_KHR = 0x00000080,
        /// Provided by VK_KHR_maintenance5 with VK_VERSION_1_3 or VK_EXT_pipeline_creation_cache_control
        #[display("FAIL_ON_PIPELINE_COMPILE_REQUIRED_KHR")]
        FAIL_ON_PIPELINE_COMPILE_REQUIRED_KHR = 0x00000100,
        /// Provided by VK_KHR_maintenance5 with VK_VERSION_1_3 or VK_EXT_pipeline_creation_cache_control
        #[display("EARLY_RETURN_ON_FAILURE_KHR")]
        EARLY_RETURN_ON_FAILURE_KHR = 0x00000200,
        /// Provided by VK_KHR_maintenance5 with VK_EXT_graphics_pipeline_library
        #[display("LINK_TIME_OPTIMIZATION_EXT")]
        LINK_TIME_OPTIMIZATION_EXT = 0x00000400,
        /// Provided by VK_KHR_maintenance5 with VK_EXT_graphics_pipeline_library
        #[display("RETAIN_LINK_TIME_OPTIMIZATION_INFO_EXT")]
        RETAIN_LINK_TIME_OPTIMIZATION_INFO_EXT = 0x00800000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_pipeline_library
        #[display("LIBRARY_KHR")]
        LIBRARY_KHR = 0x00000800,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("RAY_TRACING_SKIP_TRIANGLES_KHR")]
        RAY_TRACING_SKIP_TRIANGLES_KHR = 0x00001000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("RAY_TRACING_SKIP_AABBS_KHR")]
        RAY_TRACING_SKIP_AABBS_KHR = 0x00002000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("RAY_TRACING_NO_NULL_ANY_HIT_SHADERS_KHR")]
        RAY_TRACING_NO_NULL_ANY_HIT_SHADERS_KHR = 0x00004000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("RAY_TRACING_NO_NULL_CLOSEST_HIT_SHADERS_KHR")]
        RAY_TRACING_NO_NULL_CLOSEST_HIT_SHADERS_KHR = 0x00008000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("RAY_TRACING_NO_NULL_MISS_SHADERS_KHR")]
        RAY_TRACING_NO_NULL_MISS_SHADERS_KHR = 0x00010000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("RAY_TRACING_NO_NULL_INTERSECTION_SHADERS_KHR")]
        RAY_TRACING_NO_NULL_INTERSECTION_SHADERS_KHR = 0x00020000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("RAY_TRACING_SHADER_GROUP_HANDLE_CAPTURE_REPLAY_KHR")]
        RAY_TRACING_SHADER_GROUP_HANDLE_CAPTURE_REPLAY_KHR = 0x00080000,
        /// Provided by VK_KHR_maintenance5 with VK_NV_device_generated_commands
        #[display("INDIRECT_BINDABLE_NV")]
        INDIRECT_BINDABLE_NV = 0x00040000,
        /// Provided by VK_KHR_maintenance5 with VK_NV_ray_tracing_motion_blur
        #[display("RAY_TRACING_ALLOW_MOTION_NV")]
        RAY_TRACING_ALLOW_MOTION_NV = 0x00100000,
        /// Provided by VK_KHR_maintenance5 with (VK_KHR_dynamic_rendering or VK_VERSION_1_3) and VK_KHR_fragment_shading_rate
        #[display("RENDERING_FRAGMENT_SHADING_RATE_ATTACHMENT_KHR")]
        RENDERING_FRAGMENT_SHADING_RATE_ATTACHMENT_KHR = 0x00200000,
        /// Provided by VK_KHR_maintenance5 with (VK_KHR_dynamic_rendering or VK_VERSION_1_3) and VK_EXT_fragment_density_map
        #[display("RENDERING_FRAGMENT_DENSITY_MAP_ATTACHMENT_EXT")]
        RENDERING_FRAGMENT_DENSITY_MAP_ATTACHMENT_EXT = 0x00400000,
        /// Provided by VK_KHR_maintenance5 with VK_EXT_opacity_micromap
        #[display("RAY_TRACING_OPACITY_MICROMAP_EXT")]
        RAY_TRACING_OPACITY_MICROMAP_EXT = 0x01000000,
        /// Provided by VK_KHR_maintenance5 with VK_EXT_attachment_feedback_loop_layout
        #[display("COLOR_ATTACHMENT_FEEDBACK_LOOP_EXT")]
        COLOR_ATTACHMENT_FEEDBACK_LOOP_EXT = 0x02000000,
        /// Provided by VK_KHR_maintenance5 with VK_EXT_attachment_feedback_loop_layout
        #[display("DEPTH_STENCIL_ATTACHMENT_FEEDBACK_LOOP_EXT")]
        DEPTH_STENCIL_ATTACHMENT_FEEDBACK_LOOP_EXT = 0x04000000,
        /// Provided by VK_KHR_maintenance5 with VK_VERSION_1_4 or VK_EXT_pipeline_protected_access
        #[display("NO_PROTECTED_ACCESS_EXT")]
        NO_PROTECTED_ACCESS_EXT = 0x08000000,
        /// Provided by VK_KHR_maintenance5 with VK_VERSION_1_4 or VK_EXT_pipeline_protected_access
        #[display("PROTECTED_ACCESS_ONLY_EXT")]
        PROTECTED_ACCESS_ONLY_EXT = 0x40000000,
        /// Provided by VK_KHR_maintenance5 with VK_NV_displacement_micromap
        #[display("RAY_TRACING_DISPLACEMENT_MICROMAP_NV")]
        RAY_TRACING_DISPLACEMENT_MICROMAP_NV = 0x10000000,
        /// Provided by VK_KHR_maintenance5 with VK_EXT_descriptor_buffer
        #[display("DESCRIPTOR_BUFFER_EXT")]
        DESCRIPTOR_BUFFER_EXT = 0x20000000,
        /// Provided by VK_KHR_maintenance5 with VK_ARM_pipeline_opacity_micromap, VK_ARM_pipeline_opacity_micromap
        #[display("DISALLOW_OPACITY_MICROMAP_ARM")]
        DISALLOW_OPACITY_MICROMAP_ARM = 0x2000000000,
        /// Provided by VK_KHR_pipeline_binary
        #[display("CAPTURE_DATA_KHR")]
        CAPTURE_DATA_KHR = 0x80000000,
        /// Provided by VK_EXT_device_generated_commands
        #[display("INDIRECT_BINDABLE_EXT")]
        INDIRECT_BINDABLE_EXT = 0x4000000000,
        /// Provided by VK_VALVE_fragment_density_map_layered
        #[display("PER_LAYER_FRAGMENT_DENSITY_VALVE")]
        PER_LAYER_FRAGMENT_DENSITY_VALVE = 0x10000000000,
        /// Provided by VK_EXT_shader_64bit_indexing
        #[display("U64_BIT_INDEXING_EXT")]
        U64_BIT_INDEXING_EXT = 0x80000000000,
    }
);

/// Part of Vulkan 1.4 core.
///
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineCreateFlags2CreateInfo.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PipelineCreateFlags2CreateInfo<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub flags: PipelineCreateFlags2,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for PipelineCreateFlags2CreateInfo<'_> {

    const STRUCTURE_TYPE: StructureType
        = StructureType::PIPELINE_CREATE_FLAGS_2_CREATE_INFO;
}

impl Default for PipelineCreateFlags2CreateInfo<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            flags: PipelineCreateFlags2::empty(),
            _marker: PhantomData,
        }
    }
}

unsafe impl ExtendsGraphicsPipelineCreateInfo for PipelineCreateFlags2CreateInfo<'_> {}
unsafe impl ExtendsComputePipelineCreateInfo for PipelineCreateFlags2CreateInfo<'_> {}
unsafe impl ExtendsRayTracingPipelineCreateInfoKHR for PipelineCreateFlags2CreateInfo<'_> {}
unsafe impl ExtendsRayTracingPipelineCreateInfoNV for PipelineCreateFlags2CreateInfo<'_> {}
unsafe impl Send for PipelineCreateFlags2CreateInfo<'_> {}
unsafe impl Sync for PipelineCreateFlags2CreateInfo<'_> {}

/// VK_KHR_present_id2
pub const KHR_PRESENT_ID_2_NAME: &CStr = c"VK_KHR_present_id2";
/// VK_KHR_present_id2
pub const KHR_PRESENT_ID_2_SPEC_VERSION: u32 = 1;

/// VK_KHR_present_wait2
pub const KHR_PRESENT_WAIT_2_NAME: &CStr = c"VK_KHR_present_wait2";
/// VK_KHR_present_wait2
pub const KHR_PRESENT_WAIT_2_SPEC_VERSION: u32 = 1;

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkSurfaceCapabilitiesPresentId2KHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SurfaceCapabilitiesPresentId2KHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub present_id2_supported: Bool32,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for SurfaceCapabilitiesPresentId2KHR<'_> {

    const STRUCTURE_TYPE: ash::vk::StructureType =
        StructureType::SURFACE_CAPABILITIES_PRESENT_ID_2_KHR;
}

impl Default for SurfaceCapabilitiesPresentId2KHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            present_id2_supported: 0,
            _marker: PhantomData,
        }
    }
}

unsafe impl ExtendsSurfaceCapabilities2KHR for SurfaceCapabilitiesPresentId2KHR<'_> {}
unsafe impl Send for SurfaceCapabilitiesPresentId2KHR<'_> {}
unsafe impl Sync for SurfaceCapabilitiesPresentId2KHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPresentId2KHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PresentId2KHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub swapchain_count: u32,
    pub p_present_ids: *const u64,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for PresentId2KHR<'_> {
    
    const STRUCTURE_TYPE: ash::vk::StructureType =
        StructureType::PRESENT_ID_2_KHR;
}

impl Default for PresentId2KHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            swapchain_count: 0,
            p_present_ids: ptr::null(),
            _marker: PhantomData,
        }
    }
}

unsafe impl ExtendsPresentInfoKHR for PresentId2KHR<'_> {}
unsafe impl Send for PresentId2KHR<'_> {}
unsafe impl Sync for PresentId2KHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevicePresentId2FeaturesKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PhysicalDevicePresentId2FeaturesKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub present_id2: Bool32,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for PhysicalDevicePresentId2FeaturesKHR<'_> {

    const STRUCTURE_TYPE: ash::vk::StructureType =
        StructureType::PHYSICAL_DEVICE_PRESENT_ID_2_FEATURES_KHR;
}

impl Default for PhysicalDevicePresentId2FeaturesKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            present_id2: 0,
            _marker: PhantomData,
        }
    }
}

unsafe impl ExtendsPhysicalDeviceFeatures2 for PhysicalDevicePresentId2FeaturesKHR<'_> {}
unsafe impl ExtendsDeviceCreateInfo for PhysicalDevicePresentId2FeaturesKHR<'_> {}
unsafe impl Send for PhysicalDevicePresentId2FeaturesKHR<'_> {}
unsafe impl Sync for PhysicalDevicePresentId2FeaturesKHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkSurfaceCapabilitiesPresentWait2KHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SurfaceCapabilitiesPresentWait2KHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub present_wait2_supported: Bool32,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for SurfaceCapabilitiesPresentWait2KHR<'_> {

    const STRUCTURE_TYPE: StructureType =
        StructureType::SURFACE_CAPABILITIES_PRESENT_WAIT_2_KHR;
}

impl Default for SurfaceCapabilitiesPresentWait2KHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            present_wait2_supported: 0,
            _marker: PhantomData,
        }
    }
}

unsafe impl ExtendsSurfaceCapabilities2KHR for SurfaceCapabilitiesPresentWait2KHR<'_> {}
unsafe impl Send for SurfaceCapabilitiesPresentWait2KHR<'_> {}
unsafe impl Sync for SurfaceCapabilitiesPresentWait2KHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevicePresentWait2FeaturesKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PhysicalDevicePresentWait2FeaturesKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub present_wait2: Bool32,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for PhysicalDevicePresentWait2FeaturesKHR<'_> {

    const STRUCTURE_TYPE: StructureType =
        StructureType::PHYSICAL_DEVICE_PRESENT_WAIT_2_FEATURES_KHR;
}

impl Default for PhysicalDevicePresentWait2FeaturesKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            present_wait2: 0,
            _marker: PhantomData,
        }
    }
}

unsafe impl ExtendsPhysicalDeviceFeatures2 for PhysicalDevicePresentWait2FeaturesKHR<'_> {}
unsafe impl ExtendsDeviceCreateInfo for PhysicalDevicePresentWait2FeaturesKHR<'_> {}
unsafe impl Send for PhysicalDevicePresentWait2FeaturesKHR<'_> {}
unsafe impl Sync for PhysicalDevicePresentWait2FeaturesKHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPresentWait2InfoKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PresentWait2InfoKHR<'a> { pub s_type: StructureType,
    pub p_next: *const c_void,
    pub present_id: u64,
    pub timeout: u64,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for PresentWait2InfoKHR<'_> {

    const STRUCTURE_TYPE: StructureType =
        StructureType::PRESENT_WAIT_INFO_2_KHR;
}

impl Default for PresentWait2InfoKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            present_id: 0,
            timeout: 0,
            _marker: PhantomData,
        }
    }
}

unsafe impl Send for PresentWait2InfoKHR<'_> {}
unsafe impl Sync for PresentWait2InfoKHR<'_> {}

/// Extension trait for [`SwapchainCreateFlagsKHR`].
///
/// New values:
///
/// [`SwapchainCreateFlagsKHR::PRESENT_ID_2`]
/// [`SwapchainCreateFlagsKHR::PRESENT_WAIT_2`]
pub trait SwapchainCreateFlagsKHR2 {

    /// VK_KHR_present_id2
    const PRESENT_ID_2: Self;
    /// VK_KHR_present_wait2
    const PRESENT_WAIT_2: Self;
}

impl SwapchainCreateFlagsKHR2 for SwapchainCreateFlagsKHR {

    const PRESENT_ID_2: Self = Self::from_raw(0x00000040);
    const PRESENT_WAIT_2: Self = Self::from_raw(0x00000080);
}

/// VK_KHR_present_wait2
#[allow(non_camel_case_types)]
pub type PFN_vkWaitForPresent2KHR = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_present_wait2_info: *const PresentWait2InfoKHR,
) -> Result;

/// VK_KHR_maintenance5
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindIndexBuffer2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    size: DeviceSize,
    index_type: IndexType,
);

/// VK_KHR_dynamic_rendering_local_read
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetRenderingAttachmentLocations = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_location_info: *const RenderingAttachmentLocationInfo,
);

/// VK_KHR_pipeline_binary
pub const KHR_PIPELINE_BINARY_NAME: &CStr = c"VK_KHR_pipeline_binary";
/// VK_KHR_pipeline_binary
pub const KHR_PIPELINE_BINARY_SPEC_VERSION: u32 = 1;

ash::handle_nondispatchable!(
    PipelineBinaryKHR,
    PIPELINE_BINARY_KHR,
    doc = "Provided by VK_KHR_pipeline_binary\n\n<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineBinaryKHR.html>"
);

pub const MAX_PIPELINE_BINARY_KEY_SIZE_KHR: u32 = 32;

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineBinaryKeyKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PipelineBinaryKeyKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub key_size: u32,
    pub key: [u8; MAX_PIPELINE_BINARY_KEY_SIZE_KHR as usize],
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for PipelineBinaryKeyKHR<'_> {

    const STRUCTURE_TYPE: StructureType = StructureType::PIPELINE_BINARY_KEY_KHR;
}

impl Default for PipelineBinaryKeyKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            key_size: 0,
            key: [0; MAX_PIPELINE_BINARY_KEY_SIZE_KHR as usize],
            _marker: PhantomData,
        }
    }
}

unsafe impl Send for PipelineBinaryKeyKHR<'_> {}
unsafe impl Sync for PipelineBinaryKeyKHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineBinaryDataKHR.html>
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct PipelineBinaryDataKHR {
    pub data_size: usize,
    pub p_data: *mut c_void,
}

unsafe impl Send for PipelineBinaryDataKHR {}
unsafe impl Sync for PipelineBinaryDataKHR {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineBinaryKeysAndDataKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PipelineBinaryKeysAndDataKHR<'a> {
    pub binary_count: u32,
    pub p_pipeline_binary_keys: *const PipelineBinaryKeyKHR<'a>,
    pub p_pipeline_binary_data: *const PipelineBinaryDataKHR,
}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineCreateInfoKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PipelineCreateInfoKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for PipelineCreateInfoKHR<'_> {

    const STRUCTURE_TYPE: StructureType = StructureType::PIPELINE_CREATE_INFO_KHR;
}

impl Default for PipelineCreateInfoKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            _marker: PhantomData,
        }
    }
}

unsafe impl Send for PipelineCreateInfoKHR<'_> {}
unsafe impl Sync for PipelineCreateInfoKHR<'_> {}

pub unsafe trait ExtendsPipelineCreateInfoKHR {}

unsafe impl ExtendsPipelineCreateInfoKHR for GraphicsPipelineCreateInfo<'_> {}
unsafe impl ExtendsPipelineCreateInfoKHR for ExecutionGraphPipelineCreateInfoAMDX<'_> {}
unsafe impl ExtendsPipelineCreateInfoKHR for RayTracingPipelineCreateInfoKHR<'_> {}
unsafe impl ExtendsPipelineCreateInfoKHR for ComputePipelineCreateInfo<'_> {}

impl<'a> PipelineCreateInfoKHR<'a> {

    /// Prepends the given extension struct between the root and the first pointer.
    /// This method only exists on structs that can be passed to a function directly.
    /// Only valid extension structs can be pushed into the chain. If the chain
    /// looks like A -> B -> C, and you call x.push_next(&mut D),
    /// then the chain will look like A -> D -> B -> C.
    #[inline(always)]
    pub fn push_next<T: ExtendsPipelineCreateInfoKHR + ?Sized>(
        mut self,
        next: &'a mut T,
    ) -> Self {
        unsafe {
            let next_ptr = <*const T>::cast(next);
            let last_next = ptr_chain_iter(next).last().unwrap();
            (*last_next).p_next = self.p_next as _;
            self.p_next = next_ptr;
        }
        self
    }
}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineBinaryCreateInfoKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PipelineBinaryCreateInfoKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub p_keys_and_data_info: *const PipelineBinaryKeysAndDataKHR<'a>,
    pub pipeline: Pipeline,
    pub pipeline_create_info: *const PipelineCreateInfoKHR<'a>,
}

unsafe impl TaggedStructure for PipelineBinaryCreateInfoKHR<'_> {

    const STRUCTURE_TYPE: StructureType = StructureType::PIPELINE_BINARY_CREATE_INFO_KHR;
}

impl Default for PipelineBinaryCreateInfoKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            p_keys_and_data_info: ptr::null(),
            pipeline: Pipeline::null(),
            pipeline_create_info: ptr::null(),
        }
    }
}

unsafe impl Send for PipelineBinaryCreateInfoKHR<'_> {}
unsafe impl Sync for PipelineBinaryCreateInfoKHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineBinaryHandlesInfoKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PipelineBinaryHandlesInfoKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub pipeline_binary_count: u32,
    pub p_pipeline_binaries: *mut PipelineBinaryKHR,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for PipelineBinaryHandlesInfoKHR<'_> {

    const STRUCTURE_TYPE: StructureType = StructureType::PIPELINE_BINARY_HANDLES_INFO_KHR;
}

impl Default for PipelineBinaryHandlesInfoKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            pipeline_binary_count: 0,
            p_pipeline_binaries: ptr::null_mut(),
            _marker: PhantomData,
        }
    }
}

unsafe impl Send for PipelineBinaryHandlesInfoKHR<'_> {}
unsafe impl Sync for PipelineBinaryHandlesInfoKHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineBinaryDataInfoKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PipelineBinaryDataInfoKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub pipeline_binary: PipelineBinaryKHR,
    pub _marker: PhantomData<&'a ()>
}

unsafe impl TaggedStructure for PipelineBinaryDataInfoKHR<'_> {

    const STRUCTURE_TYPE: StructureType = StructureType::PIPELINE_BINARY_DATA_INFO_KHR;
}

impl Default for PipelineBinaryDataInfoKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            pipeline_binary: PipelineBinaryKHR::null(),
            _marker: PhantomData,
        }
    }
}

unsafe impl Send for PipelineBinaryDataInfoKHR<'_> {}
unsafe impl Sync for PipelineBinaryDataInfoKHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkReleaseCapturedPipelineDataInfoKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ReleaseCapturedPipelineDataInfoKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub pipeline: Pipeline,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for ReleaseCapturedPipelineDataInfoKHR<'_> {

    const STRUCTURE_TYPE: StructureType = StructureType::RELEASE_CAPTURED_PIPELINE_DATA_INFO_KHR;
}

impl Default for ReleaseCapturedPipelineDataInfoKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            pipeline: Pipeline::null(),
            _marker: PhantomData,
        }
    }
}

unsafe impl Send for ReleaseCapturedPipelineDataInfoKHR<'_> {}
unsafe impl Sync for ReleaseCapturedPipelineDataInfoKHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkDevicePipelineBinaryInternalCacheControlKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DevicePipelineBinaryInternalCacheControlKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub disable_internal_cache: Bool32,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for DevicePipelineBinaryInternalCacheControlKHR<'_> {

    const STRUCTURE_TYPE: StructureType
        = StructureType::DEVICE_PIPELINE_BINARY_INTERNAL_CACHE_CONTROL_KHR;
}

impl Default for DevicePipelineBinaryInternalCacheControlKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            disable_internal_cache: 0,
            _marker: PhantomData,
        }
    }
}

unsafe impl ExtendsDeviceCreateInfo for DevicePipelineBinaryInternalCacheControlKHR<'_> {}
unsafe impl Send for DevicePipelineBinaryInternalCacheControlKHR<'_> {}
unsafe impl Sync for DevicePipelineBinaryInternalCacheControlKHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineBinaryInfoKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PipelineBinaryInfoKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub binary_count: u32,
    pub p_pipeline_binaries: *const PipelineBinaryKHR,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for PipelineBinaryInfoKHR<'_> {

    const STRUCTURE_TYPE: ash::vk::StructureType = StructureType::PIPELINE_BINARY_INFO_KHR;
}

impl Default for PipelineBinaryInfoKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            binary_count: 0,
            p_pipeline_binaries: ptr::null(),
            _marker: PhantomData,
        }
    }
}

unsafe impl ExtendsGraphicsPipelineCreateInfo for PipelineBinaryInfoKHR<'_> {}
unsafe impl ExtendsComputePipelineCreateInfo for PipelineBinaryInfoKHR<'_> {}
unsafe impl ExtendsRayTracingPipelineCreateInfoKHR for PipelineBinaryInfoKHR<'_> {}
unsafe impl Send for PipelineBinaryInfoKHR<'_> {}
unsafe impl Sync for PipelineBinaryInfoKHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevicePipelineBinaryFeaturesKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PhysicalDevicePipelineBinaryFeaturesKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub pipeline_binaries: Bool32,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for PhysicalDevicePipelineBinaryFeaturesKHR<'_> {

    const STRUCTURE_TYPE: StructureType
        = StructureType::PHYSICAL_DEVICE_PIPELINE_BINARY_FEATURES_KHR;
}

impl Default for PhysicalDevicePipelineBinaryFeaturesKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            pipeline_binaries: 0,
            _marker: PhantomData,
        }
    }
}

unsafe impl ExtendsPhysicalDeviceFeatures2 for PhysicalDevicePipelineBinaryFeaturesKHR<'_> {}
unsafe impl ExtendsDeviceCreateInfo for PhysicalDevicePipelineBinaryFeaturesKHR<'_> {}
unsafe impl Send for PhysicalDevicePipelineBinaryFeaturesKHR<'_> {}
unsafe impl Sync for PhysicalDevicePipelineBinaryFeaturesKHR<'_> {}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevicePipelineBinaryPropertiesKHR.html>
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PhysicalDevicePipelineBinaryPropertiesKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const c_void,
    pub pipeline_binary_internal_cache: Bool32,
    pub pipeline_binary_internal_cache_control: Bool32,
    pub pipeline_binary_prefers_internal_cache: Bool32,
    pub pipeline_binary_precompiled_internal_cache: Bool32,
    pub pipeline_binary_compressed_data: Bool32,
    pub _marker: PhantomData<&'a ()>,
}

unsafe impl TaggedStructure for PhysicalDevicePipelineBinaryPropertiesKHR<'_> {

    const STRUCTURE_TYPE: StructureType
        = StructureType::PHYSICAL_DEVICE_PIPELINE_BINARY_PROPERTIES_KHR;
}

impl Default for PhysicalDevicePipelineBinaryPropertiesKHR<'_> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            s_type: Self::STRUCTURE_TYPE,
            p_next: ptr::null(),
            pipeline_binary_internal_cache: 0,
            pipeline_binary_internal_cache_control: 0,
            pipeline_binary_prefers_internal_cache: 0,
            pipeline_binary_precompiled_internal_cache: 0,
            pipeline_binary_compressed_data: 0,
            _marker: PhantomData,
        }
    }
}

unsafe impl ExtendsPhysicalDeviceProperties2 for PhysicalDevicePipelineBinaryPropertiesKHR<'_> {}
unsafe impl Send for PhysicalDevicePipelineBinaryPropertiesKHR<'_> {}
unsafe impl Sync for PhysicalDevicePipelineBinaryPropertiesKHR<'_> {}

/// VK_KHR_pipeline_binary
#[allow(non_camel_case_types)]
pub type PFN_vkCreatePipelineBinariesKHR = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const PipelineBinaryCreateInfoKHR<>,
    p_allocator: *const AllocationCallbacks<'_>,
    p_binaries: *mut PipelineBinaryHandlesInfoKHR<'_>,
) -> Result;

/// VK_KHR_pipeline_binary
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyPipelineBinaryKHR = unsafe extern "system" fn(
    device: Device,
    pipeline_binary: PipelineBinaryKHR,
    allocator: *const AllocationCallbacks<'_>,
);

/// VK_KHR_pipeline_binary
#[allow(non_camel_case_types)]
pub type PFN_vkGetPipelineBinaryDataKHR = unsafe extern "system" fn(
    device: Device,
    p_info: *const PipelineBinaryDataInfoKHR<'_>,
    p_pipeline_binary_key: *mut PipelineBinaryKeyKHR<'_>,
    p_pipeline_binary_data_size: *mut usize,
    p_pipeline_binary_data: *mut c_void,
) -> Result;

/// VK_KHR_pipeline_binary
#[allow(non_camel_case_types)]
pub type PFN_vkGetPipelineKeyKHR = unsafe extern "system" fn (
    device: Device,
    p_pipeline_create_info: *const PipelineCreateInfoKHR<'_>,
    p_pipeline_key: *mut PipelineBinaryKeyKHR,
) -> Result;

/// VK_KHR_pipeline_binary
#[allow(non_camel_case_types)]
pub type PFN_vkReleaseCapturedPipelineDataKHR = unsafe extern "system" fn (
    device: Device,
    p_info: *const ReleaseCapturedPipelineDataInfoKHR<'_>,
    p_allocator: *const AllocationCallbacks<'_>
) -> Result;
