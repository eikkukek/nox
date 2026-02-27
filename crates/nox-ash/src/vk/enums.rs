use super::*;

use crate::ash_style_enum;

macro_rules! extend_enum {
    (
        $for:ty => $name:ident[$(#[doc = $doc:literal] $s_type:ident = $lit:literal),* $(,)?]
    ) => {
        pub trait $name {
            $(
                #[doc = $doc]
                const $s_type: Self;
            )*
        }
        
        impl $name for $for {
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

extend_enum! { Result => ResultKHR [
    /// Provided by VK_KHR_pipeline_binary.
    PIPELINE_BINARY_MISSING_KHR = 1000483000,
    /// Provided by VK_KHR_pipeline_binary.
    ERROR_NOT_ENOUGH_SPACE_KHR = -1000483000,
]}

extend_enum! { StructureType => StructureTypeKHR [
    /// Provided by VK_KHR_present_id2.
    SURFACE_CAPABILITIES_PRESENT_ID_2_KHR = 1000479000,
    /// Provided by VK_KHR_present_id2.
    PRESENT_ID_2_KHR = 1000479001,
    /// Provided by VK_KHR_present_id2.
    PHYSICAL_DEVICE_PRESENT_ID_2_FEATURES_KHR = 1000479002,
    /// Provided by VK_KHR_present_wait2.
    SURFACE_CAPABILITIES_PRESENT_WAIT_2_KHR = 1000480000,
    /// Provided by VK_KHR_present_wait2.
    PHYSICAL_DEVICE_PRESENT_WAIT_2_FEATURES_KHR = 1000480001,
    /// Provided by VK_KHR_present_wait2.
    PRESENT_WAIT_INFO_2_KHR = 1000480002,
    /// Provided by VK_KHR_pipeline_binary.
    PHYSICAL_DEVICE_PIPELINE_BINARY_FEATURES_KHR = 1000483000,
    /// Provided by VK_KHR_pipeline_binary.
    PIPELINE_BINARY_CREATE_INFO_KHR = 1000483001,
    /// Provided by VK_KHR_pipeline_binary.
    PIPELINE_BINARY_INFO_KHR = 1000483002,
    /// Provided by VK_KHR_pipeline_binary.
    PIPELINE_BINARY_KEY_KHR = 1000483003,
    /// Provided by VK_KHR_pipeline_binary.
    PHYSICAL_DEVICE_PIPELINE_BINARY_PROPERTIES_KHR = 1000483004,
    /// Provided by VK_KHR_pipeline_binary.
    RELEASE_CAPTURED_PIPELINE_DATA_INFO_KHR = 1000483005,
    /// Provided by VK_KHR_pipeline_binary.
    PIPELINE_BINARY_DATA_INFO_KHR = 1000483006,
    /// Provided by VK_KHR_pipeline_binary.
    PIPELINE_CREATE_INFO_KHR = 1000483007,
    /// Provided by VK_KHR_pipeline_binary.
    DEVICE_PIPELINE_BINARY_INTERNAL_CACHE_CONTROL_KHR = 1000483008,
    /// Provided by VK_KHR_pipeline_binary.
    PIPELINE_BINARY_HANDLES_INFO_KHR = 1000483009,
]}

extend_enum!{ StructureType => StructureType14 [
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_VULKAN_1_4_FEATURES = 55,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_VULKAN_1_4_PROPERTIES = 56,
    /// Provided by Vulkan 1.4.
    DEVICE_QUEUE_GLOBAL_PRIORITY_CREATE_INFO = 1000174000,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_GLOBAL_PRIORITY_QUERY_FEATURES = 1000388000,
    /// Provided by Vulkan 1.4.
    QUEUE_FAMILY_GLOBAL_PRIORITY_PROPERTIES = 1000388001,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_INDEX_TYPE_UINT8_FEATURES = 1000265000,
    /// Provided by Vulkan 1.4.
    MEMORY_MAP_INFO = 1000271000,
    /// Provided by Vulkan 1.4.
    MEMORY_UNMAP_INFO = 1000271001,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_MAINTENANCE_5_FEATURES = 1000470000,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_MAINTENANCE_5_PROPERTIES = 1000470001,
    /// Provided by Vulkan 1.4.
    DEVICE_IMAGE_SUBRESOURCE_INFO = 1000470004,
    /// Provided by Vulkan 1.4.
    SUBRESOURCE_LAYOUT_2 = 1000338002,
    /// Provided by Vulkan 1.4.
    IMAGE_SUBRESOURCE_2 = 1000338003,
    /// Provided by Vulkan 1.4.
    BUFFER_USAGE_FLAGS_2_CREATE_INFO = 1000470006,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_MAINTENANCE_6_FEATURES = 1000545000,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_MAINTENANCE_6_PROPERTIES = 1000545001,
    /// Provided by Vulkan 1.4.
    BIND_MEMORY_STATUS = 1000545002,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_HOST_IMAGE_COPY_FEATURES = 1000270000,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_HOST_IMAGE_COPY_PROPERTIES = 1000270001,
    /// Provided by Vulkan 1.4.
    MEMORY_TO_IMAGE_COPY = 1000270002,
    /// Provided by Vulkan 1.4.
    IMAGE_TO_MEMORY_COPY = 1000270003,
    /// Provided by Vulkan 1.4.
    COPY_IMAGE_TO_MEMORY_INFO = 1000270004,
    /// Provided by Vulkan 1.4.
    COPY_MEMORY_TO_IMAGE_INFO = 1000270005,
    /// Provided by Vulkan 1.4.
    HOST_IMAGE_LAYOUT_TRANSITION_INFO = 1000270006,
    /// Provided by Vulkan 1.4.
    COPY_IMAGE_TO_IMAGE_INFO = 1000270007,
    /// Provided by Vulkan 1.4.
    SUBRESOURCE_HOST_MEMCPY_SIZE = 1000270008,
    /// Provided by Vulkan 1.4.
    HOST_IMAGE_COPY_DEVICE_PERFORMANCE_QUERY = 1000270009,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_SHADER_SUBGROUP_ROTATE_FEATURES = 1000416000,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_SHADER_FLOAT_CONTROLS_2_FEATURES = 1000528000,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_SHADER_EXPECT_ASSUME_FEATURES = 1000544000,
    /// Provided by Vulkan 1.4.
    PIPELINE_CREATE_FLAGS_2_CREATE_INFO = 1000470005,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_PUSH_DESCRIPTOR_PROPERTIES = 1000080000,
    /// Provided by Vulkan 1.4.
    BIND_DESCRIPTOR_SETS_INFO = 1000545003,
    /// Provided by Vulkan 1.4.
    PUSH_CONSTANTS_INFO = 1000545004,
    /// Provided by Vulkan 1.4.
    PUSH_DESCRIPTOR_SET_INFO = 1000545005,
    /// Provided by Vulkan 1.4.
    PUSH_DESCRIPTOR_SET_WITH_TEMPLATE_INFO = 1000545006,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_PIPELINE_PROTECTED_ACCESS_FEATURES = 1000466000,
    /// Provided by Vulkan 1.4.
    PIPELINE_ROBUSTNESS_CREATE_INFO = 1000068000,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_PIPELINE_ROBUSTNESS_FEATURES = 1000068001,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_PIPELINE_ROBUSTNESS_PROPERTIES = 1000068002,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_LINE_RASTERIZATION_FEATURES = 1000259000,
    /// Provided by Vulkan 1.4.
    PIPELINE_RASTERIZATION_LINE_STATE_CREATE_INFO = 1000259001,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_LINE_RASTERIZATION_PROPERTIES = 1000259002,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_PROPERTIES = 1000525000,
    /// Provided by Vulkan 1.4.
    PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO = 1000190001,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES = 1000190002,
    /// Provided by Vulkan 1.4.
    RENDERING_AREA_INFO = 1000470003,
    /// Provided by Vulkan 1.4.
    PHYSICAL_DEVICE_DYNAMIC_RENDERING_LOCAL_READ_FEATURES = 1000232000,
    /// Provided by Vulkan 1.4.
    RENDERING_ATTACHMENT_LOCATION_INFO = 1000232001,
    /// Provided by Vulkan 1.4.
    RENDERING_INPUT_ATTACHMENT_INDEX_INFO = 1000232002,
]}

extend_enum! { ImageLayout => ImageLayout14 [
    /// Provided by Vulkan 1.4.
    RENDERING_LOCAL_READ = 1000232000,
]}

extend_enum! { ImageUsageFlags => ImageUsageFlags14 [
    /// Provided by Vulkan 1.4.
    VK_IMAGE_USAGE_HOST_TRANSFER_BIT = 0x00400000,
]}

ash_style_enum!(
    
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineCreateFlags2CreateInfo.html>
    ///
    /// Provided by Vulkan 1.4.
    #[flags(Flags64)]
    pub enum PipelineCreateFlags2 {
        /// Provided by Vulkan 1.4
        #[display("disable optimiziation")]
        DISABLE_OPTIMIZATION = 0x00000001,
        /// Provided by Vulkan 1.4
        #[display("alloc derivatives")]
        ALLOW_DERIVATIVES = 0x00000002,
        /// Provided by Vulkan 1.4
        #[display("derivative")]
        DERIVATIVE = 0x00000004,
        /// Provided by Vulkan 1.4
        #[display("view index from device index")]
        VIEW_INDEX_FROM_DEVICE_INDEX = 0x00000008,
        /// Provided by Vulkan 1.4
        #[display("dispatch base")]
        DISPATCH_BASE = 0x00000010,
        /// Provided by Vulkan 1.4
        #[display("fail on pipeline compile required")]
        FAIL_ON_PIPELINE_COMPILE_REQUIRED = 0x00000100,
        /// Provided by Vulkan 1.4
        #[display("early return on failure")]
        EARLY_RETURN_ON_FAILURE = 0x00000200,
        /// Provided by Vulkan 1.4
        #[display("no protected access")]
        NO_PROTECTED_ACCESS = 0x08000000,
        /// Provided by Vulkan 1.4
        #[display("protected access only")]
        PROTECTED_ACCESS_ONLY = 0x40000000,
        /// Provided by VK_EXT_descriptor_heap
        #[display("descriptor heap EXT")]
        DESCRIPTOR_HEAP_EXT = 0x1000000000,
        /// Provided by VK_KHR_ray_tracing_pipeline
        #[display("ray tracing skip built in primitives KHR")]
        RAY_TRACING_SKIP_BUILT_IN_PRIMITIVES_KHR = 0x00001000,
        /// Provided by VK_NV_ray_tracing_linear_swept_spheres
        #[display("ray tracing allow spheres and linear swept spheres NV")]
        RAY_TRACING_ALLOW_SPHERES_AND_LINEAR_SWEPT_SPHERES_NV = 0x200000000,
        /// Provided by VK_EXT_legacy_dithering with (VK_KHR_dynamic_rendering or VK_VERSION_1_3)
        /// and (VK_KHR_maintenance5 or Vulkan 1.4)
        #[display("enable legacy dithering EXT")]
        ENABLE_LEGACY_DITHERING_EXT = 0x400000000,
        /// Provided by VK_KHR_maintenance5 with VK_NV_ray_tracing
        #[display("defer compile NV")]
        DEFER_COMPILE_NV = 0x00000020,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_pipeline_executable_properties
        #[display("capture statistics KHR")]
        CAPTURE_STATISTICS_KHR = 0x00000040,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_pipeline_executable_properties
        #[display("capture internal representations KHR")]
        CAPTURE_INTERNAL_REPRESENTATIONS_KHR = 0x00000080,
        /// Provided by VK_KHR_maintenance5 with VK_VERSION_1_3 or VK_EXT_pipeline_creation_cache_control
        #[display("fail on pipeline compile required KHR")]
        FAIL_ON_PIPELINE_COMPILE_REQUIRED_KHR = 0x00000100,
        /// Provided by VK_KHR_maintenance5 with VK_VERSION_1_3 or VK_EXT_pipeline_creation_cache_control
        #[display("early return on failure KHR")]
        EARLY_RETURN_ON_FAILURE_KHR = 0x00000200,
        /// Provided by VK_KHR_maintenance5 with VK_EXT_graphics_pipeline_library
        #[display("link time optimization EXT")]
        LINK_TIME_OPTIMIZATION_EXT = 0x00000400,
        /// Provided by VK_KHR_maintenance5 with VK_EXT_graphics_pipeline_library
        #[display("retain link time optimization info EXT")]
        RETAIN_LINK_TIME_OPTIMIZATION_INFO_EXT = 0x00800000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_pipeline_library
        #[display("library KHR")]
        LIBRARY_KHR = 0x00000800,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("ray tracing skip triangles KHR")]
        RAY_TRACING_SKIP_TRIANGLES_KHR = 0x00001000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("ray tracing skip aabbs KHR")]
        RAY_TRACING_SKIP_AABBS_KHR = 0x00002000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("ray tracing no null any hit shaders KHR")]
        RAY_TRACING_NO_NULL_ANY_HIT_SHADERS_KHR = 0x00004000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("ray tracing no null closest hit shaders KHR")]
        RAY_TRACING_NO_NULL_CLOSEST_HIT_SHADERS_KHR = 0x00008000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("ray tracing no null miss shaders KHR")]
        RAY_TRACING_NO_NULL_MISS_SHADERS_KHR = 0x00010000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("ray tracing no null intersection shaders KHR")]
        RAY_TRACING_NO_NULL_INTERSECTION_SHADERS_KHR = 0x00020000,
        /// Provided by VK_KHR_maintenance5 with VK_KHR_ray_tracing_pipeline
        #[display("ray tracing shader group handle capture replay KHR")]
        RAY_TRACING_SHADER_GROUP_HANDLE_CAPTURE_REPLAY_KHR = 0x00080000,
        /// Provided by VK_KHR_maintenance5 with VK_NV_device_generated_commands
        #[display("indirect bindable NV")]
        INDIRECT_BINDABLE_NV = 0x00040000,
        /// Provided by VK_KHR_maintenance5 with VK_NV_ray_tracing_motion_blur
        #[display("ray tracing allow motion NV")]
        RAY_TRACING_ALLOW_MOTION_NV = 0x00100000,
        /// Provided by VK_KHR_maintenance5 with (VK_KHR_dynamic_rendering or VK_VERSION_1_3) and VK_KHR_fragment_shading_rate
        #[display("rendering fragment shading rate attachment KHR")]
        RENDERING_FRAGMENT_SHADING_RATE_ATTACHMENT_KHR = 0x00200000,
        /// Provided by VK_KHR_maintenance5 with (VK_KHR_dynamic_rendering or VK_VERSION_1_3) and VK_EXT_fragment_density_map
        #[display("rendering fragment density map attachment EXT")]
        RENDERING_FRAGMENT_DENSITY_MAP_ATTACHMENT_EXT = 0x00400000,
        /// Provided by VK_KHR_maintenance5 with VK_EXT_opacity_micromap
        #[display("ray tracing opacity micromap EXT")]
        RAY_TRACING_OPACITY_MICROMAP_EXT = 0x01000000,
        /// Provided by VK_KHR_maintenance5 with VK_EXT_attachment_feedback_loop_layout
        #[display("color attachment feedback loop EXT")]
        COLOR_ATTACHMENT_FEEDBACK_LOOP_EXT = 0x02000000,
        /// Provided by VK_KHR_maintenance5 with VK_EXT_attachment_feedback_loop_layout
        #[display("depth stencil attachment feedback loop EXT")]
        DEPTH_STENCIL_ATTACHMENT_FEEDBACK_LOOP_EXT = 0x04000000,
        /// Provided by VK_KHR_maintenance5 with Vulkan 1.4 or VK_EXT_pipeline_protected_access
        #[display("no protected access EXT")]
        NO_PROTECTED_ACCESS_EXT = 0x08000000,
        /// Provided by VK_KHR_maintenance5 with Vulkan 1.4 or VK_EXT_pipeline_protected_access
        #[display("protected access only EXT")]
        PROTECTED_ACCESS_ONLY_EXT = 0x40000000,
        /// Provided by VK_KHR_maintenance5 with VK_NV_displacement_micromap
        #[display("ray tracing displacement micromap NV")]
        RAY_TRACING_DISPLACEMENT_MICROMAP_NV = 0x10000000,
        /// Provided by VK_KHR_maintenance5 with VK_EXT_descriptor_buffer
        #[display("descriptor buffer EXT")]
        DESCRIPTOR_BUFFER_EXT = 0x20000000,
        /// Provided by VK_KHR_maintenance5 with VK_ARM_pipeline_opacity_micromap, VK_ARM_pipeline_opacity_micromap
        #[display("disallow opacity micromap ARM")]
        DISALLOW_OPACITY_MICROMAP_ARM = 0x2000000000,
        /// Provided by VK_KHR_pipeline_binary
        #[display("capture data KHR")]
        CAPTURE_DATA_KHR = 0x80000000,
        /// Provided by VK_EXT_device_generated_commands
        #[display("indirect bindable EXT")]
        INDIRECT_BINDABLE_EXT = 0x4000000000,
        /// Provided by VK_VALVE_fragment_density_map_layered
        #[display("per layer fragment density VALVE")]
        PER_LAYER_FRAGMENT_DENSITY_VALVE = 0x10000000000,
        /// Provided by VK_EXT_shader_64bit_indexing
        #[display("u64 bit indexing EXT")]
        U64_BIT_INDEXING_EXT = 0x80000000000,
    }
);

impl Default for PipelineCreateFlags2 {

    #[inline(always)]
    fn default() -> Self {
        Self::empty()
    }
}

impl PipelineCreateFlags2KHR {

    /// [`nox-ash`] provides an updated [`PipelineCreateFlags2KHR`], so this function converts itb
    /// to the [`ash`] version.
    #[inline(always)]
    pub fn into_ash(self) -> ash::vk::PipelineCreateFlags2KHR {
        ash::vk::PipelineCreateFlags2KHR::from_raw(self.as_raw())
    }
}

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineCreateFlags2CreateInfo.html>
///
/// VK_KHR_maintenance5
pub type PipelineCreateFlags2KHR = PipelineCreateFlags2;

ash_style_enum!(

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRobustnessBufferBehavior.html>
    ///
    /// Provided by Vulkan 1.4
    #[enum(i32)]
    #[default = Self::DEVICE_DEFAULT]
    pub enum PipelineRobustnessBufferBehavior {
        #[display("device default")]
        DEVICE_DEFAULT = 0,
        #[display("disabled")]
        DISABLED = 1,
        #[display("robust buffer access")]
        ROBUST_BUFFER_ACCESS = 2,
        #[display("robust buffer access 2")]
        ROBUST_BUFFER_ACCESS_2 = 3,
        /// Provided by VK_EXT_pipeline_robustness
        #[display("device default")]
        DEVICE_DEFAULT_EXT = Self::DEVICE_DEFAULT.as_raw(),
        /// Provided by VK_EXT_pipeline_robustness
        #[display("disabled")]
        DISABLED_EXT = Self::DISABLED.as_raw(),
        /// Provided by VK_EXT_pipeline_robustness
        #[display("robust buffer access")]
        ROBUST_BUFFER_ACCESS_EXT = Self::ROBUST_BUFFER_ACCESS.as_raw(),
        /// Provided by VK_EXT_pipeline_robustness
        #[display("robust buffer access 2")]
        ROBUST_BUFFER_ACCESS_2_EXT = Self::ROBUST_BUFFER_ACCESS_2.as_raw(),
    }

    #[enum(i32)]
    #[default = Self::DEVICE_DEFAULT]
    pub enum PipelineRobustnessImageBehavior {
        #[display("device default")]
        DEVICE_DEFAULT = 0,
        #[display("disabled")]
        DISABLED = 1,
        #[display("robust image access")]
        ROBUST_IMAGE_ACCESS = 2,
        #[display("robust image access 2")]
        ROBUST_IMAGE_ACCESS_2 = 3,
        /// Provided by VK_EXT_pipeline_robustness
        #[display("device default")]
        DEVICE_DEFAULT_EXT = Self::DEVICE_DEFAULT.as_raw(),
        /// Provided by VK_EXT_pipeline_robustness
        #[display("disabled")]
        DISABLED_EXT = Self::DISABLED.as_raw(),
        /// Provided by VK_EXT_pipeline_robustness
        #[display("robust image access")]
        ROBUST_IMAGE_ACCESS_EXT = Self::ROBUST_IMAGE_ACCESS.as_raw(),
        /// Provided by VK_EXT_pipeline_robustness
        #[display("robust image access 2")]
        ROBUST_IMAGE_ACCESS_2_EXT = Self::ROBUST_IMAGE_ACCESS_2.as_raw(),
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/VkIndexType.html>
    #[enum(i32)]
    pub enum IndexType {
        #[display("uint16")]
        UINT16 = 0,
        #[display("uint32")]
        UINT32 = 1,
        /// Provided by Vulkan 1.4
        #[display("uint8")]
        UINT8 = 1000265000,
        /// Provided by VK_KHR_acceleration_structure
        #[display("none KHR")]
        NONE_KHR = 1000165000,
        /// Provided by VK_NV_ray_tracing
        #[display("none NV")]
        NONE_NV = Self::NONE_KHR.as_raw(),
        /// Provided by VK_EXT_index_type_uint8
        #[display("uint8 EXT")]
        UINT8_EXT = Self::UINT8.as_raw(),
        /// Provided by VK_KHR_index_type_uint8
        #[display("uint8 KHR")]
        UINT8_KHR = Self::UINT8.as_raw(),
    }
);

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRobustnessBufferBehavior.html>
/// 
/// Provided by VK_EXT_pipeline_robustness.
pub type PipelineRobustnessBufferBehaviorEXT = PipelineRobustnessBufferBehavior;

/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRobustnessImageBehavior.html>
///
/// Provided by VK_EXT_pipeline_robustness.
pub type PipelineRobustnessImageBehaviorEXT = PipelineRobustnessImageBehavior;

impl IndexType {

    /// [`nox-ash`] provides an updated [`IndexType`], so this function converts it to the [`ash`]
    /// version when you need it.
    #[inline(always)]
    pub fn into_ash(self) -> ash::vk::IndexType {
        ash::vk::IndexType::from_raw(
            self.as_raw()
        )
    }
}
