use super::*;
#[allow(non_camel_case_types)]
pub type PFN_vkInternalAllocationNotification = unsafe extern "system" fn(
    p_user_data: *mut ffi::c_void,
    size: usize,
    allocation_type: InternalAllocationType,
    allocation_scope: SystemAllocationScope,
);
impl PFN_Default for PFN_vkInternalAllocationNotification {
    #[inline]
    #[allow(unused_variables)]
    fn default() -> Self {
        unsafe extern "system" fn f(
            p_user_data: *mut ffi::c_void,
            size: usize,
            allocation_type: InternalAllocationType,
            allocation_scope: SystemAllocationScope,
        ) {
            panic!(concat!(
                stringify!(PFN_vkInternalAllocationNotification),
                " not defined"
            ))
        }
        f
    }
}
#[allow(non_camel_case_types)]
pub type PFN_vkInternalFreeNotification = unsafe extern "system" fn(
    p_user_data: *mut ffi::c_void,
    size: usize,
    allocation_type: InternalAllocationType,
    allocation_scope: SystemAllocationScope,
);
#[allow(non_camel_case_types)]
pub type PFN_vkReallocationFunction = unsafe extern "system" fn(
    p_user_data: *mut ffi::c_void,
    p_original: *mut ffi::c_void,
    size: usize,
    alignment: usize,
    allocation_scope: SystemAllocationScope,
) -> *mut ffi::c_void;
impl PFN_Default for PFN_vkReallocationFunction {
    #[inline]
    #[allow(unused_variables)]
    fn default() -> Self {
        unsafe extern "system" fn f(
            p_user_data: *mut ffi::c_void,
            p_original: *mut ffi::c_void,
            size: usize,
            alignment: usize,
            allocation_scope: SystemAllocationScope,
        ) -> *mut ffi::c_void {
            panic!(concat!(
                stringify!(PFN_vkReallocationFunction),
                " not defined"
            ))
        }
        f
    }
}
#[allow(non_camel_case_types)]
pub type PFN_vkAllocationFunction = unsafe extern "system" fn(
    p_user_data: *mut ffi::c_void,
    size: usize,
    alignment: usize,
    allocation_scope: SystemAllocationScope,
) -> *mut ffi::c_void;
impl PFN_Default for PFN_vkAllocationFunction {
    #[inline]
    #[allow(unused_variables)]
    fn default() -> Self {
        unsafe extern "system" fn f(
            p_user_data: *mut ffi::c_void,
            size: usize,
            alignment: usize,
            allocation_scope: SystemAllocationScope,
        ) -> *mut ffi::c_void {
            panic!(concat!(
                stringify!(PFN_vkAllocationFunction),
                " not defined"
            ))
        }
        f
    }
}
#[allow(non_camel_case_types)]
pub type PFN_vkFreeFunction =
    unsafe extern "system" fn(p_user_data: *mut ffi::c_void, p_memory: *mut ffi::c_void);
impl PFN_Default for PFN_vkFreeFunction {
    #[inline]
    #[allow(unused_variables)]
    fn default() -> Self {
        unsafe extern "system" fn f(p_user_data: *mut ffi::c_void, p_memory: *mut ffi::c_void) {
            panic!(concat!(stringify!(PFN_vkFreeFunction), " not defined"))
        }
        f
    }
}
#[allow(non_camel_case_types)]
pub type PFN_vkVoidFunction = unsafe extern "system" fn();
impl PFN_Default for PFN_vkVoidFunction {
    #[inline]
    #[allow(unused_variables)]
    fn default() -> Self {
        unsafe extern "system" fn f() {
            panic!(concat!(stringify!(PFN_vkVoidFunction), " not defined"))
        }
        f
    }
}
#[allow(non_camel_case_types)]
pub type PFN_vkDebugReportCallbackEXT = unsafe extern "system" fn(
    flags: DebugReportFlagsEXT,
    object_type: DebugReportObjectTypeEXT,
    object: u64,
    location: usize,
    message_code: i32,
    p_layer_prefix: *const ffi::c_char,
    p_message: *const ffi::c_char,
    p_user_data: *mut ffi::c_void,
) -> Bool32;
impl PFN_Default for PFN_vkDebugReportCallbackEXT {
    #[inline]
    #[allow(unused_variables)]
    fn default() -> Self {
        unsafe extern "system" fn f(
            flags: DebugReportFlagsEXT,
            object_type: DebugReportObjectTypeEXT,
            object: u64,
            location: usize,
            message_code: i32,
            p_layer_prefix: *const ffi::c_char,
            p_message: *const ffi::c_char,
            p_user_data: *mut ffi::c_void,
        ) -> Bool32 {
            panic!(concat!(
                stringify!(PFN_vkDebugReportCallbackEXT),
                " not defined"
            ))
        }
        f
    }
}
#[allow(non_camel_case_types)]
pub type PFN_vkDebugUtilsMessengerCallbackEXT = unsafe extern "system" fn(
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    message_types: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut ffi::c_void,
) -> Bool32;
impl PFN_Default for PFN_vkDebugUtilsMessengerCallbackEXT {
    #[inline]
    #[allow(unused_variables)]
    fn default() -> Self {
        unsafe extern "system" fn f(
            message_severity: DebugUtilsMessageSeverityFlagsEXT,
            message_types: DebugUtilsMessageTypeFlagsEXT,
            p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
            p_user_data: *mut ffi::c_void,
        ) -> Bool32 {
            panic!(concat!(
                stringify!(PFN_vkDebugUtilsMessengerCallbackEXT),
                " not defined"
            ))
        }
        f
    }
}
#[allow(non_camel_case_types)]
pub type PFN_vkDeviceMemoryReportCallbackEXT = unsafe extern "system" fn(
    p_callback_data: *const DeviceMemoryReportCallbackDataEXT,
    p_user_data: *mut ffi::c_void,
);
impl PFN_Default for PFN_vkDeviceMemoryReportCallbackEXT {
    #[inline]
    #[allow(unused_variables)]
    fn default() -> Self {
        unsafe extern "system" fn f(
            p_callback_data: *const DeviceMemoryReportCallbackDataEXT,
            p_user_data: *mut ffi::c_void,
        ) {
            panic!(concat!(
                stringify!(PFN_vkDeviceMemoryReportCallbackEXT),
                " not defined"
            ))
        }
        f
    }
}
#[allow(non_camel_case_types)]
pub type PFN_vkGetInstanceProcAddrLUNARG =
    unsafe extern "system" fn(instance: Instance, p_name: *const ffi::c_char) -> PFN_vkVoidFunction;
impl PFN_Default for PFN_vkGetInstanceProcAddrLUNARG {
    #[inline]
    #[allow(unused_variables)]
    fn default() -> Self {
        unsafe extern "system" fn f(
            instance: Instance,
            p_name: *const ffi::c_char,
        ) -> PFN_vkVoidFunction {
            panic!(concat!(
                stringify!(PFN_vkGetInstanceProcAddrLUNARG),
                " not defined"
            ))
        }
        f
    }
}
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateInstance = unsafe extern "system" fn(
    p_create_info: *const InstanceCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_instance: *mut Instance,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyInstance =
    unsafe extern "system" fn(instance: Instance, p_allocator: *const AllocationCallbacks);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkEnumeratePhysicalDevices = unsafe extern "system" fn(
    instance: Instance,
    p_physical_device_count: *mut u32,
    p_physical_devices: *mut PhysicalDevice,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceProcAddr =
    unsafe extern "system" fn(device: Device, p_name: *const ffi::c_char) -> PFN_vkVoidFunction;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetInstanceProcAddr =
    unsafe extern "system" fn(instance: Instance, p_name: *const ffi::c_char) -> PFN_vkVoidFunction;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_properties: *mut PhysicalDeviceProperties,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceQueueFamilyProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_queue_family_property_count: *mut u32,
    p_queue_family_properties: *mut QueueFamilyProperties,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceMemoryProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_memory_properties: *mut PhysicalDeviceMemoryProperties,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceFeatures = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_features: *mut PhysicalDeviceFeatures,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceFormatProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    format: Format,
    p_format_properties: *mut FormatProperties,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceImageFormatProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    format: Format,
    ty: ImageType,
    tiling: ImageTiling,
    usage: ImageUsageFlags,
    flags: ImageCreateFlags,
    p_image_format_properties: *mut ImageFormatProperties,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDevice = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_create_info: *const DeviceCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_device: *mut Device,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyDevice =
    unsafe extern "system" fn(device: Device, p_allocator: *const AllocationCallbacks);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkEnumerateInstanceVersion =
    unsafe extern "system" fn(p_api_version: *mut u32) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkEnumerateInstanceLayerProperties = unsafe extern "system" fn(
    p_property_count: *mut u32,
    p_properties: *mut LayerProperties,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkEnumerateInstanceExtensionProperties = unsafe extern "system" fn(
    p_layer_name: *const ffi::c_char,
    p_property_count: *mut u32,
    p_properties: *mut ExtensionProperties,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkEnumerateDeviceLayerProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_property_count: *mut u32,
    p_properties: *mut LayerProperties,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkEnumerateDeviceExtensionProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_layer_name: *const ffi::c_char,
    p_property_count: *mut u32,
    p_properties: *mut ExtensionProperties,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceQueue = unsafe extern "system" fn(
    device: Device,
    queue_family_index: u32,
    queue_index: u32,
    p_queue: *mut Queue,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkQueueSubmit = unsafe extern "system" fn(
    queue: Queue,
    submit_count: u32,
    p_submits: *const SubmitInfo,
    fence: Fence,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkQueueWaitIdle = unsafe extern "system" fn(queue: Queue) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDeviceWaitIdle = unsafe extern "system" fn(device: Device) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkAllocateMemory = unsafe extern "system" fn(
    device: Device,
    p_allocate_info: *const MemoryAllocateInfo,
    p_allocator: *const AllocationCallbacks,
    p_memory: *mut DeviceMemory,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkFreeMemory = unsafe extern "system" fn(
    device: Device,
    memory: DeviceMemory,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkMapMemory = unsafe extern "system" fn(
    device: Device,
    memory: DeviceMemory,
    offset: DeviceSize,
    size: DeviceSize,
    flags: MemoryMapFlags,
    pp_data: *mut *mut ffi::c_void,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkUnmapMemory = unsafe extern "system" fn(device: Device, memory: DeviceMemory);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkFlushMappedMemoryRanges = unsafe extern "system" fn(
    device: Device,
    memory_range_count: u32,
    p_memory_ranges: *const MappedMemoryRange,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkInvalidateMappedMemoryRanges = unsafe extern "system" fn(
    device: Device,
    memory_range_count: u32,
    p_memory_ranges: *const MappedMemoryRange,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceMemoryCommitment = unsafe extern "system" fn(
    device: Device,
    memory: DeviceMemory,
    p_committed_memory_in_bytes: *mut DeviceSize,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetBufferMemoryRequirements = unsafe extern "system" fn(
    device: Device,
    buffer: Buffer,
    p_memory_requirements: *mut MemoryRequirements,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkBindBufferMemory = unsafe extern "system" fn(
    device: Device,
    buffer: Buffer,
    memory: DeviceMemory,
    memory_offset: DeviceSize,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageMemoryRequirements = unsafe extern "system" fn(
    device: Device,
    image: Image,
    p_memory_requirements: *mut MemoryRequirements,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkBindImageMemory = unsafe extern "system" fn(
    device: Device,
    image: Image,
    memory: DeviceMemory,
    memory_offset: DeviceSize,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageSparseMemoryRequirements = unsafe extern "system" fn(
    device: Device,
    image: Image,
    p_sparse_memory_requirement_count: *mut u32,
    p_sparse_memory_requirements: *mut SparseImageMemoryRequirements,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSparseImageFormatProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    format: Format,
    ty: ImageType,
    samples: SampleCountFlags,
    usage: ImageUsageFlags,
    tiling: ImageTiling,
    p_property_count: *mut u32,
    p_properties: *mut SparseImageFormatProperties,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkQueueBindSparse = unsafe extern "system" fn(
    queue: Queue,
    bind_info_count: u32,
    p_bind_info: *const BindSparseInfo,
    fence: Fence,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateFence = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const FenceCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_fence: *mut Fence,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyFence = unsafe extern "system" fn(
    device: Device,
    fence: Fence,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkResetFences =
    unsafe extern "system" fn(device: Device, fence_count: u32, p_fences: *const Fence) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetFenceStatus = unsafe extern "system" fn(device: Device, fence: Fence) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkWaitForFences = unsafe extern "system" fn(
    device: Device,
    fence_count: u32,
    p_fences: *const Fence,
    wait_all: Bool32,
    timeout: u64,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateSemaphore = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const SemaphoreCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_semaphore: *mut Semaphore,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroySemaphore = unsafe extern "system" fn(
    device: Device,
    semaphore: Semaphore,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateEvent = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const EventCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_event: *mut Event,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyEvent = unsafe extern "system" fn(
    device: Device,
    event: Event,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetEventStatus = unsafe extern "system" fn(device: Device, event: Event) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkSetEvent = unsafe extern "system" fn(device: Device, event: Event) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkResetEvent = unsafe extern "system" fn(device: Device, event: Event) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateQueryPool = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const QueryPoolCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_query_pool: *mut QueryPool,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyQueryPool = unsafe extern "system" fn(
    device: Device,
    query_pool: QueryPool,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetQueryPoolResults = unsafe extern "system" fn(
    device: Device,
    query_pool: QueryPool,
    first_query: u32,
    query_count: u32,
    data_size: usize,
    p_data: *mut ffi::c_void,
    stride: DeviceSize,
    flags: QueryResultFlags,
) -> Result;
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkResetQueryPool = unsafe extern "system" fn(
    device: Device,
    query_pool: QueryPool,
    first_query: u32,
    query_count: u32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateBuffer = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const BufferCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_buffer: *mut Buffer,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyBuffer = unsafe extern "system" fn(
    device: Device,
    buffer: Buffer,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateBufferView = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const BufferViewCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_view: *mut BufferView,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyBufferView = unsafe extern "system" fn(
    device: Device,
    buffer_view: BufferView,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateImage = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const ImageCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_image: *mut Image,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyImage = unsafe extern "system" fn(
    device: Device,
    image: Image,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageSubresourceLayout = unsafe extern "system" fn(
    device: Device,
    image: Image,
    p_subresource: *const ImageSubresource,
    p_layout: *mut SubresourceLayout,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateImageView = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const ImageViewCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_view: *mut ImageView,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyImageView = unsafe extern "system" fn(
    device: Device,
    image_view: ImageView,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateShaderModule = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const ShaderModuleCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_shader_module: *mut ShaderModule,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyShaderModule = unsafe extern "system" fn(
    device: Device,
    shader_module: ShaderModule,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreatePipelineCache = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const PipelineCacheCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_pipeline_cache: *mut PipelineCache,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyPipelineCache = unsafe extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPipelineCacheData = unsafe extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    p_data_size: *mut usize,
    p_data: *mut ffi::c_void,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkMergePipelineCaches = unsafe extern "system" fn(
    device: Device,
    dst_cache: PipelineCache,
    src_cache_count: u32,
    p_src_caches: *const PipelineCache,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreatePipelineBinariesKHR = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const PipelineBinaryCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_binaries: *mut PipelineBinaryHandlesInfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyPipelineBinaryKHR = unsafe extern "system" fn(
    device: Device,
    pipeline_binary: PipelineBinaryKHR,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPipelineKeyKHR = unsafe extern "system" fn(
    device: Device,
    p_pipeline_create_info: *const PipelineCreateInfoKHR,
    p_pipeline_key: *mut PipelineBinaryKeyKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPipelineBinaryDataKHR = unsafe extern "system" fn(
    device: Device,
    p_info: *const PipelineBinaryDataInfoKHR,
    p_pipeline_binary_key: *mut PipelineBinaryKeyKHR,
    p_pipeline_binary_data_size: *mut usize,
    p_pipeline_binary_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkReleaseCapturedPipelineDataKHR = unsafe extern "system" fn(
    device: Device,
    p_info: *const ReleaseCapturedPipelineDataInfoKHR,
    p_allocator: *const AllocationCallbacks,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateGraphicsPipelines = unsafe extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    create_info_count: u32,
    p_create_infos: *const GraphicsPipelineCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_pipelines: *mut Pipeline,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateComputePipelines = unsafe extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    create_info_count: u32,
    p_create_infos: *const ComputePipelineCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_pipelines: *mut Pipeline,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceSubpassShadingMaxWorkgroupSizeHUAWEI = unsafe extern "system" fn(
    device: Device,
    renderpass: RenderPass,
    p_max_workgroup_size: *mut Extent2D,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyPipeline = unsafe extern "system" fn(
    device: Device,
    pipeline: Pipeline,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreatePipelineLayout = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const PipelineLayoutCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_pipeline_layout: *mut PipelineLayout,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyPipelineLayout = unsafe extern "system" fn(
    device: Device,
    pipeline_layout: PipelineLayout,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateSampler = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const SamplerCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_sampler: *mut Sampler,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroySampler = unsafe extern "system" fn(
    device: Device,
    sampler: Sampler,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDescriptorSetLayout = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const DescriptorSetLayoutCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_set_layout: *mut DescriptorSetLayout,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyDescriptorSetLayout = unsafe extern "system" fn(
    device: Device,
    descriptor_set_layout: DescriptorSetLayout,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDescriptorPool = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const DescriptorPoolCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_descriptor_pool: *mut DescriptorPool,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyDescriptorPool = unsafe extern "system" fn(
    device: Device,
    descriptor_pool: DescriptorPool,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkResetDescriptorPool = unsafe extern "system" fn(
    device: Device,
    descriptor_pool: DescriptorPool,
    flags: DescriptorPoolResetFlags,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkAllocateDescriptorSets = unsafe extern "system" fn(
    device: Device,
    p_allocate_info: *const DescriptorSetAllocateInfo,
    p_descriptor_sets: *mut DescriptorSet,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkFreeDescriptorSets = unsafe extern "system" fn(
    device: Device,
    descriptor_pool: DescriptorPool,
    descriptor_set_count: u32,
    p_descriptor_sets: *const DescriptorSet,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkUpdateDescriptorSets = unsafe extern "system" fn(
    device: Device,
    descriptor_write_count: u32,
    p_descriptor_writes: *const WriteDescriptorSet,
    descriptor_copy_count: u32,
    p_descriptor_copies: *const CopyDescriptorSet,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateFramebuffer = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const FramebufferCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_framebuffer: *mut Framebuffer,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyFramebuffer = unsafe extern "system" fn(
    device: Device,
    framebuffer: Framebuffer,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateRenderPass = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const RenderPassCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_render_pass: *mut RenderPass,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyRenderPass = unsafe extern "system" fn(
    device: Device,
    render_pass: RenderPass,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetRenderAreaGranularity = unsafe extern "system" fn(
    device: Device,
    render_pass: RenderPass,
    p_granularity: *mut Extent2D,
);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetRenderingAreaGranularity = unsafe extern "system" fn(
    device: Device,
    p_rendering_area_info: *const RenderingAreaInfo,
    p_granularity: *mut Extent2D,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateCommandPool = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const CommandPoolCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_command_pool: *mut CommandPool,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyCommandPool = unsafe extern "system" fn(
    device: Device,
    command_pool: CommandPool,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkResetCommandPool = unsafe extern "system" fn(
    device: Device,
    command_pool: CommandPool,
    flags: CommandPoolResetFlags,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkAllocateCommandBuffers = unsafe extern "system" fn(
    device: Device,
    p_allocate_info: *const CommandBufferAllocateInfo,
    p_command_buffers: *mut CommandBuffer,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkFreeCommandBuffers = unsafe extern "system" fn(
    device: Device,
    command_pool: CommandPool,
    command_buffer_count: u32,
    p_command_buffers: *const CommandBuffer,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkBeginCommandBuffer = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_begin_info: *const CommandBufferBeginInfo,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkEndCommandBuffer =
    unsafe extern "system" fn(command_buffer: CommandBuffer) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkResetCommandBuffer = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    flags: CommandBufferResetFlags,
) -> Result;
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindPipeline = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_bind_point: PipelineBindPoint,
    pipeline: Pipeline,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetPrimitiveRestartIndexEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, primitive_restart_index: u32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetAttachmentFeedbackLoopEnableEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, aspect_mask: ImageAspectFlags);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetViewport = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_viewport: u32,
    viewport_count: u32,
    p_viewports: *const Viewport,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetScissor = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_scissor: u32,
    scissor_count: u32,
    p_scissors: *const Rect2D,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetLineWidth =
    unsafe extern "system" fn(command_buffer: CommandBuffer, line_width: f32);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthBias = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    depth_bias_constant_factor: f32,
    depth_bias_clamp: f32,
    depth_bias_slope_factor: f32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetBlendConstants =
    unsafe extern "system" fn(command_buffer: CommandBuffer, blend_constants: [f32; 4usize]);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthBounds = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    min_depth_bounds: f32,
    max_depth_bounds: f32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetStencilCompareMask = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    face_mask: StencilFaceFlags,
    compare_mask: u32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetStencilWriteMask = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    face_mask: StencilFaceFlags,
    write_mask: u32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetStencilReference = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    face_mask: StencilFaceFlags,
    reference: u32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindDescriptorSets = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_bind_point: PipelineBindPoint,
    layout: PipelineLayout,
    first_set: u32,
    descriptor_set_count: u32,
    p_descriptor_sets: *const DescriptorSet,
    dynamic_offset_count: u32,
    p_dynamic_offsets: *const u32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindIndexBuffer = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    index_type: IndexType,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindVertexBuffers = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_binding: u32,
    binding_count: u32,
    p_buffers: *const Buffer,
    p_offsets: *const DeviceSize,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDraw = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndexed = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    vertex_offset: i32,
    first_instance: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawMultiEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    draw_count: u32,
    p_vertex_info: *const MultiDrawInfoEXT,
    instance_count: u32,
    first_instance: u32,
    stride: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawMultiIndexedEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    draw_count: u32,
    p_index_info: *const MultiDrawIndexedInfoEXT,
    instance_count: u32,
    first_instance: u32,
    stride: u32,
    p_vertex_offset: *const i32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndirect = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    draw_count: u32,
    stride: u32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndexedIndirect = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    draw_count: u32,
    stride: u32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDispatch = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    group_count_x: u32,
    group_count_y: u32,
    group_count_z: u32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDispatchIndirect =
    unsafe extern "system" fn(command_buffer: CommandBuffer, buffer: Buffer, offset: DeviceSize);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSubpassShadingHUAWEI = unsafe extern "system" fn(command_buffer: CommandBuffer);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawClusterHUAWEI = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    group_count_x: u32,
    group_count_y: u32,
    group_count_z: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawClusterIndirectHUAWEI =
    unsafe extern "system" fn(command_buffer: CommandBuffer, buffer: Buffer, offset: DeviceSize);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdUpdatePipelineIndirectBufferNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_bind_point: PipelineBindPoint,
    pipeline: Pipeline,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyBuffer = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    src_buffer: Buffer,
    dst_buffer: Buffer,
    region_count: u32,
    p_regions: *const BufferCopy,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyImage = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    src_image: Image,
    src_image_layout: ImageLayout,
    dst_image: Image,
    dst_image_layout: ImageLayout,
    region_count: u32,
    p_regions: *const ImageCopy,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBlitImage = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    src_image: Image,
    src_image_layout: ImageLayout,
    dst_image: Image,
    dst_image_layout: ImageLayout,
    region_count: u32,
    p_regions: *const ImageBlit,
    filter: Filter,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyBufferToImage = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    src_buffer: Buffer,
    dst_image: Image,
    dst_image_layout: ImageLayout,
    region_count: u32,
    p_regions: *const BufferImageCopy,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyImageToBuffer = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    src_image: Image,
    src_image_layout: ImageLayout,
    dst_buffer: Buffer,
    region_count: u32,
    p_regions: *const BufferImageCopy,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyMemoryIndirectNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    copy_buffer_address: DeviceAddress,
    copy_count: u32,
    stride: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyMemoryIndirectKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_copy_memory_indirect_info: *const CopyMemoryIndirectInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyMemoryToImageIndirectNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    copy_buffer_address: DeviceAddress,
    copy_count: u32,
    stride: u32,
    dst_image: Image,
    dst_image_layout: ImageLayout,
    p_image_subresources: *const ImageSubresourceLayers,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyMemoryToImageIndirectKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_copy_memory_to_image_indirect_info: *const CopyMemoryToImageIndirectInfoKHR,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdUpdateBuffer = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    dst_buffer: Buffer,
    dst_offset: DeviceSize,
    data_size: DeviceSize,
    p_data: *const ffi::c_void,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdFillBuffer = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    dst_buffer: Buffer,
    dst_offset: DeviceSize,
    size: DeviceSize,
    data: u32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdClearColorImage = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    image: Image,
    image_layout: ImageLayout,
    p_color: *const ClearColorValue,
    range_count: u32,
    p_ranges: *const ImageSubresourceRange,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdClearDepthStencilImage = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    image: Image,
    image_layout: ImageLayout,
    p_depth_stencil: *const ClearDepthStencilValue,
    range_count: u32,
    p_ranges: *const ImageSubresourceRange,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdClearAttachments = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    attachment_count: u32,
    p_attachments: *const ClearAttachment,
    rect_count: u32,
    p_rects: *const ClearRect,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdResolveImage = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    src_image: Image,
    src_image_layout: ImageLayout,
    dst_image: Image,
    dst_image_layout: ImageLayout,
    region_count: u32,
    p_regions: *const ImageResolve,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetEvent = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    event: Event,
    stage_mask: PipelineStageFlags,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdResetEvent = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    event: Event,
    stage_mask: PipelineStageFlags,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdWaitEvents = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    event_count: u32,
    p_events: *const Event,
    src_stage_mask: PipelineStageFlags,
    dst_stage_mask: PipelineStageFlags,
    memory_barrier_count: u32,
    p_memory_barriers: *const MemoryBarrier,
    buffer_memory_barrier_count: u32,
    p_buffer_memory_barriers: *const BufferMemoryBarrier,
    image_memory_barrier_count: u32,
    p_image_memory_barriers: *const ImageMemoryBarrier,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPipelineBarrier = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    src_stage_mask: PipelineStageFlags,
    dst_stage_mask: PipelineStageFlags,
    dependency_flags: DependencyFlags,
    memory_barrier_count: u32,
    p_memory_barriers: *const MemoryBarrier,
    buffer_memory_barrier_count: u32,
    p_buffer_memory_barriers: *const BufferMemoryBarrier,
    image_memory_barrier_count: u32,
    p_image_memory_barriers: *const ImageMemoryBarrier,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginQuery = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    query_pool: QueryPool,
    query: u32,
    flags: QueryControlFlags,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndQuery =
    unsafe extern "system" fn(command_buffer: CommandBuffer, query_pool: QueryPool, query: u32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginConditionalRenderingEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_conditional_rendering_begin: *const ConditionalRenderingBeginInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndConditionalRenderingEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginCustomResolveEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_begin_custom_resolve_info: *const BeginCustomResolveInfoEXT,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdResetQueryPool = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    query_pool: QueryPool,
    first_query: u32,
    query_count: u32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdWriteTimestamp = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_stage: PipelineStageFlags,
    query_pool: QueryPool,
    query: u32,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyQueryPoolResults = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    query_pool: QueryPool,
    first_query: u32,
    query_count: u32,
    dst_buffer: Buffer,
    dst_offset: DeviceSize,
    stride: DeviceSize,
    flags: QueryResultFlags,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPushConstants = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    layout: PipelineLayout,
    stage_flags: ShaderStageFlags,
    offset: u32,
    size: u32,
    p_values: *const ffi::c_void,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginRenderPass = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_render_pass_begin: *const RenderPassBeginInfo,
    contents: SubpassContents,
);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdNextSubpass =
    unsafe extern "system" fn(command_buffer: CommandBuffer, contents: SubpassContents);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndRenderPass = unsafe extern "system" fn(command_buffer: CommandBuffer);
#[doc = "Provided by Vulkan version 1.0"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdExecuteCommands = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    command_buffer_count: u32,
    p_command_buffers: *const CommandBuffer,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateAndroidSurfaceKHR = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const AndroidSurfaceCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateSurfaceOHOS = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const SurfaceCreateInfoOHOS,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceDisplayPropertiesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_property_count: *mut u32,
    p_properties: *mut DisplayPropertiesKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceDisplayPlanePropertiesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_property_count: *mut u32,
    p_properties: *mut DisplayPlanePropertiesKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDisplayPlaneSupportedDisplaysKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    plane_index: u32,
    p_display_count: *mut u32,
    p_displays: *mut DisplayKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDisplayModePropertiesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    display: DisplayKHR,
    p_property_count: *mut u32,
    p_properties: *mut DisplayModePropertiesKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDisplayModeKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    display: DisplayKHR,
    p_create_info: *const DisplayModeCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_mode: *mut DisplayModeKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDisplayPlaneCapabilitiesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    mode: DisplayModeKHR,
    plane_index: u32,
    p_capabilities: *mut DisplayPlaneCapabilitiesKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDisplayPlaneSurfaceKHR = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const DisplaySurfaceCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateSharedSwapchainsKHR = unsafe extern "system" fn(
    device: Device,
    swapchain_count: u32,
    p_create_infos: *const SwapchainCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_swapchains: *mut SwapchainKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroySurfaceKHR = unsafe extern "system" fn(
    instance: Instance,
    surface: SurfaceKHR,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSurfaceSupportKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    queue_family_index: u32,
    surface: SurfaceKHR,
    p_supported: *mut Bool32,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    p_surface_capabilities: *mut SurfaceCapabilitiesKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSurfaceFormatsKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    p_surface_format_count: *mut u32,
    p_surface_formats: *mut SurfaceFormatKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSurfacePresentModesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    p_present_mode_count: *mut u32,
    p_present_modes: *mut PresentModeKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateSwapchainKHR = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const SwapchainCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_swapchain: *mut SwapchainKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroySwapchainKHR = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetSwapchainImagesKHR = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_swapchain_image_count: *mut u32,
    p_swapchain_images: *mut Image,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkAcquireNextImageKHR = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    timeout: u64,
    semaphore: Semaphore,
    fence: Fence,
    p_image_index: *mut u32,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkQueuePresentKHR =
    unsafe extern "system" fn(queue: Queue, p_present_info: *const PresentInfoKHR) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateViSurfaceNN = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const ViSurfaceCreateInfoNN,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateWaylandSurfaceKHR = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const WaylandSurfaceCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceWaylandPresentationSupportKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    queue_family_index: u32,
    display: *mut wl_display,
)
    -> Bool32;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateUbmSurfaceSEC = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const UbmSurfaceCreateInfoSEC,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceUbmPresentationSupportSEC = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    queue_family_index: u32,
    device: *mut ubm_device,
) -> Bool32;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateWin32SurfaceKHR = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const Win32SurfaceCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceWin32PresentationSupportKHR =
    unsafe extern "system" fn(physical_device: PhysicalDevice, queue_family_index: u32) -> Bool32;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateXlibSurfaceKHR = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const XlibSurfaceCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceXlibPresentationSupportKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    queue_family_index: u32,
    dpy: *mut Display,
    visual_id: VisualID,
) -> Bool32;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateXcbSurfaceKHR = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const XcbSurfaceCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceXcbPresentationSupportKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    queue_family_index: u32,
    connection: *mut xcb_connection_t,
    visual_id: xcb_visualid_t,
) -> Bool32;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDirectFBSurfaceEXT = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const DirectFBSurfaceCreateInfoEXT,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceDirectFBPresentationSupportEXT =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        queue_family_index: u32,
        dfb: *mut IDirectFB,
    ) -> Bool32;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateImagePipeSurfaceFUCHSIA = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const ImagePipeSurfaceCreateInfoFUCHSIA,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateStreamDescriptorSurfaceGGP = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const StreamDescriptorSurfaceCreateInfoGGP,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateScreenSurfaceQNX = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const ScreenSurfaceCreateInfoQNX,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceScreenPresentationSupportQNX = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    queue_family_index: u32,
    window: *mut _screen_window,
) -> Bool32;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDebugReportCallbackEXT = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const DebugReportCallbackCreateInfoEXT,
    p_allocator: *const AllocationCallbacks,
    p_callback: *mut DebugReportCallbackEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyDebugReportCallbackEXT = unsafe extern "system" fn(
    instance: Instance,
    callback: DebugReportCallbackEXT,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkDebugReportMessageEXT = unsafe extern "system" fn(
    instance: Instance,
    flags: DebugReportFlagsEXT,
    object_type: DebugReportObjectTypeEXT,
    object: u64,
    location: usize,
    message_code: i32,
    p_layer_prefix: *const ffi::c_char,
    p_message: *const ffi::c_char,
);
#[allow(non_camel_case_types)]
pub type PFN_vkDebugMarkerSetObjectNameEXT = unsafe extern "system" fn(
    device: Device,
    p_name_info: *const DebugMarkerObjectNameInfoEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDebugMarkerSetObjectTagEXT = unsafe extern "system" fn(
    device: Device,
    p_tag_info: *const DebugMarkerObjectTagInfoEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDebugMarkerBeginEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_marker_info: *const DebugMarkerMarkerInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDebugMarkerEndEXT = unsafe extern "system" fn(command_buffer: CommandBuffer);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDebugMarkerInsertEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_marker_info: *const DebugMarkerMarkerInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceExternalImageFormatPropertiesNV =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        format: Format,
        ty: ImageType,
        tiling: ImageTiling,
        usage: ImageUsageFlags,
        flags: ImageCreateFlags,
        external_handle_type: ExternalMemoryHandleTypeFlagsNV,
        p_external_image_format_properties: *mut ExternalImageFormatPropertiesNV,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryWin32HandleNV = unsafe extern "system" fn(
    device: Device,
    memory: DeviceMemory,
    handle_type: ExternalMemoryHandleTypeFlagsNV,
    p_handle: *mut HANDLE,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdExecuteGeneratedCommandsNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    is_preprocessed: Bool32,
    p_generated_commands_info: *const GeneratedCommandsInfoNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPreprocessGeneratedCommandsNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_generated_commands_info: *const GeneratedCommandsInfoNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindPipelineShaderGroupNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_bind_point: PipelineBindPoint,
    pipeline: Pipeline,
    group_index: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetGeneratedCommandsMemoryRequirementsNV = unsafe extern "system" fn(
    device: Device,
    p_info: *const GeneratedCommandsMemoryRequirementsInfoNV,
    p_memory_requirements: *mut MemoryRequirements2,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateIndirectCommandsLayoutNV = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const IndirectCommandsLayoutCreateInfoNV,
    p_allocator: *const AllocationCallbacks,
    p_indirect_commands_layout: *mut IndirectCommandsLayoutNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyIndirectCommandsLayoutNV = unsafe extern "system" fn(
    device: Device,
    indirect_commands_layout: IndirectCommandsLayoutNV,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdExecuteGeneratedCommandsEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    is_preprocessed: Bool32,
    p_generated_commands_info: *const GeneratedCommandsInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPreprocessGeneratedCommandsEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_generated_commands_info: *const GeneratedCommandsInfoEXT,
    state_command_buffer: CommandBuffer,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetGeneratedCommandsMemoryRequirementsEXT = unsafe extern "system" fn(
    device: Device,
    p_info: *const GeneratedCommandsMemoryRequirementsInfoEXT,
    p_memory_requirements: *mut MemoryRequirements2,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateIndirectCommandsLayoutEXT = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const IndirectCommandsLayoutCreateInfoEXT,
    p_allocator: *const AllocationCallbacks,
    p_indirect_commands_layout: *mut IndirectCommandsLayoutEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyIndirectCommandsLayoutEXT = unsafe extern "system" fn(
    device: Device,
    indirect_commands_layout: IndirectCommandsLayoutEXT,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateIndirectExecutionSetEXT = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const IndirectExecutionSetCreateInfoEXT,
    p_allocator: *const AllocationCallbacks,
    p_indirect_execution_set: *mut IndirectExecutionSetEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyIndirectExecutionSetEXT = unsafe extern "system" fn(
    device: Device,
    indirect_execution_set: IndirectExecutionSetEXT,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkUpdateIndirectExecutionSetPipelineEXT = unsafe extern "system" fn(
    device: Device,
    indirect_execution_set: IndirectExecutionSetEXT,
    execution_set_write_count: u32,
    p_execution_set_writes: *const WriteIndirectExecutionSetPipelineEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkUpdateIndirectExecutionSetShaderEXT = unsafe extern "system" fn(
    device: Device,
    indirect_execution_set: IndirectExecutionSetEXT,
    execution_set_write_count: u32,
    p_execution_set_writes: *const WriteIndirectExecutionSetShaderEXT,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceFeatures2 = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_features: *mut PhysicalDeviceFeatures2,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceProperties2 = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_properties: *mut PhysicalDeviceProperties2,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceFormatProperties2 = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    format: Format,
    p_format_properties: *mut FormatProperties2,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceImageFormatProperties2 = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_image_format_info: *const PhysicalDeviceImageFormatInfo2,
    p_image_format_properties: *mut ImageFormatProperties2,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceQueueFamilyProperties2 = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_queue_family_property_count: *mut u32,
    p_queue_family_properties: *mut QueueFamilyProperties2,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceMemoryProperties2 = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_memory_properties: *mut PhysicalDeviceMemoryProperties2,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSparseImageFormatProperties2 = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_format_info: *const PhysicalDeviceSparseImageFormatInfo2,
    p_property_count: *mut u32,
    p_properties: *mut SparseImageFormatProperties2,
);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPushDescriptorSet = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_bind_point: PipelineBindPoint,
    layout: PipelineLayout,
    set: u32,
    descriptor_write_count: u32,
    p_descriptor_writes: *const WriteDescriptorSet,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkTrimCommandPool = unsafe extern "system" fn(
    device: Device,
    command_pool: CommandPool,
    flags: CommandPoolTrimFlags,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceExternalBufferProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_external_buffer_info: *const PhysicalDeviceExternalBufferInfo,
    p_external_buffer_properties: *mut ExternalBufferProperties,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryWin32HandleKHR = unsafe extern "system" fn(
    device: Device,
    p_get_win32_handle_info: *const MemoryGetWin32HandleInfoKHR,
    p_handle: *mut HANDLE,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryWin32HandlePropertiesKHR = unsafe extern "system" fn(
    device: Device,
    handle_type: ExternalMemoryHandleTypeFlags,
    handle: HANDLE,
    p_memory_win32_handle_properties: *mut MemoryWin32HandlePropertiesKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryFdKHR = unsafe extern "system" fn(
    device: Device,
    p_get_fd_info: *const MemoryGetFdInfoKHR,
    p_fd: *mut ffi::c_int,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryFdPropertiesKHR = unsafe extern "system" fn(
    device: Device,
    handle_type: ExternalMemoryHandleTypeFlags,
    fd: ffi::c_int,
    p_memory_fd_properties: *mut MemoryFdPropertiesKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryZirconHandleFUCHSIA = unsafe extern "system" fn(
    device: Device,
    p_get_zircon_handle_info: *const MemoryGetZirconHandleInfoFUCHSIA,
    p_zircon_handle: *mut zx_handle_t,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryZirconHandlePropertiesFUCHSIA = unsafe extern "system" fn(
    device: Device,
    handle_type: ExternalMemoryHandleTypeFlags,
    zircon_handle: zx_handle_t,
    p_memory_zircon_handle_properties: *mut MemoryZirconHandlePropertiesFUCHSIA,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryRemoteAddressNV = unsafe extern "system" fn(
    device: Device,
    p_memory_get_remote_address_info: *const MemoryGetRemoteAddressInfoNV,
    p_address: *mut RemoteAddressNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemorySciBufNV = unsafe extern "system" fn(
    device: Device,
    p_get_sci_buf_info: *const MemoryGetSciBufInfoNV,
    p_handle: *mut NvSciBufObj,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceExternalMemorySciBufPropertiesNV =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        handle_type: ExternalMemoryHandleTypeFlags,
        handle: NvSciBufObj,
        p_memory_sci_buf_properties: *mut MemorySciBufPropertiesNV,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSciBufAttributesNV = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_attributes: NvSciBufAttrList,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceExternalSemaphoreProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_external_semaphore_info: *const PhysicalDeviceExternalSemaphoreInfo,
    p_external_semaphore_properties: *mut ExternalSemaphoreProperties,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetSemaphoreWin32HandleKHR = unsafe extern "system" fn(
    device: Device,
    p_get_win32_handle_info: *const SemaphoreGetWin32HandleInfoKHR,
    p_handle: *mut HANDLE,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkImportSemaphoreWin32HandleKHR = unsafe extern "system" fn(
    device: Device,
    p_import_semaphore_win32_handle_info: *const ImportSemaphoreWin32HandleInfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetSemaphoreFdKHR = unsafe extern "system" fn(
    device: Device,
    p_get_fd_info: *const SemaphoreGetFdInfoKHR,
    p_fd: *mut ffi::c_int,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkImportSemaphoreFdKHR = unsafe extern "system" fn(
    device: Device,
    p_import_semaphore_fd_info: *const ImportSemaphoreFdInfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetSemaphoreZirconHandleFUCHSIA = unsafe extern "system" fn(
    device: Device,
    p_get_zircon_handle_info: *const SemaphoreGetZirconHandleInfoFUCHSIA,
    p_zircon_handle: *mut zx_handle_t,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkImportSemaphoreZirconHandleFUCHSIA = unsafe extern "system" fn(
    device: Device,
    p_import_semaphore_zircon_handle_info: *const ImportSemaphoreZirconHandleInfoFUCHSIA,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceExternalFenceProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_external_fence_info: *const PhysicalDeviceExternalFenceInfo,
    p_external_fence_properties: *mut ExternalFenceProperties,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetFenceWin32HandleKHR = unsafe extern "system" fn(
    device: Device,
    p_get_win32_handle_info: *const FenceGetWin32HandleInfoKHR,
    p_handle: *mut HANDLE,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkImportFenceWin32HandleKHR = unsafe extern "system" fn(
    device: Device,
    p_import_fence_win32_handle_info: *const ImportFenceWin32HandleInfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetFenceFdKHR = unsafe extern "system" fn(
    device: Device,
    p_get_fd_info: *const FenceGetFdInfoKHR,
    p_fd: *mut ffi::c_int,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkImportFenceFdKHR = unsafe extern "system" fn(
    device: Device,
    p_import_fence_fd_info: *const ImportFenceFdInfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetFenceSciSyncFenceNV = unsafe extern "system" fn(
    device: Device,
    p_get_sci_sync_handle_info: *const FenceGetSciSyncInfoNV,
    p_handle: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetFenceSciSyncObjNV = unsafe extern "system" fn(
    device: Device,
    p_get_sci_sync_handle_info: *const FenceGetSciSyncInfoNV,
    p_handle: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkImportFenceSciSyncFenceNV = unsafe extern "system" fn(
    device: Device,
    p_import_fence_sci_sync_info: *const ImportFenceSciSyncInfoNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkImportFenceSciSyncObjNV = unsafe extern "system" fn(
    device: Device,
    p_import_fence_sci_sync_info: *const ImportFenceSciSyncInfoNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetSemaphoreSciSyncObjNV = unsafe extern "system" fn(
    device: Device,
    p_get_sci_sync_info: *const SemaphoreGetSciSyncInfoNV,
    p_handle: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkImportSemaphoreSciSyncObjNV = unsafe extern "system" fn(
    device: Device,
    p_import_semaphore_sci_sync_info: *const ImportSemaphoreSciSyncInfoNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSciSyncAttributesNV = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_sci_sync_attributes_info: *const SciSyncAttributesInfoNV,
    p_attributes: NvSciSyncAttrList,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateSemaphoreSciSyncPoolNV = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const SemaphoreSciSyncPoolCreateInfoNV,
    p_allocator: *const AllocationCallbacks,
    p_semaphore_pool: *mut SemaphoreSciSyncPoolNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroySemaphoreSciSyncPoolNV = unsafe extern "system" fn(
    device: Device,
    semaphore_pool: SemaphoreSciSyncPoolNV,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkReleaseDisplayEXT =
    unsafe extern "system" fn(physical_device: PhysicalDevice, display: DisplayKHR) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkAcquireXlibDisplayEXT = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    dpy: *mut Display,
    display: DisplayKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetRandROutputDisplayEXT = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    dpy: *mut Display,
    rr_output: RROutput,
    p_display: *mut DisplayKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkAcquireWinrtDisplayNV =
    unsafe extern "system" fn(physical_device: PhysicalDevice, display: DisplayKHR) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetWinrtDisplayNV = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    device_relative_id: u32,
    p_display: *mut DisplayKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDisplayPowerControlEXT = unsafe extern "system" fn(
    device: Device,
    display: DisplayKHR,
    p_display_power_info: *const DisplayPowerInfoEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkRegisterDeviceEventEXT = unsafe extern "system" fn(
    device: Device,
    p_device_event_info: *const DeviceEventInfoEXT,
    p_allocator: *const AllocationCallbacks,
    p_fence: *mut Fence,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkRegisterDisplayEventEXT = unsafe extern "system" fn(
    device: Device,
    display: DisplayKHR,
    p_display_event_info: *const DisplayEventInfoEXT,
    p_allocator: *const AllocationCallbacks,
    p_fence: *mut Fence,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetSwapchainCounterEXT = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    counter: SurfaceCounterFlagsEXT,
    p_counter_value: *mut u64,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSurfaceCapabilities2EXT = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    p_surface_capabilities: *mut SurfaceCapabilities2EXT,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkEnumeratePhysicalDeviceGroups = unsafe extern "system" fn(
    instance: Instance,
    p_physical_device_group_count: *mut u32,
    p_physical_device_group_properties: *mut PhysicalDeviceGroupProperties,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceGroupPeerMemoryFeatures = unsafe extern "system" fn(
    device: Device,
    heap_index: u32,
    local_device_index: u32,
    remote_device_index: u32,
    p_peer_memory_features: *mut PeerMemoryFeatureFlags,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkBindBufferMemory2 = unsafe extern "system" fn(
    device: Device,
    bind_info_count: u32,
    p_bind_infos: *const BindBufferMemoryInfo,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkBindImageMemory2 = unsafe extern "system" fn(
    device: Device,
    bind_info_count: u32,
    p_bind_infos: *const BindImageMemoryInfo,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDeviceMask =
    unsafe extern "system" fn(command_buffer: CommandBuffer, device_mask: u32);
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceGroupPresentCapabilitiesKHR = unsafe extern "system" fn(
    device: Device,
    p_device_group_present_capabilities: *mut DeviceGroupPresentCapabilitiesKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceGroupSurfacePresentModesKHR = unsafe extern "system" fn(
    device: Device,
    surface: SurfaceKHR,
    p_modes: *mut DeviceGroupPresentModeFlagsKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkAcquireNextImage2KHR = unsafe extern "system" fn(
    device: Device,
    p_acquire_info: *const AcquireNextImageInfoKHR,
    p_image_index: *mut u32,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDispatchBase = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    base_group_x: u32,
    base_group_y: u32,
    base_group_z: u32,
    group_count_x: u32,
    group_count_y: u32,
    group_count_z: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDevicePresentRectanglesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    p_rect_count: *mut u32,
    p_rects: *mut Rect2D,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDescriptorUpdateTemplate = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const DescriptorUpdateTemplateCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_descriptor_update_template: *mut DescriptorUpdateTemplate,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyDescriptorUpdateTemplate = unsafe extern "system" fn(
    device: Device,
    descriptor_update_template: DescriptorUpdateTemplate,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkUpdateDescriptorSetWithTemplate = unsafe extern "system" fn(
    device: Device,
    descriptor_set: DescriptorSet,
    descriptor_update_template: DescriptorUpdateTemplate,
    p_data: *const ffi::c_void,
);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPushDescriptorSetWithTemplate = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    descriptor_update_template: DescriptorUpdateTemplate,
    layout: PipelineLayout,
    set: u32,
    p_data: *const ffi::c_void,
);
#[allow(non_camel_case_types)]
pub type PFN_vkSetHdrMetadataEXT = unsafe extern "system" fn(
    device: Device,
    swapchain_count: u32,
    p_swapchains: *const SwapchainKHR,
    p_metadata: *const HdrMetadataEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetSwapchainStatusKHR =
    unsafe extern "system" fn(device: Device, swapchain: SwapchainKHR) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetRefreshCycleDurationGOOGLE = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_display_timing_properties: *mut RefreshCycleDurationGOOGLE,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPastPresentationTimingGOOGLE = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_presentation_timing_count: *mut u32,
    p_presentation_timings: *mut PastPresentationTimingGOOGLE,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateIOSSurfaceMVK = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const IOSSurfaceCreateInfoMVK,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateMacOSSurfaceMVK = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const MacOSSurfaceCreateInfoMVK,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateMetalSurfaceEXT = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const MetalSurfaceCreateInfoEXT,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetViewportWScalingNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_viewport: u32,
    viewport_count: u32,
    p_viewport_wscalings: *const ViewportWScalingNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDiscardRectangleEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_discard_rectangle: u32,
    discard_rectangle_count: u32,
    p_discard_rectangles: *const Rect2D,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDiscardRectangleEnableEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, discard_rectangle_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDiscardRectangleModeEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    discard_rectangle_mode: DiscardRectangleModeEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetSampleLocationsEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_sample_locations_info: *const SampleLocationsInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceMultisamplePropertiesEXT = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    samples: SampleCountFlags,
    p_multisample_properties: *mut MultisamplePropertiesEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSurfaceCapabilities2KHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_surface_info: *const PhysicalDeviceSurfaceInfo2KHR,
    p_surface_capabilities: *mut SurfaceCapabilities2KHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSurfaceFormats2KHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_surface_info: *const PhysicalDeviceSurfaceInfo2KHR,
    p_surface_format_count: *mut u32,
    p_surface_formats: *mut SurfaceFormat2KHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceDisplayProperties2KHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_property_count: *mut u32,
    p_properties: *mut DisplayProperties2KHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceDisplayPlaneProperties2KHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_property_count: *mut u32,
    p_properties: *mut DisplayPlaneProperties2KHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDisplayModeProperties2KHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    display: DisplayKHR,
    p_property_count: *mut u32,
    p_properties: *mut DisplayModeProperties2KHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDisplayPlaneCapabilities2KHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_display_plane_info: *const DisplayPlaneInfo2KHR,
    p_capabilities: *mut DisplayPlaneCapabilities2KHR,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetBufferMemoryRequirements2 = unsafe extern "system" fn(
    device: Device,
    p_info: *const BufferMemoryRequirementsInfo2,
    p_memory_requirements: *mut MemoryRequirements2,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageMemoryRequirements2 = unsafe extern "system" fn(
    device: Device,
    p_info: *const ImageMemoryRequirementsInfo2,
    p_memory_requirements: *mut MemoryRequirements2,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageSparseMemoryRequirements2 = unsafe extern "system" fn(
    device: Device,
    p_info: *const ImageSparseMemoryRequirementsInfo2,
    p_sparse_memory_requirement_count: *mut u32,
    p_sparse_memory_requirements: *mut SparseImageMemoryRequirements2,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceBufferMemoryRequirements = unsafe extern "system" fn(
    device: Device,
    p_info: *const DeviceBufferMemoryRequirements,
    p_memory_requirements: *mut MemoryRequirements2,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceImageMemoryRequirements = unsafe extern "system" fn(
    device: Device,
    p_info: *const DeviceImageMemoryRequirements,
    p_memory_requirements: *mut MemoryRequirements2,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceImageSparseMemoryRequirements = unsafe extern "system" fn(
    device: Device,
    p_info: *const DeviceImageMemoryRequirements,
    p_sparse_memory_requirement_count: *mut u32,
    p_sparse_memory_requirements: *mut SparseImageMemoryRequirements2,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateSamplerYcbcrConversion = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const SamplerYcbcrConversionCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_ycbcr_conversion: *mut SamplerYcbcrConversion,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroySamplerYcbcrConversion = unsafe extern "system" fn(
    device: Device,
    ycbcr_conversion: SamplerYcbcrConversion,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceQueue2 = unsafe extern "system" fn(
    device: Device,
    p_queue_info: *const DeviceQueueInfo2,
    p_queue: *mut Queue,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateValidationCacheEXT = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const ValidationCacheCreateInfoEXT,
    p_allocator: *const AllocationCallbacks,
    p_validation_cache: *mut ValidationCacheEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyValidationCacheEXT = unsafe extern "system" fn(
    device: Device,
    validation_cache: ValidationCacheEXT,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetValidationCacheDataEXT = unsafe extern "system" fn(
    device: Device,
    validation_cache: ValidationCacheEXT,
    p_data_size: *mut usize,
    p_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkMergeValidationCachesEXT = unsafe extern "system" fn(
    device: Device,
    dst_cache: ValidationCacheEXT,
    src_cache_count: u32,
    p_src_caches: *const ValidationCacheEXT,
) -> Result;
#[doc = "Provided by Vulkan version 1.1"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDescriptorSetLayoutSupport = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const DescriptorSetLayoutCreateInfo,
    p_support: *mut DescriptorSetLayoutSupport,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetSwapchainGrallocUsageANDROID = unsafe extern "system" fn(
    device: Device,
    format: Format,
    image_usage: ImageUsageFlags,
    gralloc_usage: *mut ffi::c_int,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetSwapchainGrallocUsage2ANDROID = unsafe extern "system" fn(
    device: Device,
    format: Format,
    image_usage: ImageUsageFlags,
    swapchain_image_usage: SwapchainImageUsageFlagsANDROID,
    gralloc_consumer_usage: *mut u64,
    gralloc_producer_usage: *mut u64,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkAcquireImageANDROID = unsafe extern "system" fn(
    device: Device,
    image: Image,
    native_fence_fd: ffi::c_int,
    semaphore: Semaphore,
    fence: Fence,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkQueueSignalReleaseImageANDROID = unsafe extern "system" fn(
    queue: Queue,
    wait_semaphore_count: u32,
    p_wait_semaphores: *const Semaphore,
    image: Image,
    p_native_fence_fd: *mut ffi::c_int,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetShaderInfoAMD = unsafe extern "system" fn(
    device: Device,
    pipeline: Pipeline,
    shader_stage: ShaderStageFlags,
    info_type: ShaderInfoTypeAMD,
    p_info_size: *mut usize,
    p_info: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkSetLocalDimmingAMD = unsafe extern "system" fn(
    device: Device,
    swap_chain: SwapchainKHR,
    local_dimming_enable: Bool32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceCalibrateableTimeDomainsKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_time_domain_count: *mut u32,
    p_time_domains: *mut TimeDomainKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetCalibratedTimestampsKHR = unsafe extern "system" fn(
    device: Device,
    timestamp_count: u32,
    p_timestamp_infos: *const CalibratedTimestampInfoKHR,
    p_timestamps: *mut u64,
    p_max_deviation: *mut u64,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkSetDebugUtilsObjectNameEXT = unsafe extern "system" fn(
    device: Device,
    p_name_info: *const DebugUtilsObjectNameInfoEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkSetDebugUtilsObjectTagEXT = unsafe extern "system" fn(
    device: Device,
    p_tag_info: *const DebugUtilsObjectTagInfoEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkQueueBeginDebugUtilsLabelEXT =
    unsafe extern "system" fn(queue: Queue, p_label_info: *const DebugUtilsLabelEXT);
#[allow(non_camel_case_types)]
pub type PFN_vkQueueEndDebugUtilsLabelEXT = unsafe extern "system" fn(queue: Queue);
#[allow(non_camel_case_types)]
pub type PFN_vkQueueInsertDebugUtilsLabelEXT =
    unsafe extern "system" fn(queue: Queue, p_label_info: *const DebugUtilsLabelEXT);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginDebugUtilsLabelEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_label_info: *const DebugUtilsLabelEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndDebugUtilsLabelEXT = unsafe extern "system" fn(command_buffer: CommandBuffer);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdInsertDebugUtilsLabelEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_label_info: *const DebugUtilsLabelEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDebugUtilsMessengerEXT = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const DebugUtilsMessengerCreateInfoEXT,
    p_allocator: *const AllocationCallbacks,
    p_messenger: *mut DebugUtilsMessengerEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyDebugUtilsMessengerEXT = unsafe extern "system" fn(
    instance: Instance,
    messenger: DebugUtilsMessengerEXT,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkSubmitDebugUtilsMessageEXT = unsafe extern "system" fn(
    instance: Instance,
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    message_types: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryHostPointerPropertiesEXT = unsafe extern "system" fn(
    device: Device,
    handle_type: ExternalMemoryHandleTypeFlags,
    p_host_pointer: *const ffi::c_void,
    p_memory_host_pointer_properties: *mut MemoryHostPointerPropertiesEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdWriteBufferMarkerAMD = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_stage: PipelineStageFlags,
    dst_buffer: Buffer,
    dst_offset: DeviceSize,
    marker: u32,
);
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateRenderPass2 = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const RenderPassCreateInfo2,
    p_allocator: *const AllocationCallbacks,
    p_render_pass: *mut RenderPass,
) -> Result;
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginRenderPass2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_render_pass_begin: *const RenderPassBeginInfo,
    p_subpass_begin_info: *const SubpassBeginInfo,
);
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdNextSubpass2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_subpass_begin_info: *const SubpassBeginInfo,
    p_subpass_end_info: *const SubpassEndInfo,
);
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndRenderPass2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_subpass_end_info: *const SubpassEndInfo,
);
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetSemaphoreCounterValue =
    unsafe extern "system" fn(device: Device, semaphore: Semaphore, p_value: *mut u64) -> Result;
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkWaitSemaphores = unsafe extern "system" fn(
    device: Device,
    p_wait_info: *const SemaphoreWaitInfo,
    timeout: u64,
) -> Result;
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkSignalSemaphore =
    unsafe extern "system" fn(device: Device, p_signal_info: *const SemaphoreSignalInfo) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetAndroidHardwareBufferPropertiesANDROID = unsafe extern "system" fn(
    device: Device,
    buffer: *const AHardwareBuffer,
    p_properties: *mut AndroidHardwareBufferPropertiesANDROID,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryAndroidHardwareBufferANDROID = unsafe extern "system" fn(
    device: Device,
    p_info: *const MemoryGetAndroidHardwareBufferInfoANDROID,
    p_buffer: *mut *mut AHardwareBuffer,
) -> Result;
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndirectCount = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    count_buffer: Buffer,
    count_buffer_offset: DeviceSize,
    max_draw_count: u32,
    stride: u32,
);
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndexedIndirectCount = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    count_buffer: Buffer,
    count_buffer_offset: DeviceSize,
    max_draw_count: u32,
    stride: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetCheckpointNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_checkpoint_marker: *const ffi::c_void,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetQueueCheckpointDataNV = unsafe extern "system" fn(
    queue: Queue,
    p_checkpoint_data_count: *mut u32,
    p_checkpoint_data: *mut CheckpointDataNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindTransformFeedbackBuffersEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_binding: u32,
    binding_count: u32,
    p_buffers: *const Buffer,
    p_offsets: *const DeviceSize,
    p_sizes: *const DeviceSize,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginTransformFeedbackEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_counter_buffer: u32,
    counter_buffer_count: u32,
    p_counter_buffers: *const Buffer,
    p_counter_buffer_offsets: *const DeviceSize,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndTransformFeedbackEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_counter_buffer: u32,
    counter_buffer_count: u32,
    p_counter_buffers: *const Buffer,
    p_counter_buffer_offsets: *const DeviceSize,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginQueryIndexedEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    query_pool: QueryPool,
    query: u32,
    flags: QueryControlFlags,
    index: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndQueryIndexedEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    query_pool: QueryPool,
    query: u32,
    index: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndirectByteCountEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    instance_count: u32,
    first_instance: u32,
    counter_buffer: Buffer,
    counter_buffer_offset: DeviceSize,
    counter_offset: u32,
    vertex_stride: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetExclusiveScissorNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_exclusive_scissor: u32,
    exclusive_scissor_count: u32,
    p_exclusive_scissors: *const Rect2D,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetExclusiveScissorEnableNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_exclusive_scissor: u32,
    exclusive_scissor_count: u32,
    p_exclusive_scissor_enables: *const Bool32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindShadingRateImageNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    image_view: ImageView,
    image_layout: ImageLayout,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetViewportShadingRatePaletteNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_viewport: u32,
    viewport_count: u32,
    p_shading_rate_palettes: *const ShadingRatePaletteNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetCoarseSampleOrderNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    sample_order_type: CoarseSampleOrderTypeNV,
    custom_sample_order_count: u32,
    p_custom_sample_orders: *const CoarseSampleOrderCustomNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawMeshTasksNV =
    unsafe extern "system" fn(command_buffer: CommandBuffer, task_count: u32, first_task: u32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawMeshTasksIndirectNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    draw_count: u32,
    stride: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawMeshTasksIndirectCountNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    count_buffer: Buffer,
    count_buffer_offset: DeviceSize,
    max_draw_count: u32,
    stride: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawMeshTasksEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    group_count_x: u32,
    group_count_y: u32,
    group_count_z: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawMeshTasksIndirectEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    draw_count: u32,
    stride: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawMeshTasksIndirectCountEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    count_buffer: Buffer,
    count_buffer_offset: DeviceSize,
    max_draw_count: u32,
    stride: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCompileDeferredNV =
    unsafe extern "system" fn(device: Device, pipeline: Pipeline, shader: u32) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateAccelerationStructureNV = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const AccelerationStructureCreateInfoNV,
    p_allocator: *const AllocationCallbacks,
    p_acceleration_structure: *mut AccelerationStructureNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindInvocationMaskHUAWEI = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    image_view: ImageView,
    image_layout: ImageLayout,
);
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyAccelerationStructureKHR = unsafe extern "system" fn(
    device: Device,
    acceleration_structure: AccelerationStructureKHR,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyAccelerationStructureNV = unsafe extern "system" fn(
    device: Device,
    acceleration_structure: AccelerationStructureNV,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetAccelerationStructureMemoryRequirementsNV = unsafe extern "system" fn(
    device: Device,
    p_info: *const AccelerationStructureMemoryRequirementsInfoNV,
    p_memory_requirements: *mut MemoryRequirements2,
);
#[allow(non_camel_case_types)]
pub type PFN_vkBindAccelerationStructureMemoryNV = unsafe extern "system" fn(
    device: Device,
    bind_info_count: u32,
    p_bind_infos: *const BindAccelerationStructureMemoryInfoNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyAccelerationStructureNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    dst: AccelerationStructureNV,
    src: AccelerationStructureNV,
    mode: CopyAccelerationStructureModeKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyAccelerationStructureKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_info: *const CopyAccelerationStructureInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCopyAccelerationStructureKHR = unsafe extern "system" fn(
    device: Device,
    deferred_operation: DeferredOperationKHR,
    p_info: *const CopyAccelerationStructureInfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyAccelerationStructureToMemoryKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_info: *const CopyAccelerationStructureToMemoryInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCopyAccelerationStructureToMemoryKHR = unsafe extern "system" fn(
    device: Device,
    deferred_operation: DeferredOperationKHR,
    p_info: *const CopyAccelerationStructureToMemoryInfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyMemoryToAccelerationStructureKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_info: *const CopyMemoryToAccelerationStructureInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCopyMemoryToAccelerationStructureKHR = unsafe extern "system" fn(
    device: Device,
    deferred_operation: DeferredOperationKHR,
    p_info: *const CopyMemoryToAccelerationStructureInfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdWriteAccelerationStructuresPropertiesKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    acceleration_structure_count: u32,
    p_acceleration_structures: *const AccelerationStructureKHR,
    query_type: QueryType,
    query_pool: QueryPool,
    first_query: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdWriteAccelerationStructuresPropertiesNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    acceleration_structure_count: u32,
    p_acceleration_structures: *const AccelerationStructureNV,
    query_type: QueryType,
    query_pool: QueryPool,
    first_query: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBuildAccelerationStructureNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_info: *const AccelerationStructureInfoNV,
    instance_data: Buffer,
    instance_offset: DeviceSize,
    update: Bool32,
    dst: AccelerationStructureNV,
    src: AccelerationStructureNV,
    scratch: Buffer,
    scratch_offset: DeviceSize,
);
#[allow(non_camel_case_types)]
pub type PFN_vkWriteAccelerationStructuresPropertiesKHR = unsafe extern "system" fn(
    device: Device,
    acceleration_structure_count: u32,
    p_acceleration_structures: *const AccelerationStructureKHR,
    query_type: QueryType,
    data_size: usize,
    p_data: *mut ffi::c_void,
    stride: usize,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdTraceRaysKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_raygen_shader_binding_table: *const StridedDeviceAddressRegionKHR,
    p_miss_shader_binding_table: *const StridedDeviceAddressRegionKHR,
    p_hit_shader_binding_table: *const StridedDeviceAddressRegionKHR,
    p_callable_shader_binding_table: *const StridedDeviceAddressRegionKHR,
    width: u32,
    height: u32,
    depth: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdTraceRaysNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    raygen_shader_binding_table_buffer: Buffer,
    raygen_shader_binding_offset: DeviceSize,
    miss_shader_binding_table_buffer: Buffer,
    miss_shader_binding_offset: DeviceSize,
    miss_shader_binding_stride: DeviceSize,
    hit_shader_binding_table_buffer: Buffer,
    hit_shader_binding_offset: DeviceSize,
    hit_shader_binding_stride: DeviceSize,
    callable_shader_binding_table_buffer: Buffer,
    callable_shader_binding_offset: DeviceSize,
    callable_shader_binding_stride: DeviceSize,
    width: u32,
    height: u32,
    depth: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetRayTracingShaderGroupHandlesKHR = unsafe extern "system" fn(
    device: Device,
    pipeline: Pipeline,
    first_group: u32,
    group_count: u32,
    data_size: usize,
    p_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetRayTracingCaptureReplayShaderGroupHandlesKHR =
    unsafe extern "system" fn(
        device: Device,
        pipeline: Pipeline,
        first_group: u32,
        group_count: u32,
        data_size: usize,
        p_data: *mut ffi::c_void,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetAccelerationStructureHandleNV = unsafe extern "system" fn(
    device: Device,
    acceleration_structure: AccelerationStructureNV,
    data_size: usize,
    p_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateRayTracingPipelinesNV = unsafe extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    create_info_count: u32,
    p_create_infos: *const RayTracingPipelineCreateInfoNV,
    p_allocator: *const AllocationCallbacks,
    p_pipelines: *mut Pipeline,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateRayTracingPipelinesKHR = unsafe extern "system" fn(
    device: Device,
    deferred_operation: DeferredOperationKHR,
    pipeline_cache: PipelineCache,
    create_info_count: u32,
    p_create_infos: *const RayTracingPipelineCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_pipelines: *mut Pipeline,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceCooperativeMatrixPropertiesNV = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_property_count: *mut u32,
    p_properties: *mut CooperativeMatrixPropertiesNV,
)
    -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdTraceRaysIndirectKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_raygen_shader_binding_table: *const StridedDeviceAddressRegionKHR,
    p_miss_shader_binding_table: *const StridedDeviceAddressRegionKHR,
    p_hit_shader_binding_table: *const StridedDeviceAddressRegionKHR,
    p_callable_shader_binding_table: *const StridedDeviceAddressRegionKHR,
    indirect_device_address: DeviceAddress,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdTraceRaysIndirect2KHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    indirect_device_address: DeviceAddress,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetClusterAccelerationStructureBuildSizesNV = unsafe extern "system" fn(
    device: Device,
    p_info: *const ClusterAccelerationStructureInputInfoNV,
    p_size_info: *mut AccelerationStructureBuildSizesInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBuildClusterAccelerationStructureIndirectNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_command_infos: *const ClusterAccelerationStructureCommandsInfoNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceAccelerationStructureCompatibilityKHR = unsafe extern "system" fn(
    device: Device,
    p_version_info: *const AccelerationStructureVersionInfoKHR,
    p_compatibility: *mut AccelerationStructureCompatibilityKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetRayTracingShaderGroupStackSizeKHR = unsafe extern "system" fn(
    device: Device,
    pipeline: Pipeline,
    group: u32,
    group_shader: ShaderGroupShaderKHR,
) -> DeviceSize;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetRayTracingPipelineStackSizeKHR =
    unsafe extern "system" fn(command_buffer: CommandBuffer, pipeline_stack_size: u32);
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageViewHandleNVX =
    unsafe extern "system" fn(device: Device, p_info: *const ImageViewHandleInfoNVX) -> u32;
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageViewHandle64NVX =
    unsafe extern "system" fn(device: Device, p_info: *const ImageViewHandleInfoNVX) -> u64;
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageViewAddressNVX = unsafe extern "system" fn(
    device: Device,
    image_view: ImageView,
    p_properties: *mut ImageViewAddressPropertiesNVX,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceCombinedImageSamplerIndexNVX =
    unsafe extern "system" fn(device: Device, image_view_index: u64, sampler_index: u64) -> u64;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSurfacePresentModes2EXT = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_surface_info: *const PhysicalDeviceSurfaceInfo2KHR,
    p_present_mode_count: *mut u32,
    p_present_modes: *mut PresentModeKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceGroupSurfacePresentModes2EXT = unsafe extern "system" fn(
    device: Device,
    p_surface_info: *const PhysicalDeviceSurfaceInfo2KHR,
    p_modes: *mut DeviceGroupPresentModeFlagsKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkAcquireFullScreenExclusiveModeEXT =
    unsafe extern "system" fn(device: Device, swapchain: SwapchainKHR) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkReleaseFullScreenExclusiveModeEXT =
    unsafe extern "system" fn(device: Device, swapchain: SwapchainKHR) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkEnumeratePhysicalDeviceQueueFamilyPerformanceQueryCountersKHR =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        queue_family_index: u32,
        p_counter_count: *mut u32,
        p_counters: *mut PerformanceCounterKHR,
        p_counter_descriptions: *mut PerformanceCounterDescriptionKHR,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceQueueFamilyPerformanceQueryPassesKHR =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        p_performance_query_create_info: *const QueryPoolPerformanceCreateInfoKHR,
        p_num_passes: *mut u32,
    );
#[allow(non_camel_case_types)]
pub type PFN_vkAcquireProfilingLockKHR =
    unsafe extern "system" fn(device: Device, p_info: *const AcquireProfilingLockInfoKHR) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkReleaseProfilingLockKHR = unsafe extern "system" fn(device: Device);
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageDrmFormatModifierPropertiesEXT = unsafe extern "system" fn(
    device: Device,
    image: Image,
    p_properties: *mut ImageDrmFormatModifierPropertiesEXT,
) -> Result;
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetBufferOpaqueCaptureAddress =
    unsafe extern "system" fn(device: Device, p_info: *const BufferDeviceAddressInfo) -> u64;
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetBufferDeviceAddress = unsafe extern "system" fn(
    device: Device,
    p_info: *const BufferDeviceAddressInfo,
) -> DeviceAddress;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateHeadlessSurfaceEXT = unsafe extern "system" fn(
    instance: Instance,
    p_create_info: *const HeadlessSurfaceCreateInfoEXT,
    p_allocator: *const AllocationCallbacks,
    p_surface: *mut SurfaceKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSupportedFramebufferMixedSamplesCombinationsNV =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        p_combination_count: *mut u32,
        p_combinations: *mut FramebufferMixedSamplesCombinationNV,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkInitializePerformanceApiINTEL = unsafe extern "system" fn(
    device: Device,
    p_initialize_info: *const InitializePerformanceApiInfoINTEL,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkUninitializePerformanceApiINTEL = unsafe extern "system" fn(device: Device);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetPerformanceMarkerINTEL = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_marker_info: *const PerformanceMarkerInfoINTEL,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetPerformanceStreamMarkerINTEL = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_marker_info: *const PerformanceStreamMarkerInfoINTEL,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetPerformanceOverrideINTEL = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_override_info: *const PerformanceOverrideInfoINTEL,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkAcquirePerformanceConfigurationINTEL = unsafe extern "system" fn(
    device: Device,
    p_acquire_info: *const PerformanceConfigurationAcquireInfoINTEL,
    p_configuration: *mut PerformanceConfigurationINTEL,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkReleasePerformanceConfigurationINTEL = unsafe extern "system" fn(
    device: Device,
    configuration: PerformanceConfigurationINTEL,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkQueueSetPerformanceConfigurationINTEL =
    unsafe extern "system" fn(queue: Queue, configuration: PerformanceConfigurationINTEL) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPerformanceParameterINTEL = unsafe extern "system" fn(
    device: Device,
    parameter: PerformanceParameterTypeINTEL,
    p_value: *mut PerformanceValueINTEL,
) -> Result;
#[doc = "Provided by Vulkan version 1.2"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceMemoryOpaqueCaptureAddress = unsafe extern "system" fn(
    device: Device,
    p_info: *const DeviceMemoryOpaqueCaptureAddressInfo,
) -> u64;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPipelineExecutablePropertiesKHR = unsafe extern "system" fn(
    device: Device,
    p_pipeline_info: *const PipelineInfoKHR,
    p_executable_count: *mut u32,
    p_properties: *mut PipelineExecutablePropertiesKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPipelineExecutableStatisticsKHR = unsafe extern "system" fn(
    device: Device,
    p_executable_info: *const PipelineExecutableInfoKHR,
    p_statistic_count: *mut u32,
    p_statistics: *mut PipelineExecutableStatisticKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPipelineExecutableInternalRepresentationsKHR =
    unsafe extern "system" fn(
        device: Device,
        p_executable_info: *const PipelineExecutableInfoKHR,
        p_internal_representation_count: *mut u32,
        p_internal_representations: *mut PipelineExecutableInternalRepresentationKHR,
    ) -> Result;
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetLineStipple = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    line_stipple_factor: u32,
    line_stipple_pattern: u16,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceToolProperties = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_tool_count: *mut u32,
    p_tool_properties: *mut PhysicalDeviceToolProperties,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateAccelerationStructureKHR = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const AccelerationStructureCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_acceleration_structure: *mut AccelerationStructureKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBuildAccelerationStructuresKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    info_count: u32,
    p_infos: *const AccelerationStructureBuildGeometryInfoKHR,
    pp_build_range_infos: *const *const AccelerationStructureBuildRangeInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBuildAccelerationStructuresIndirectKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    info_count: u32,
    p_infos: *const AccelerationStructureBuildGeometryInfoKHR,
    p_indirect_device_addresses: *const DeviceAddress,
    p_indirect_strides: *const u32,
    pp_max_primitive_counts: *const *const u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkBuildAccelerationStructuresKHR = unsafe extern "system" fn(
    device: Device,
    deferred_operation: DeferredOperationKHR,
    info_count: u32,
    p_infos: *const AccelerationStructureBuildGeometryInfoKHR,
    pp_build_range_infos: *const *const AccelerationStructureBuildRangeInfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetAccelerationStructureDeviceAddressKHR =
    unsafe extern "system" fn(
        device: Device,
        p_info: *const AccelerationStructureDeviceAddressInfoKHR,
    ) -> DeviceAddress;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDeferredOperationKHR = unsafe extern "system" fn(
    device: Device,
    p_allocator: *const AllocationCallbacks,
    p_deferred_operation: *mut DeferredOperationKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyDeferredOperationKHR = unsafe extern "system" fn(
    device: Device,
    operation: DeferredOperationKHR,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeferredOperationMaxConcurrencyKHR =
    unsafe extern "system" fn(device: Device, operation: DeferredOperationKHR) -> u32;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeferredOperationResultKHR =
    unsafe extern "system" fn(device: Device, operation: DeferredOperationKHR) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDeferredOperationJoinKHR =
    unsafe extern "system" fn(device: Device, operation: DeferredOperationKHR) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPipelineIndirectMemoryRequirementsNV = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const ComputePipelineCreateInfo,
    p_memory_requirements: *mut MemoryRequirements2,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPipelineIndirectDeviceAddressNV = unsafe extern "system" fn(
    device: Device,
    p_info: *const PipelineIndirectDeviceAddressInfoNV,
) -> DeviceAddress;
#[allow(non_camel_case_types)]
pub type PFN_vkAntiLagUpdateAMD =
    unsafe extern "system" fn(device: Device, p_data: *const AntiLagDataAMD);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetCullMode =
    unsafe extern "system" fn(command_buffer: CommandBuffer, cull_mode: CullModeFlags);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetFrontFace =
    unsafe extern "system" fn(command_buffer: CommandBuffer, front_face: FrontFace);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetPrimitiveTopology =
    unsafe extern "system" fn(command_buffer: CommandBuffer, primitive_topology: PrimitiveTopology);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetViewportWithCount = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    viewport_count: u32,
    p_viewports: *const Viewport,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetScissorWithCount = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    scissor_count: u32,
    p_scissors: *const Rect2D,
);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindIndexBuffer2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer: Buffer,
    offset: DeviceSize,
    size: DeviceSize,
    index_type: IndexType,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindVertexBuffers2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_binding: u32,
    binding_count: u32,
    p_buffers: *const Buffer,
    p_offsets: *const DeviceSize,
    p_sizes: *const DeviceSize,
    p_strides: *const DeviceSize,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthTestEnable =
    unsafe extern "system" fn(command_buffer: CommandBuffer, depth_test_enable: Bool32);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthWriteEnable =
    unsafe extern "system" fn(command_buffer: CommandBuffer, depth_write_enable: Bool32);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthCompareOp =
    unsafe extern "system" fn(command_buffer: CommandBuffer, depth_compare_op: CompareOp);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthBoundsTestEnable =
    unsafe extern "system" fn(command_buffer: CommandBuffer, depth_bounds_test_enable: Bool32);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetStencilTestEnable =
    unsafe extern "system" fn(command_buffer: CommandBuffer, stencil_test_enable: Bool32);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetStencilOp = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    face_mask: StencilFaceFlags,
    fail_op: StencilOp,
    pass_op: StencilOp,
    depth_fail_op: StencilOp,
    compare_op: CompareOp,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetPatchControlPointsEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, patch_control_points: u32);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetRasterizerDiscardEnable =
    unsafe extern "system" fn(command_buffer: CommandBuffer, rasterizer_discard_enable: Bool32);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthBiasEnable =
    unsafe extern "system" fn(command_buffer: CommandBuffer, depth_bias_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetLogicOpEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, logic_op: LogicOp);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetPrimitiveRestartEnable =
    unsafe extern "system" fn(command_buffer: CommandBuffer, primitive_restart_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetTessellationDomainOriginEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    domain_origin: TessellationDomainOrigin,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthClampEnableEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, depth_clamp_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetPolygonModeEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, polygon_mode: PolygonMode);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetRasterizationSamplesEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    rasterization_samples: SampleCountFlags,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetSampleMaskEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    samples: SampleCountFlags,
    p_sample_mask: *const SampleMask,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetAlphaToCoverageEnableEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, alpha_to_coverage_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetAlphaToOneEnableEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, alpha_to_one_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetLogicOpEnableEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, logic_op_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetColorBlendEnableEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_attachment: u32,
    attachment_count: u32,
    p_color_blend_enables: *const Bool32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetColorBlendEquationEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_attachment: u32,
    attachment_count: u32,
    p_color_blend_equations: *const ColorBlendEquationEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetColorWriteMaskEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_attachment: u32,
    attachment_count: u32,
    p_color_write_masks: *const ColorComponentFlags,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetRasterizationStreamEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, rasterization_stream: u32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetConservativeRasterizationModeEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    conservative_rasterization_mode: ConservativeRasterizationModeEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetExtraPrimitiveOverestimationSizeEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    extra_primitive_overestimation_size: f32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthClipEnableEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, depth_clip_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetSampleLocationsEnableEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, sample_locations_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetColorBlendAdvancedEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_attachment: u32,
    attachment_count: u32,
    p_color_blend_advanced: *const ColorBlendAdvancedEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetProvokingVertexModeEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    provoking_vertex_mode: ProvokingVertexModeEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetLineRasterizationModeEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    line_rasterization_mode: LineRasterizationModeEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetLineStippleEnableEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, stippled_line_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthClipNegativeOneToOneEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, negative_one_to_one: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetViewportWScalingEnableNV =
    unsafe extern "system" fn(command_buffer: CommandBuffer, viewport_wscaling_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetViewportSwizzleNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_viewport: u32,
    viewport_count: u32,
    p_viewport_swizzles: *const ViewportSwizzleNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetCoverageToColorEnableNV =
    unsafe extern "system" fn(command_buffer: CommandBuffer, coverage_to_color_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetCoverageToColorLocationNV =
    unsafe extern "system" fn(command_buffer: CommandBuffer, coverage_to_color_location: u32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetCoverageModulationModeNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    coverage_modulation_mode: CoverageModulationModeNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetCoverageModulationTableEnableNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    coverage_modulation_table_enable: Bool32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetCoverageModulationTableNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    coverage_modulation_table_count: u32,
    p_coverage_modulation_table: *const f32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetShadingRateImageEnableNV =
    unsafe extern "system" fn(command_buffer: CommandBuffer, shading_rate_image_enable: Bool32);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetCoverageReductionModeNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    coverage_reduction_mode: CoverageReductionModeNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetRepresentativeFragmentTestEnableNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    representative_fragment_test_enable: Bool32,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreatePrivateDataSlot = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const PrivateDataSlotCreateInfo,
    p_allocator: *const AllocationCallbacks,
    p_private_data_slot: *mut PrivateDataSlot,
) -> Result;
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyPrivateDataSlot = unsafe extern "system" fn(
    device: Device,
    private_data_slot: PrivateDataSlot,
    p_allocator: *const AllocationCallbacks,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkSetPrivateData = unsafe extern "system" fn(
    device: Device,
    object_type: ObjectType,
    object_handle: u64,
    private_data_slot: PrivateDataSlot,
    data: u64,
) -> Result;
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPrivateData = unsafe extern "system" fn(
    device: Device,
    object_type: ObjectType,
    object_handle: u64,
    private_data_slot: PrivateDataSlot,
    p_data: *mut u64,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyBuffer2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_copy_buffer_info: *const CopyBufferInfo2,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyImage2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_copy_image_info: *const CopyImageInfo2,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBlitImage2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_blit_image_info: *const BlitImageInfo2,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyBufferToImage2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_copy_buffer_to_image_info: *const CopyBufferToImageInfo2,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyImageToBuffer2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_copy_image_to_buffer_info: *const CopyImageToBufferInfo2,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdResolveImage2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_resolve_image_info: *const ResolveImageInfo2,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdRefreshObjectsKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_refresh_objects: *const RefreshObjectListKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceRefreshableObjectTypesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_refreshable_object_type_count: *mut u32,
    p_refreshable_object_types: *mut ObjectType,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetFragmentShadingRateKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_fragment_size: *const Extent2D,
    combiner_ops: [FragmentShadingRateCombinerOpKHR; 2usize],
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceFragmentShadingRatesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_fragment_shading_rate_count: *mut u32,
    p_fragment_shading_rates: *mut PhysicalDeviceFragmentShadingRateKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetFragmentShadingRateEnumNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    shading_rate: FragmentShadingRateNV,
    combiner_ops: [FragmentShadingRateCombinerOpKHR; 2usize],
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetAccelerationStructureBuildSizesKHR = unsafe extern "system" fn(
    device: Device,
    build_type: AccelerationStructureBuildTypeKHR,
    p_build_info: *const AccelerationStructureBuildGeometryInfoKHR,
    p_max_primitive_counts: *const u32,
    p_size_info: *mut AccelerationStructureBuildSizesInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetVertexInputEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    vertex_binding_description_count: u32,
    p_vertex_binding_descriptions: *const VertexInputBindingDescription2EXT,
    vertex_attribute_description_count: u32,
    p_vertex_attribute_descriptions: *const VertexInputAttributeDescription2EXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetColorWriteEnableEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    attachment_count: u32,
    p_color_write_enables: *const Bool32,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetEvent2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    event: Event,
    p_dependency_info: *const DependencyInfo,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdResetEvent2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    event: Event,
    stage_mask: PipelineStageFlags2,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdWaitEvents2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    event_count: u32,
    p_events: *const Event,
    p_dependency_infos: *const DependencyInfo,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPipelineBarrier2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_dependency_info: *const DependencyInfo,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkQueueSubmit2 = unsafe extern "system" fn(
    queue: Queue,
    submit_count: u32,
    p_submits: *const SubmitInfo2,
    fence: Fence,
) -> Result;
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdWriteTimestamp2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    stage: PipelineStageFlags2,
    query_pool: QueryPool,
    query: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdWriteBufferMarker2AMD = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    stage: PipelineStageFlags2,
    dst_buffer: Buffer,
    dst_offset: DeviceSize,
    marker: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetQueueCheckpointData2NV = unsafe extern "system" fn(
    queue: Queue,
    p_checkpoint_data_count: *mut u32,
    p_checkpoint_data: *mut CheckpointData2NV,
);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCopyMemoryToImage = unsafe extern "system" fn(
    device: Device,
    p_copy_memory_to_image_info: *const CopyMemoryToImageInfo,
) -> Result;
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCopyImageToMemory = unsafe extern "system" fn(
    device: Device,
    p_copy_image_to_memory_info: *const CopyImageToMemoryInfo,
) -> Result;
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCopyImageToImage = unsafe extern "system" fn(
    device: Device,
    p_copy_image_to_image_info: *const CopyImageToImageInfo,
) -> Result;
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkTransitionImageLayout = unsafe extern "system" fn(
    device: Device,
    transition_count: u32,
    p_transitions: *const HostImageLayoutTransitionInfo,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceVideoCapabilitiesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_video_profile: *const VideoProfileInfoKHR,
    p_capabilities: *mut VideoCapabilitiesKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceVideoFormatPropertiesKHR = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_video_format_info: *const PhysicalDeviceVideoFormatInfoKHR,
    p_video_format_property_count: *mut u32,
    p_video_format_properties: *mut VideoFormatPropertiesKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceVideoEncodeQualityLevelPropertiesKHR =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        p_quality_level_info: *const PhysicalDeviceVideoEncodeQualityLevelInfoKHR,
        p_quality_level_properties: *mut VideoEncodeQualityLevelPropertiesKHR,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateVideoSessionKHR = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const VideoSessionCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_video_session: *mut VideoSessionKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyVideoSessionKHR = unsafe extern "system" fn(
    device: Device,
    video_session: VideoSessionKHR,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateVideoSessionParametersKHR = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const VideoSessionParametersCreateInfoKHR,
    p_allocator: *const AllocationCallbacks,
    p_video_session_parameters: *mut VideoSessionParametersKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkUpdateVideoSessionParametersKHR = unsafe extern "system" fn(
    device: Device,
    video_session_parameters: VideoSessionParametersKHR,
    p_update_info: *const VideoSessionParametersUpdateInfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetEncodedVideoSessionParametersKHR = unsafe extern "system" fn(
    device: Device,
    p_video_session_parameters_info: *const VideoEncodeSessionParametersGetInfoKHR,
    p_feedback_info: *mut VideoEncodeSessionParametersFeedbackInfoKHR,
    p_data_size: *mut usize,
    p_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyVideoSessionParametersKHR = unsafe extern "system" fn(
    device: Device,
    video_session_parameters: VideoSessionParametersKHR,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetVideoSessionMemoryRequirementsKHR = unsafe extern "system" fn(
    device: Device,
    video_session: VideoSessionKHR,
    p_memory_requirements_count: *mut u32,
    p_memory_requirements: *mut VideoSessionMemoryRequirementsKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkBindVideoSessionMemoryKHR = unsafe extern "system" fn(
    device: Device,
    video_session: VideoSessionKHR,
    bind_session_memory_info_count: u32,
    p_bind_session_memory_infos: *const BindVideoSessionMemoryInfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDecodeVideoKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_decode_info: *const VideoDecodeInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginVideoCodingKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_begin_info: *const VideoBeginCodingInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdControlVideoCodingKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_coding_control_info: *const VideoCodingControlInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndVideoCodingKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_end_coding_info: *const VideoEndCodingInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEncodeVideoKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_encode_info: *const VideoEncodeInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDecompressMemoryNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    decompress_region_count: u32,
    p_decompress_memory_regions: *const DecompressMemoryRegionNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDecompressMemoryIndirectCountNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    indirect_commands_address: DeviceAddress,
    indirect_commands_count_address: DeviceAddress,
    stride: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPartitionedAccelerationStructuresBuildSizesNV = unsafe extern "system" fn(
    device: Device,
    p_info: *const PartitionedAccelerationStructureInstancesInputNV,
    p_size_info: *mut AccelerationStructureBuildSizesInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBuildPartitionedAccelerationStructuresNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_build_info: *const BuildPartitionedAccelerationStructureInfoNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDecompressMemoryEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_decompress_memory_info_ext: *const DecompressMemoryInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDecompressMemoryIndirectCountEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    decompression_method: MemoryDecompressionMethodFlagsEXT,
    indirect_commands_address: DeviceAddress,
    indirect_commands_count_address: DeviceAddress,
    max_decompression_count: u32,
    stride: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateCuModuleNVX = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const CuModuleCreateInfoNVX,
    p_allocator: *const AllocationCallbacks,
    p_module: *mut CuModuleNVX,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateCuFunctionNVX = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const CuFunctionCreateInfoNVX,
    p_allocator: *const AllocationCallbacks,
    p_function: *mut CuFunctionNVX,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyCuModuleNVX = unsafe extern "system" fn(
    device: Device,
    module: CuModuleNVX,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyCuFunctionNVX = unsafe extern "system" fn(
    device: Device,
    function: CuFunctionNVX,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCuLaunchKernelNVX =
    unsafe extern "system" fn(command_buffer: CommandBuffer, p_launch_info: *const CuLaunchInfoNVX);
#[allow(non_camel_case_types)]
pub type PFN_vkGetDescriptorSetLayoutSizeEXT = unsafe extern "system" fn(
    device: Device,
    layout: DescriptorSetLayout,
    p_layout_size_in_bytes: *mut DeviceSize,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetDescriptorSetLayoutBindingOffsetEXT = unsafe extern "system" fn(
    device: Device,
    layout: DescriptorSetLayout,
    binding: u32,
    p_offset: *mut DeviceSize,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetDescriptorEXT = unsafe extern "system" fn(
    device: Device,
    p_descriptor_info: *const DescriptorGetInfoEXT,
    data_size: usize,
    p_descriptor: *mut ffi::c_void,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindDescriptorBuffersEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    buffer_count: u32,
    p_binding_infos: *const DescriptorBufferBindingInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDescriptorBufferOffsetsEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_bind_point: PipelineBindPoint,
    layout: PipelineLayout,
    first_set: u32,
    set_count: u32,
    p_buffer_indices: *const u32,
    p_offsets: *const DeviceSize,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindDescriptorBufferEmbeddedSamplersEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    pipeline_bind_point: PipelineBindPoint,
    layout: PipelineLayout,
    set: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetBufferOpaqueCaptureDescriptorDataEXT = unsafe extern "system" fn(
    device: Device,
    p_info: *const BufferCaptureDescriptorDataInfoEXT,
    p_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageOpaqueCaptureDescriptorDataEXT = unsafe extern "system" fn(
    device: Device,
    p_info: *const ImageCaptureDescriptorDataInfoEXT,
    p_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageViewOpaqueCaptureDescriptorDataEXT = unsafe extern "system" fn(
    device: Device,
    p_info: *const ImageViewCaptureDescriptorDataInfoEXT,
    p_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetSamplerOpaqueCaptureDescriptorDataEXT = unsafe extern "system" fn(
    device: Device,
    p_info: *const SamplerCaptureDescriptorDataInfoEXT,
    p_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetAccelerationStructureOpaqueCaptureDescriptorDataEXT =
    unsafe extern "system" fn(
        device: Device,
        p_info: *const AccelerationStructureCaptureDescriptorDataInfoEXT,
        p_data: *mut ffi::c_void,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkSetDeviceMemoryPriorityEXT =
    unsafe extern "system" fn(device: Device, memory: DeviceMemory, priority: f32);
#[allow(non_camel_case_types)]
pub type PFN_vkAcquireDrmDisplayEXT = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    drm_fd: i32,
    display: DisplayKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDrmDisplayEXT = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    drm_fd: i32,
    connector_id: u32,
    display: *mut DisplayKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkWaitForPresent2KHR = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_present_wait2_info: *const PresentWait2InfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkWaitForPresentKHR = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    present_id: u64,
    timeout: u64,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateBufferCollectionFUCHSIA = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const BufferCollectionCreateInfoFUCHSIA,
    p_allocator: *const AllocationCallbacks,
    p_collection: *mut BufferCollectionFUCHSIA,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkSetBufferCollectionBufferConstraintsFUCHSIA = unsafe extern "system" fn(
    device: Device,
    collection: BufferCollectionFUCHSIA,
    p_buffer_constraints_info: *const BufferConstraintsInfoFUCHSIA,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkSetBufferCollectionImageConstraintsFUCHSIA = unsafe extern "system" fn(
    device: Device,
    collection: BufferCollectionFUCHSIA,
    p_image_constraints_info: *const ImageConstraintsInfoFUCHSIA,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyBufferCollectionFUCHSIA = unsafe extern "system" fn(
    device: Device,
    collection: BufferCollectionFUCHSIA,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetBufferCollectionPropertiesFUCHSIA = unsafe extern "system" fn(
    device: Device,
    collection: BufferCollectionFUCHSIA,
    p_properties: *mut BufferCollectionPropertiesFUCHSIA,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateCudaModuleNV = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const CudaModuleCreateInfoNV,
    p_allocator: *const AllocationCallbacks,
    p_module: *mut CudaModuleNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetCudaModuleCacheNV = unsafe extern "system" fn(
    device: Device,
    module: CudaModuleNV,
    p_cache_size: *mut usize,
    p_cache_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateCudaFunctionNV = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const CudaFunctionCreateInfoNV,
    p_allocator: *const AllocationCallbacks,
    p_function: *mut CudaFunctionNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyCudaModuleNV = unsafe extern "system" fn(
    device: Device,
    module: CudaModuleNV,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyCudaFunctionNV = unsafe extern "system" fn(
    device: Device,
    function: CudaFunctionNV,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCudaLaunchKernelNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_launch_info: *const CudaLaunchInfoNV,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginRendering = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_rendering_info: *const RenderingInfo,
);
#[doc = "Provided by Vulkan version 1.3"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndRendering = unsafe extern "system" fn(command_buffer: CommandBuffer);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndRendering2KHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_rendering_end_info: *const RenderingEndInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetDescriptorSetLayoutHostMappingInfoVALVE = unsafe extern "system" fn(
    device: Device,
    p_binding_reference: *const DescriptorSetBindingReferenceVALVE,
    p_host_mapping: *mut DescriptorSetLayoutHostMappingInfoVALVE,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetDescriptorSetHostMappingVALVE = unsafe extern "system" fn(
    device: Device,
    descriptor_set: DescriptorSet,
    pp_data: *mut *mut ffi::c_void,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateMicromapEXT = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const MicromapCreateInfoEXT,
    p_allocator: *const AllocationCallbacks,
    p_micromap: *mut MicromapEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBuildMicromapsEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    info_count: u32,
    p_infos: *const MicromapBuildInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkBuildMicromapsEXT = unsafe extern "system" fn(
    device: Device,
    deferred_operation: DeferredOperationKHR,
    info_count: u32,
    p_infos: *const MicromapBuildInfoEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyMicromapEXT = unsafe extern "system" fn(
    device: Device,
    micromap: MicromapEXT,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyMicromapEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, p_info: *const CopyMicromapInfoEXT);
#[allow(non_camel_case_types)]
pub type PFN_vkCopyMicromapEXT = unsafe extern "system" fn(
    device: Device,
    deferred_operation: DeferredOperationKHR,
    p_info: *const CopyMicromapInfoEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyMicromapToMemoryEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_info: *const CopyMicromapToMemoryInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCopyMicromapToMemoryEXT = unsafe extern "system" fn(
    device: Device,
    deferred_operation: DeferredOperationKHR,
    p_info: *const CopyMicromapToMemoryInfoEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyMemoryToMicromapEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_info: *const CopyMemoryToMicromapInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCopyMemoryToMicromapEXT = unsafe extern "system" fn(
    device: Device,
    deferred_operation: DeferredOperationKHR,
    p_info: *const CopyMemoryToMicromapInfoEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdWriteMicromapsPropertiesEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    micromap_count: u32,
    p_micromaps: *const MicromapEXT,
    query_type: QueryType,
    query_pool: QueryPool,
    first_query: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkWriteMicromapsPropertiesEXT = unsafe extern "system" fn(
    device: Device,
    micromap_count: u32,
    p_micromaps: *const MicromapEXT,
    query_type: QueryType,
    data_size: usize,
    p_data: *mut ffi::c_void,
    stride: usize,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceMicromapCompatibilityEXT = unsafe extern "system" fn(
    device: Device,
    p_version_info: *const MicromapVersionInfoEXT,
    p_compatibility: *mut AccelerationStructureCompatibilityKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetMicromapBuildSizesEXT = unsafe extern "system" fn(
    device: Device,
    build_type: AccelerationStructureBuildTypeKHR,
    p_build_info: *const MicromapBuildInfoEXT,
    p_size_info: *mut MicromapBuildSizesInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetShaderModuleIdentifierEXT = unsafe extern "system" fn(
    device: Device,
    shader_module: ShaderModule,
    p_identifier: *mut ShaderModuleIdentifierEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetShaderModuleCreateInfoIdentifierEXT = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const ShaderModuleCreateInfo,
    p_identifier: *mut ShaderModuleIdentifierEXT,
);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageSubresourceLayout2 = unsafe extern "system" fn(
    device: Device,
    image: Image,
    p_subresource: *const ImageSubresource2,
    p_layout: *mut SubresourceLayout2,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPipelinePropertiesEXT = unsafe extern "system" fn(
    device: Device,
    p_pipeline_info: *const PipelineInfoKHR,
    p_pipeline_properties: *mut BaseOutStructure,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkExportMetalObjectsEXT =
    unsafe extern "system" fn(device: Device, p_metal_objects_info: *mut ExportMetalObjectsInfoEXT);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindTileMemoryQCOM = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_tile_memory_bind_info: *const TileMemoryBindInfoQCOM,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetFramebufferTilePropertiesQCOM = unsafe extern "system" fn(
    device: Device,
    framebuffer: Framebuffer,
    p_properties_count: *mut u32,
    p_properties: *mut TilePropertiesQCOM,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDynamicRenderingTilePropertiesQCOM = unsafe extern "system" fn(
    device: Device,
    p_rendering_info: *const RenderingInfo,
    p_properties: *mut TilePropertiesQCOM,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceOpticalFlowImageFormatsNV = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_optical_flow_image_format_info: *const OpticalFlowImageFormatInfoNV,
    p_format_count: *mut u32,
    p_image_format_properties: *mut OpticalFlowImageFormatPropertiesNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateOpticalFlowSessionNV = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const OpticalFlowSessionCreateInfoNV,
    p_allocator: *const AllocationCallbacks,
    p_session: *mut OpticalFlowSessionNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyOpticalFlowSessionNV = unsafe extern "system" fn(
    device: Device,
    session: OpticalFlowSessionNV,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkBindOpticalFlowSessionImageNV = unsafe extern "system" fn(
    device: Device,
    session: OpticalFlowSessionNV,
    binding_point: OpticalFlowSessionBindingPointNV,
    view: ImageView,
    layout: ImageLayout,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdOpticalFlowExecuteNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    session: OpticalFlowSessionNV,
    p_execute_info: *const OpticalFlowExecuteInfoNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceFaultInfoEXT = unsafe extern "system" fn(
    device: Device,
    p_fault_counts: *mut DeviceFaultCountsEXT,
    p_fault_info: *mut DeviceFaultInfoEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceFaultReportsKHR = unsafe extern "system" fn(
    device: Device,
    timeout: u64,
    p_fault_counts: *mut u32,
    p_fault_info: *mut DeviceFaultInfoKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceFaultDebugInfoKHR =
    unsafe extern "system" fn(device: Device, p_debug_info: *mut DeviceFaultDebugInfoKHR) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthBias2EXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_depth_bias_info: *const DepthBiasInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkReleaseSwapchainImagesKHR = unsafe extern "system" fn(
    device: Device,
    p_release_info: *const ReleaseSwapchainImagesInfoKHR,
) -> Result;
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceImageSubresourceLayout = unsafe extern "system" fn(
    device: Device,
    p_info: *const DeviceImageSubresourceInfo,
    p_layout: *mut SubresourceLayout2,
);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkMapMemory2 = unsafe extern "system" fn(
    device: Device,
    p_memory_map_info: *const MemoryMapInfo,
    pp_data: *mut *mut ffi::c_void,
) -> Result;
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkUnmapMemory2 = unsafe extern "system" fn(
    device: Device,
    p_memory_unmap_info: *const MemoryUnmapInfo,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateShadersEXT = unsafe extern "system" fn(
    device: Device,
    create_info_count: u32,
    p_create_infos: *const ShaderCreateInfoEXT,
    p_allocator: *const AllocationCallbacks,
    p_shaders: *mut ShaderEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyShaderEXT = unsafe extern "system" fn(
    device: Device,
    shader: ShaderEXT,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetShaderBinaryDataEXT = unsafe extern "system" fn(
    device: Device,
    shader: ShaderEXT,
    p_data_size: *mut usize,
    p_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindShadersEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    stage_count: u32,
    p_stages: *const ShaderStageFlags,
    p_shaders: *const ShaderEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkSetSwapchainPresentTimingQueueSizeEXT =
    unsafe extern "system" fn(device: Device, swapchain: SwapchainKHR, size: u32) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetSwapchainTimingPropertiesEXT = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_swapchain_timing_properties: *mut SwapchainTimingPropertiesEXT,
    p_swapchain_timing_properties_counter: *mut u64,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetSwapchainTimeDomainPropertiesEXT = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_swapchain_time_domain_properties: *mut SwapchainTimeDomainPropertiesEXT,
    p_time_domains_counter: *mut u64,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPastPresentationTimingEXT = unsafe extern "system" fn(
    device: Device,
    p_past_presentation_timing_info: *const PastPresentationTimingInfoEXT,
    p_past_presentation_timing_properties: *mut PastPresentationTimingPropertiesEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetScreenBufferPropertiesQNX = unsafe extern "system" fn(
    device: Device,
    buffer: *const _screen_buffer,
    p_properties: *mut ScreenBufferPropertiesQNX,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceCooperativeMatrixPropertiesKHR =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        p_property_count: *mut u32,
        p_properties: *mut CooperativeMatrixPropertiesKHR,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetExecutionGraphPipelineScratchSizeAMDX = unsafe extern "system" fn(
    device: Device,
    execution_graph: Pipeline,
    p_size_info: *mut ExecutionGraphPipelineScratchSizeAMDX,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetExecutionGraphPipelineNodeIndexAMDX = unsafe extern "system" fn(
    device: Device,
    execution_graph: Pipeline,
    p_node_info: *const PipelineShaderStageNodeCreateInfoAMDX,
    p_node_index: *mut u32,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateExecutionGraphPipelinesAMDX = unsafe extern "system" fn(
    device: Device,
    pipeline_cache: PipelineCache,
    create_info_count: u32,
    p_create_infos: *const ExecutionGraphPipelineCreateInfoAMDX,
    p_allocator: *const AllocationCallbacks,
    p_pipelines: *mut Pipeline,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdInitializeGraphScratchMemoryAMDX = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    execution_graph: Pipeline,
    scratch: DeviceAddress,
    scratch_size: DeviceSize,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDispatchGraphAMDX = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    scratch: DeviceAddress,
    scratch_size: DeviceSize,
    p_count_info: *const DispatchGraphCountInfoAMDX,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDispatchGraphIndirectAMDX = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    scratch: DeviceAddress,
    scratch_size: DeviceSize,
    p_count_info: *const DispatchGraphCountInfoAMDX,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDispatchGraphIndirectCountAMDX = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    scratch: DeviceAddress,
    scratch_size: DeviceSize,
    count_info: DeviceAddress,
);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindDescriptorSets2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_bind_descriptor_sets_info: *const BindDescriptorSetsInfo,
);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPushConstants2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_push_constants_info: *const PushConstantsInfo,
);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPushDescriptorSet2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_push_descriptor_set_info: *const PushDescriptorSetInfo,
);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPushDescriptorSetWithTemplate2 = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_push_descriptor_set_with_template_info: *const PushDescriptorSetWithTemplateInfo,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDescriptorBufferOffsets2EXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_set_descriptor_buffer_offsets_info: *const SetDescriptorBufferOffsetsInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindDescriptorBufferEmbeddedSamplers2EXT = unsafe extern "system" fn (command_buffer : CommandBuffer , p_bind_descriptor_buffer_embedded_samplers_info : * const BindDescriptorBufferEmbeddedSamplersInfoEXT ,) ;
#[allow(non_camel_case_types)]
pub type PFN_vkSetLatencySleepModeNV = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_sleep_mode_info: *const LatencySleepModeInfoNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkLatencySleepNV = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_sleep_info: *const LatencySleepInfoNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkSetLatencyMarkerNV = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_latency_marker_info: *const SetLatencyMarkerInfoNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetLatencyTimingsNV = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_latency_marker_info: *mut GetLatencyMarkerInfoNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkQueueNotifyOutOfBandNV =
    unsafe extern "system" fn(queue: Queue, p_queue_type_info: *const OutOfBandQueueTypeInfoNV);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetRenderingAttachmentLocations = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_location_info: *const RenderingAttachmentLocationInfo,
);
#[doc = "Provided by Vulkan version 1.4"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetRenderingInputAttachmentIndices = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_input_attachment_index_info: *const RenderingInputAttachmentIndexInfo,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthClampRangeEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    depth_clamp_mode: DepthClampModeEXT,
    p_depth_clamp_range: *const DepthClampRangeEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceCooperativeMatrixFlexibleDimensionsPropertiesNV =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        p_property_count: *mut u32,
        p_properties: *mut CooperativeMatrixFlexibleDimensionsPropertiesNV,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryMetalHandleEXT = unsafe extern "system" fn(
    device: Device,
    p_get_metal_handle_info: *const MemoryGetMetalHandleInfoEXT,
    p_handle: *mut *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryMetalHandlePropertiesEXT = unsafe extern "system" fn(
    device: Device,
    handle_type: ExternalMemoryHandleTypeFlags,
    p_handle: *const ffi::c_void,
    p_memory_metal_handle_properties: *mut MemoryMetalHandlePropertiesEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceCooperativeVectorPropertiesNV = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_property_count: *mut u32,
    p_properties: *mut CooperativeVectorPropertiesNV,
)
    -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkConvertCooperativeVectorMatrixNV = unsafe extern "system" fn(
    device: Device,
    p_info: *const ConvertCooperativeVectorMatrixInfoNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdConvertCooperativeVectorMatrixNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    info_count: u32,
    p_infos: *const ConvertCooperativeVectorMatrixInfoNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDispatchTileQCOM = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_dispatch_tile_info: *const DispatchTileInfoQCOM,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginPerTileExecutionQCOM = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_per_tile_begin_info: *const PerTileBeginInfoQCOM,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndPerTileExecutionQCOM = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_per_tile_end_info: *const PerTileEndInfoQCOM,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateExternalComputeQueueNV = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const ExternalComputeQueueCreateInfoNV,
    p_allocator: *const AllocationCallbacks,
    p_external_queue: *mut ExternalComputeQueueNV,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyExternalComputeQueueNV = unsafe extern "system" fn(
    device: Device,
    external_queue: ExternalComputeQueueNV,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetExternalComputeQueueDataNV = unsafe extern "system" fn(
    external_queue: ExternalComputeQueueNV,
    params: *mut ExternalComputeQueueDataParamsNV,
    p_data: *mut ffi::c_void,
);
#[allow(non_camel_case_types)]
pub type PFN_vkEnumeratePhysicalDeviceShaderInstrumentationMetricsARM =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        p_description_count: *mut u32,
        p_descriptions: *mut ShaderInstrumentationMetricDescriptionARM,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateShaderInstrumentationARM = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const ShaderInstrumentationCreateInfoARM,
    p_allocator: *const AllocationCallbacks,
    p_instrumentation: *mut ShaderInstrumentationARM,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyShaderInstrumentationARM = unsafe extern "system" fn(
    device: Device,
    instrumentation: ShaderInstrumentationARM,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginShaderInstrumentationARM = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    instrumentation: ShaderInstrumentationARM,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndShaderInstrumentationARM =
    unsafe extern "system" fn(command_buffer: CommandBuffer);
#[allow(non_camel_case_types)]
pub type PFN_vkGetShaderInstrumentationValuesARM = unsafe extern "system" fn(
    device: Device,
    instrumentation: ShaderInstrumentationARM,
    p_metric_block_count: *mut u32,
    p_metric_values: *mut ffi::c_void,
    flags: ShaderInstrumentationValuesFlagsARM,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkClearShaderInstrumentationMetricsARM =
    unsafe extern "system" fn(device: Device, instrumentation: ShaderInstrumentationARM);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateTensorARM = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const TensorCreateInfoARM,
    p_allocator: *const AllocationCallbacks,
    p_tensor: *mut TensorARM,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyTensorARM = unsafe extern "system" fn(
    device: Device,
    tensor: TensorARM,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateTensorViewARM = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const TensorViewCreateInfoARM,
    p_allocator: *const AllocationCallbacks,
    p_view: *mut TensorViewARM,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyTensorViewARM = unsafe extern "system" fn(
    device: Device,
    tensor_view: TensorViewARM,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetTensorMemoryRequirementsARM = unsafe extern "system" fn(
    device: Device,
    p_info: *const TensorMemoryRequirementsInfoARM,
    p_memory_requirements: *mut MemoryRequirements2,
);
#[allow(non_camel_case_types)]
pub type PFN_vkBindTensorMemoryARM = unsafe extern "system" fn(
    device: Device,
    bind_info_count: u32,
    p_bind_infos: *const BindTensorMemoryInfoARM,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceTensorMemoryRequirementsARM = unsafe extern "system" fn(
    device: Device,
    p_info: *const DeviceTensorMemoryRequirementsARM,
    p_memory_requirements: *mut MemoryRequirements2,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyTensorARM = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_copy_tensor_info: *const CopyTensorInfoARM,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetTensorOpaqueCaptureDescriptorDataARM = unsafe extern "system" fn(
    device: Device,
    p_info: *const TensorCaptureDescriptorDataInfoARM,
    p_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetTensorViewOpaqueCaptureDescriptorDataARM = unsafe extern "system" fn(
    device: Device,
    p_info: *const TensorViewCaptureDescriptorDataInfoARM,
    p_data: *mut ffi::c_void,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceExternalTensorPropertiesARM = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    p_external_tensor_info: *const PhysicalDeviceExternalTensorInfoARM,
    p_external_tensor_properties: *mut ExternalTensorPropertiesARM,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDataGraphPipelinesARM = unsafe extern "system" fn(
    device: Device,
    deferred_operation: DeferredOperationKHR,
    pipeline_cache: PipelineCache,
    create_info_count: u32,
    p_create_infos: *const DataGraphPipelineCreateInfoARM,
    p_allocator: *const AllocationCallbacks,
    p_pipelines: *mut Pipeline,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDataGraphPipelineSessionARM = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const DataGraphPipelineSessionCreateInfoARM,
    p_allocator: *const AllocationCallbacks,
    p_session: *mut DataGraphPipelineSessionARM,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDataGraphPipelineSessionBindPointRequirementsARM =
    unsafe extern "system" fn(
        device: Device,
        p_info: *const DataGraphPipelineSessionBindPointRequirementsInfoARM,
        p_bind_point_requirement_count: *mut u32,
        p_bind_point_requirements: *mut DataGraphPipelineSessionBindPointRequirementARM,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDataGraphPipelineSessionMemoryRequirementsARM = unsafe extern "system" fn(
    device: Device,
    p_info: *const DataGraphPipelineSessionMemoryRequirementsInfoARM,
    p_memory_requirements: *mut MemoryRequirements2,
);
#[allow(non_camel_case_types)]
pub type PFN_vkBindDataGraphPipelineSessionMemoryARM = unsafe extern "system" fn(
    device: Device,
    bind_info_count: u32,
    p_bind_infos: *const BindDataGraphPipelineSessionMemoryInfoARM,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyDataGraphPipelineSessionARM = unsafe extern "system" fn(
    device: Device,
    session: DataGraphPipelineSessionARM,
    p_allocator: *const AllocationCallbacks,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDispatchDataGraphARM = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    session: DataGraphPipelineSessionARM,
    p_info: *const DataGraphPipelineDispatchInfoARM,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetDataGraphPipelineAvailablePropertiesARM = unsafe extern "system" fn(
    device: Device,
    p_pipeline_info: *const DataGraphPipelineInfoARM,
    p_properties_count: *mut u32,
    p_properties: *mut DataGraphPipelinePropertyARM,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetDataGraphPipelinePropertiesARM = unsafe extern "system" fn(
    device: Device,
    p_pipeline_info: *const DataGraphPipelineInfoARM,
    properties_count: u32,
    p_properties: *mut DataGraphPipelinePropertyQueryResultARM,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceQueueFamilyDataGraphPropertiesARM =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        queue_family_index: u32,
        p_queue_family_data_graph_property_count: *mut u32,
        p_queue_family_data_graph_properties: *mut QueueFamilyDataGraphPropertiesARM,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceQueueFamilyDataGraphProcessingEnginePropertiesARM = unsafe extern "system" fn (physical_device : PhysicalDevice , p_queue_family_data_graph_processing_engine_info : * const PhysicalDeviceQueueFamilyDataGraphProcessingEngineInfoARM , p_queue_family_data_graph_processing_engine_properties : * mut QueueFamilyDataGraphProcessingEnginePropertiesARM ,) ;
#[allow(non_camel_case_types)]
pub type PFN_vkGetNativeBufferPropertiesOHOS = unsafe extern "system" fn(
    device: Device,
    buffer: *const OH_NativeBuffer,
    p_properties: *mut NativeBufferPropertiesOHOS,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetMemoryNativeBufferOHOS = unsafe extern "system" fn(
    device: Device,
    p_info: *const MemoryGetNativeBufferInfoOHOS,
    p_buffer: *mut *mut OH_NativeBuffer,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetSwapchainGrallocUsageOHOS = unsafe extern "system" fn(
    device: Device,
    format: Format,
    image_usage: ImageUsageFlags,
    gralloc_usage: *mut u64,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkAcquireImageOHOS = unsafe extern "system" fn(
    device: Device,
    image: Image,
    native_fence_fd: i32,
    semaphore: Semaphore,
    fence: Fence,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkQueueSignalReleaseImageOHOS = unsafe extern "system" fn(
    queue: Queue,
    wait_semaphore_count: u32,
    p_wait_semaphores: *const Semaphore,
    image: Image,
    p_native_fence_fd: *mut i32,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkQueueSetPerfHintQCOM =
    unsafe extern "system" fn(queue: Queue, p_perf_hint_info: *const PerfHintInfoQCOM) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkEnumeratePhysicalDeviceQueueFamilyPerformanceCountersByRegionARM =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        queue_family_index: u32,
        p_counter_count: *mut u32,
        p_counters: *mut PerformanceCounterARM,
        p_counter_descriptions: *mut PerformanceCounterDescriptionARM,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetComputeOccupancyPriorityNV = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_parameters: *const ComputeOccupancyPriorityParametersNV,
);
#[allow(non_camel_case_types)]
pub type PFN_vkWriteSamplerDescriptorsEXT = unsafe extern "system" fn(
    device: Device,
    sampler_count: u32,
    p_samplers: *const SamplerCreateInfo,
    p_descriptors: *const HostAddressRangeEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkWriteResourceDescriptorsEXT = unsafe extern "system" fn(
    device: Device,
    resource_count: u32,
    p_resources: *const ResourceDescriptorInfoEXT,
    p_descriptors: *const HostAddressRangeEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindSamplerHeapEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, p_bind_info: *const BindHeapInfoEXT);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindResourceHeapEXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, p_bind_info: *const BindHeapInfoEXT);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPushDataEXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_push_data_info: *const PushDataInfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkRegisterCustomBorderColorEXT = unsafe extern "system" fn(
    device: Device,
    p_border_color: *const SamplerCustomBorderColorCreateInfoEXT,
    request_index: Bool32,
    p_index: *mut u32,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkUnregisterCustomBorderColorEXT =
    unsafe extern "system" fn(device: Device, index: u32);
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageOpaqueCaptureDataEXT = unsafe extern "system" fn(
    device: Device,
    image_count: u32,
    p_images: *const Image,
    p_datas: *mut HostAddressRangeEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceDescriptorSizeEXT = unsafe extern "system" fn(
    physical_device: PhysicalDevice,
    descriptor_type: DescriptorType,
) -> DeviceSize;
#[allow(non_camel_case_types)]
pub type PFN_vkGetTensorOpaqueCaptureDataARM = unsafe extern "system" fn(
    device: Device,
    tensor_count: u32,
    p_tensors: *const TensorARM,
    p_datas: *mut HostAddressRangeEXT,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyMemoryKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_copy_memory_info: *const CopyDeviceMemoryInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyMemoryToImageKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_copy_memory_info: *const CopyDeviceMemoryImageInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyImageToMemoryKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_copy_memory_info: *const CopyDeviceMemoryImageInfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdUpdateMemoryKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_dst_range: *const DeviceAddressRangeKHR,
    dst_flags: AddressCommandFlagsKHR,
    data_size: DeviceSize,
    p_data: *const ffi::c_void,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdFillMemoryKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_dst_range: *const DeviceAddressRangeKHR,
    dst_flags: AddressCommandFlagsKHR,
    data: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyQueryPoolResultsToMemoryKHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    query_pool: QueryPool,
    first_query: u32,
    query_count: u32,
    p_dst_range: *const StridedDeviceAddressRangeKHR,
    dst_flags: AddressCommandFlagsKHR,
    query_result_flags: QueryResultFlags,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginConditionalRendering2EXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_conditional_rendering_begin: *const ConditionalRenderingBeginInfo2EXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindTransformFeedbackBuffers2EXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_binding: u32,
    binding_count: u32,
    p_binding_infos: *const BindTransformFeedbackBuffer2InfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginTransformFeedback2EXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_counter_range: u32,
    counter_range_count: u32,
    p_counter_infos: *const BindTransformFeedbackBuffer2InfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndTransformFeedback2EXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_counter_range: u32,
    counter_range_count: u32,
    p_counter_infos: *const BindTransformFeedbackBuffer2InfoEXT,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndirectByteCount2EXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    instance_count: u32,
    first_instance: u32,
    p_counter_info: *const BindTransformFeedbackBuffer2InfoEXT,
    counter_offset: u32,
    vertex_stride: u32,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdWriteMarkerToMemoryAMD =
    unsafe extern "system" fn(command_buffer: CommandBuffer, p_info: *const MemoryMarkerInfoAMD);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindIndexBuffer3KHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_info: *const BindIndexBuffer3InfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindVertexBuffers3KHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    first_binding: u32,
    binding_count: u32,
    p_binding_infos: *const BindVertexBuffer3InfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndirect2KHR =
    unsafe extern "system" fn(command_buffer: CommandBuffer, p_info: *const DrawIndirect2InfoKHR);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndexedIndirect2KHR =
    unsafe extern "system" fn(command_buffer: CommandBuffer, p_info: *const DrawIndirect2InfoKHR);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndirectCount2KHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_info: *const DrawIndirectCount2InfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndexedIndirectCount2KHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_info: *const DrawIndirectCount2InfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawMeshTasksIndirect2EXT =
    unsafe extern "system" fn(command_buffer: CommandBuffer, p_info: *const DrawIndirect2InfoKHR);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawMeshTasksIndirectCount2EXT = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_info: *const DrawIndirectCount2InfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDispatchIndirect2KHR = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_info: *const DispatchIndirect2InfoKHR,
);
#[allow(non_camel_case_types)]
pub type PFN_vkCreateAccelerationStructure2KHR = unsafe extern "system" fn(
    device: Device,
    p_create_info: *const AccelerationStructureCreateInfo2KHR,
    p_allocator: *const AllocationCallbacks,
    p_acceleration_structure: *mut AccelerationStructureKHR,
) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceQueueFamilyDataGraphEngineOperationPropertiesARM =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        queue_family_index: u32,
        p_queue_family_data_graph_properties: *const QueueFamilyDataGraphPropertiesARM,
        p_properties: *mut BaseOutStructure,
    ) -> Result;
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDispatchParametersARM = unsafe extern "system" fn(
    command_buffer: CommandBuffer,
    p_dispatch_parameters: *const DispatchParametersARM,
);
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceQueueFamilyDataGraphOpticalFlowImageFormatsARM =
    unsafe extern "system" fn(
        physical_device: PhysicalDevice,
        queue_family_index: u32,
        p_queue_family_data_graph_properties: *const QueueFamilyDataGraphPropertiesARM,
        p_optical_flow_image_format_info: *const DataGraphOpticalFlowImageFormatInfoARM,
        p_format_count: *mut u32,
        p_image_format_properties: *mut DataGraphOpticalFlowImageFormatPropertiesARM,
    ) -> Result;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryUnmapFlagsKHR.html>"]
pub type MemoryUnmapFlagsKHR = MemoryUnmapFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkGeometryFlagsNV.html>"]
pub type GeometryFlagsNV = GeometryFlagsKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkGeometryInstanceFlagsNV.html>"]
pub type GeometryInstanceFlagsNV = GeometryInstanceFlagsKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBuildAccelerationStructureFlagsNV.html>"]
pub type BuildAccelerationStructureFlagsNV = BuildAccelerationStructureFlagsKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPrivateDataSlotCreateFlagsEXT.html>"]
pub type PrivateDataSlotCreateFlagsEXT = PrivateDataSlotCreateFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDescriptorUpdateTemplateCreateFlagsKHR.html>"]
pub type DescriptorUpdateTemplateCreateFlagsKHR = DescriptorUpdateTemplateCreateFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineCreationFeedbackFlagsEXT.html>"]
pub type PipelineCreationFeedbackFlagsEXT = PipelineCreationFeedbackFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSemaphoreWaitFlagsKHR.html>"]
pub type SemaphoreWaitFlagsKHR = SemaphoreWaitFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkAccessFlags2KHR.html>"]
pub type AccessFlags2KHR = AccessFlags2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineStageFlags2KHR.html>"]
pub type PipelineStageFlags2KHR = PipelineStageFlags2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkFormatFeatureFlags2KHR.html>"]
pub type FormatFeatureFlags2KHR = FormatFeatureFlags2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryDecompressionMethodFlagsNV.html>"]
pub type MemoryDecompressionMethodFlagsNV = MemoryDecompressionMethodFlagsEXT;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRenderingFlagsKHR.html>"]
pub type RenderingFlagsKHR = RenderingFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineCreateFlags2KHR.html>"]
pub type PipelineCreateFlags2KHR = PipelineCreateFlags2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBufferUsageFlags2KHR.html>"]
pub type BufferUsageFlags2KHR = BufferUsageFlags2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPeerMemoryFeatureFlagsKHR.html>"]
pub type PeerMemoryFeatureFlagsKHR = PeerMemoryFeatureFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryAllocateFlagsKHR.html>"]
pub type MemoryAllocateFlagsKHR = MemoryAllocateFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCommandPoolTrimFlagsKHR.html>"]
pub type CommandPoolTrimFlagsKHR = CommandPoolTrimFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalMemoryHandleTypeFlagsKHR.html>"]
pub type ExternalMemoryHandleTypeFlagsKHR = ExternalMemoryHandleTypeFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalMemoryFeatureFlagsKHR.html>"]
pub type ExternalMemoryFeatureFlagsKHR = ExternalMemoryFeatureFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalSemaphoreHandleTypeFlagsKHR.html>"]
pub type ExternalSemaphoreHandleTypeFlagsKHR = ExternalSemaphoreHandleTypeFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalSemaphoreFeatureFlagsKHR.html>"]
pub type ExternalSemaphoreFeatureFlagsKHR = ExternalSemaphoreFeatureFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSemaphoreImportFlagsKHR.html>"]
pub type SemaphoreImportFlagsKHR = SemaphoreImportFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalFenceHandleTypeFlagsKHR.html>"]
pub type ExternalFenceHandleTypeFlagsKHR = ExternalFenceHandleTypeFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalFenceFeatureFlagsKHR.html>"]
pub type ExternalFenceFeatureFlagsKHR = ExternalFenceFeatureFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkFenceImportFlagsKHR.html>"]
pub type FenceImportFlagsKHR = FenceImportFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDescriptorBindingFlagsEXT.html>"]
pub type DescriptorBindingFlagsEXT = DescriptorBindingFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkResolveModeFlagsKHR.html>"]
pub type ResolveModeFlagsKHR = ResolveModeFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkToolPurposeFlagsEXT.html>"]
pub type ToolPurposeFlagsEXT = ToolPurposeFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSubmitFlagsKHR.html>"]
pub type SubmitFlagsKHR = SubmitFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkHostImageCopyFlagsEXT.html>"]
pub type HostImageCopyFlagsEXT = HostImageCopyFlags;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPresentScalingFlagsEXT.html>"]
pub type PresentScalingFlagsEXT = PresentScalingFlagsKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPresentGravityFlagsEXT.html>"]
pub type PresentGravityFlagsEXT = PresentGravityFlagsKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDescriptorUpdateTemplateKHR.html>"]
pub type DescriptorUpdateTemplateKHR = DescriptorUpdateTemplate;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSamplerYcbcrConversionKHR.html>"]
pub type SamplerYcbcrConversionKHR = SamplerYcbcrConversion;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPrivateDataSlotEXT.html>"]
pub type PrivateDataSlotEXT = PrivateDataSlot;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRayTracingInvocationReorderModeNV.html>"]
pub type RayTracingInvocationReorderModeNV = RayTracingInvocationReorderModeEXT;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDescriptorUpdateTemplateTypeKHR.html>"]
pub type DescriptorUpdateTemplateTypeKHR = DescriptorUpdateTemplateType;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPointClippingBehaviorKHR.html>"]
pub type PointClippingBehaviorKHR = PointClippingBehavior;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkQueueGlobalPriorityKHR.html>"]
pub type QueueGlobalPriorityKHR = QueueGlobalPriority;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkQueueGlobalPriorityEXT.html>"]
pub type QueueGlobalPriorityEXT = QueueGlobalPriority;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkTimeDomainEXT.html>"]
pub type TimeDomainEXT = TimeDomainKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSemaphoreTypeKHR.html>"]
pub type SemaphoreTypeKHR = SemaphoreType;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCopyAccelerationStructureModeNV.html>"]
pub type CopyAccelerationStructureModeNV = CopyAccelerationStructureModeKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkAccelerationStructureTypeNV.html>"]
pub type AccelerationStructureTypeNV = AccelerationStructureTypeKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkGeometryTypeNV.html>"]
pub type GeometryTypeNV = GeometryTypeKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRayTracingShaderGroupTypeNV.html>"]
pub type RayTracingShaderGroupTypeNV = RayTracingShaderGroupTypeKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkLineRasterizationModeKHR.html>"]
pub type LineRasterizationModeKHR = LineRasterizationMode;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkLineRasterizationModeEXT.html>"]
pub type LineRasterizationModeEXT = LineRasterizationMode;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRobustnessBufferBehaviorEXT.html>"]
pub type PipelineRobustnessBufferBehaviorEXT = PipelineRobustnessBufferBehavior;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRobustnessImageBehaviorEXT.html>"]
pub type PipelineRobustnessImageBehaviorEXT = PipelineRobustnessImageBehavior;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceFaultVendorBinaryHeaderVersionEXT.html>"]
pub type DeviceFaultVendorBinaryHeaderVersionEXT = DeviceFaultVendorBinaryHeaderVersionKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkScopeNV.html>"]
pub type ScopeNV = ScopeKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkComponentTypeNV.html>"]
pub type ComponentTypeNV = ComponentTypeKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkTessellationDomainOriginKHR.html>"]
pub type TessellationDomainOriginKHR = TessellationDomainOrigin;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSamplerYcbcrModelConversionKHR.html>"]
pub type SamplerYcbcrModelConversionKHR = SamplerYcbcrModelConversion;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSamplerYcbcrRangeKHR.html>"]
pub type SamplerYcbcrRangeKHR = SamplerYcbcrRange;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkChromaLocationKHR.html>"]
pub type ChromaLocationKHR = ChromaLocation;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSamplerReductionModeEXT.html>"]
pub type SamplerReductionModeEXT = SamplerReductionMode;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkShaderFloatControlsIndependenceKHR.html>"]
pub type ShaderFloatControlsIndependenceKHR = ShaderFloatControlsIndependence;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceFaultAddressTypeEXT.html>"]
pub type DeviceFaultAddressTypeEXT = DeviceFaultAddressTypeKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDriverIdKHR.html>"]
pub type DriverIdKHR = DriverId;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBufferUsageFlags2CreateInfoKHR.html>"]
pub type BufferUsageFlags2CreateInfoKHR<'a> = BufferUsageFlags2CreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCopyMemoryIndirectCommandNV.html>"]
pub type CopyMemoryIndirectCommandNV = CopyMemoryIndirectCommandKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCopyMemoryToImageIndirectCommandNV.html>"]
pub type CopyMemoryToImageIndirectCommandNV = CopyMemoryToImageIndirectCommandKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineCreateFlags2CreateInfoKHR.html>"]
pub type PipelineCreateFlags2CreateInfoKHR<'a> = PipelineCreateFlags2CreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceExternalSciBufFeaturesNV.html>"]
pub type PhysicalDeviceExternalSciBufFeaturesNV<'a> =
    PhysicalDeviceExternalMemorySciBufFeaturesNV<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDevicePrivateDataCreateInfoEXT.html>"]
pub type DevicePrivateDataCreateInfoEXT<'a> = DevicePrivateDataCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPrivateDataSlotCreateInfoEXT.html>"]
pub type PrivateDataSlotCreateInfoEXT<'a> = PrivateDataSlotCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevicePrivateDataFeaturesEXT.html>"]
pub type PhysicalDevicePrivateDataFeaturesEXT<'a> = PhysicalDevicePrivateDataFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceFeatures2KHR.html>"]
pub type PhysicalDeviceFeatures2KHR<'a> = PhysicalDeviceFeatures2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceProperties2KHR.html>"]
pub type PhysicalDeviceProperties2KHR<'a> = PhysicalDeviceProperties2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkFormatProperties2KHR.html>"]
pub type FormatProperties2KHR<'a> = FormatProperties2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageFormatProperties2KHR.html>"]
pub type ImageFormatProperties2KHR<'a> = ImageFormatProperties2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceImageFormatInfo2KHR.html>"]
pub type PhysicalDeviceImageFormatInfo2KHR<'a> = PhysicalDeviceImageFormatInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkQueueFamilyProperties2KHR.html>"]
pub type QueueFamilyProperties2KHR<'a> = QueueFamilyProperties2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMemoryProperties2KHR.html>"]
pub type PhysicalDeviceMemoryProperties2KHR<'a> = PhysicalDeviceMemoryProperties2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSparseImageFormatProperties2KHR.html>"]
pub type SparseImageFormatProperties2KHR<'a> = SparseImageFormatProperties2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceSparseImageFormatInfo2KHR.html>"]
pub type PhysicalDeviceSparseImageFormatInfo2KHR<'a> = PhysicalDeviceSparseImageFormatInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevicePushDescriptorPropertiesKHR.html>"]
pub type PhysicalDevicePushDescriptorPropertiesKHR<'a> = PhysicalDevicePushDescriptorProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkConformanceVersionKHR.html>"]
pub type ConformanceVersionKHR = ConformanceVersion;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceDriverPropertiesKHR.html>"]
pub type PhysicalDeviceDriverPropertiesKHR<'a> = PhysicalDeviceDriverProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceVariablePointersFeaturesKHR.html>"]
pub type PhysicalDeviceVariablePointersFeaturesKHR<'a> = PhysicalDeviceVariablePointersFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceVariablePointerFeaturesKHR.html>"]
pub type PhysicalDeviceVariablePointerFeaturesKHR<'a> = PhysicalDeviceVariablePointersFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceVariablePointerFeatures.html>"]
pub type PhysicalDeviceVariablePointerFeatures<'a> = PhysicalDeviceVariablePointersFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalMemoryPropertiesKHR.html>"]
pub type ExternalMemoryPropertiesKHR = ExternalMemoryProperties;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceExternalImageFormatInfoKHR.html>"]
pub type PhysicalDeviceExternalImageFormatInfoKHR<'a> = PhysicalDeviceExternalImageFormatInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalImageFormatPropertiesKHR.html>"]
pub type ExternalImageFormatPropertiesKHR<'a> = ExternalImageFormatProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceExternalBufferInfoKHR.html>"]
pub type PhysicalDeviceExternalBufferInfoKHR<'a> = PhysicalDeviceExternalBufferInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalBufferPropertiesKHR.html>"]
pub type ExternalBufferPropertiesKHR<'a> = ExternalBufferProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceIDPropertiesKHR.html>"]
pub type PhysicalDeviceIDPropertiesKHR<'a> = PhysicalDeviceIDProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalMemoryImageCreateInfoKHR.html>"]
pub type ExternalMemoryImageCreateInfoKHR<'a> = ExternalMemoryImageCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalMemoryBufferCreateInfoKHR.html>"]
pub type ExternalMemoryBufferCreateInfoKHR<'a> = ExternalMemoryBufferCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExportMemoryAllocateInfoKHR.html>"]
pub type ExportMemoryAllocateInfoKHR<'a> = ExportMemoryAllocateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceExternalSemaphoreInfoKHR.html>"]
pub type PhysicalDeviceExternalSemaphoreInfoKHR<'a> = PhysicalDeviceExternalSemaphoreInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalSemaphorePropertiesKHR.html>"]
pub type ExternalSemaphorePropertiesKHR<'a> = ExternalSemaphoreProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExportSemaphoreCreateInfoKHR.html>"]
pub type ExportSemaphoreCreateInfoKHR<'a> = ExportSemaphoreCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceExternalFenceInfoKHR.html>"]
pub type PhysicalDeviceExternalFenceInfoKHR<'a> = PhysicalDeviceExternalFenceInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExternalFencePropertiesKHR.html>"]
pub type ExternalFencePropertiesKHR<'a> = ExternalFenceProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkExportFenceCreateInfoKHR.html>"]
pub type ExportFenceCreateInfoKHR<'a> = ExportFenceCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMultiviewFeaturesKHR.html>"]
pub type PhysicalDeviceMultiviewFeaturesKHR<'a> = PhysicalDeviceMultiviewFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMultiviewPropertiesKHR.html>"]
pub type PhysicalDeviceMultiviewPropertiesKHR<'a> = PhysicalDeviceMultiviewProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRenderPassMultiviewCreateInfoKHR.html>"]
pub type RenderPassMultiviewCreateInfoKHR<'a> = RenderPassMultiviewCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceGroupPropertiesKHR.html>"]
pub type PhysicalDeviceGroupPropertiesKHR<'a> = PhysicalDeviceGroupProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryAllocateFlagsInfoKHR.html>"]
pub type MemoryAllocateFlagsInfoKHR<'a> = MemoryAllocateFlagsInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBindBufferMemoryInfoKHR.html>"]
pub type BindBufferMemoryInfoKHR<'a> = BindBufferMemoryInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBindBufferMemoryDeviceGroupInfoKHR.html>"]
pub type BindBufferMemoryDeviceGroupInfoKHR<'a> = BindBufferMemoryDeviceGroupInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBindImageMemoryInfoKHR.html>"]
pub type BindImageMemoryInfoKHR<'a> = BindImageMemoryInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBindImageMemoryDeviceGroupInfoKHR.html>"]
pub type BindImageMemoryDeviceGroupInfoKHR<'a> = BindImageMemoryDeviceGroupInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceGroupRenderPassBeginInfoKHR.html>"]
pub type DeviceGroupRenderPassBeginInfoKHR<'a> = DeviceGroupRenderPassBeginInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceGroupCommandBufferBeginInfoKHR.html>"]
pub type DeviceGroupCommandBufferBeginInfoKHR<'a> = DeviceGroupCommandBufferBeginInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceGroupSubmitInfoKHR.html>"]
pub type DeviceGroupSubmitInfoKHR<'a> = DeviceGroupSubmitInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceGroupBindSparseInfoKHR.html>"]
pub type DeviceGroupBindSparseInfoKHR<'a> = DeviceGroupBindSparseInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceGroupDeviceCreateInfoKHR.html>"]
pub type DeviceGroupDeviceCreateInfoKHR<'a> = DeviceGroupDeviceCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDescriptorUpdateTemplateEntryKHR.html>"]
pub type DescriptorUpdateTemplateEntryKHR = DescriptorUpdateTemplateEntry;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDescriptorUpdateTemplateCreateInfoKHR.html>"]
pub type DescriptorUpdateTemplateCreateInfoKHR<'a> = DescriptorUpdateTemplateCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkInputAttachmentAspectReferenceKHR.html>"]
pub type InputAttachmentAspectReferenceKHR = InputAttachmentAspectReference;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRenderPassInputAttachmentAspectCreateInfoKHR.html>"]
pub type RenderPassInputAttachmentAspectCreateInfoKHR<'a> =
    RenderPassInputAttachmentAspectCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevice16BitStorageFeaturesKHR.html>"]
pub type PhysicalDevice16BitStorageFeaturesKHR<'a> = PhysicalDevice16BitStorageFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderSubgroupExtendedTypesFeaturesKHR.html>"]
pub type PhysicalDeviceShaderSubgroupExtendedTypesFeaturesKHR<'a> =
    PhysicalDeviceShaderSubgroupExtendedTypesFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBufferMemoryRequirementsInfo2KHR.html>"]
pub type BufferMemoryRequirementsInfo2KHR<'a> = BufferMemoryRequirementsInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceBufferMemoryRequirementsKHR.html>"]
pub type DeviceBufferMemoryRequirementsKHR<'a> = DeviceBufferMemoryRequirements<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageMemoryRequirementsInfo2KHR.html>"]
pub type ImageMemoryRequirementsInfo2KHR<'a> = ImageMemoryRequirementsInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageSparseMemoryRequirementsInfo2KHR.html>"]
pub type ImageSparseMemoryRequirementsInfo2KHR<'a> = ImageSparseMemoryRequirementsInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceImageMemoryRequirementsKHR.html>"]
pub type DeviceImageMemoryRequirementsKHR<'a> = DeviceImageMemoryRequirements<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryRequirements2KHR.html>"]
pub type MemoryRequirements2KHR<'a> = MemoryRequirements2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSparseImageMemoryRequirements2KHR.html>"]
pub type SparseImageMemoryRequirements2KHR<'a> = SparseImageMemoryRequirements2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevicePointClippingPropertiesKHR.html>"]
pub type PhysicalDevicePointClippingPropertiesKHR<'a> = PhysicalDevicePointClippingProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryDedicatedRequirementsKHR.html>"]
pub type MemoryDedicatedRequirementsKHR<'a> = MemoryDedicatedRequirements<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryDedicatedAllocateInfoKHR.html>"]
pub type MemoryDedicatedAllocateInfoKHR<'a> = MemoryDedicatedAllocateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageViewUsageCreateInfoKHR.html>"]
pub type ImageViewUsageCreateInfoKHR<'a> = ImageViewUsageCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineTessellationDomainOriginStateCreateInfoKHR.html>"]
pub type PipelineTessellationDomainOriginStateCreateInfoKHR<'a> =
    PipelineTessellationDomainOriginStateCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSamplerYcbcrConversionInfoKHR.html>"]
pub type SamplerYcbcrConversionInfoKHR<'a> = SamplerYcbcrConversionInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSamplerYcbcrConversionCreateInfoKHR.html>"]
pub type SamplerYcbcrConversionCreateInfoKHR<'a> = SamplerYcbcrConversionCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBindImagePlaneMemoryInfoKHR.html>"]
pub type BindImagePlaneMemoryInfoKHR<'a> = BindImagePlaneMemoryInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImagePlaneMemoryRequirementsInfoKHR.html>"]
pub type ImagePlaneMemoryRequirementsInfoKHR<'a> = ImagePlaneMemoryRequirementsInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceSamplerYcbcrConversionFeaturesKHR.html>"]
pub type PhysicalDeviceSamplerYcbcrConversionFeaturesKHR<'a> =
    PhysicalDeviceSamplerYcbcrConversionFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSamplerYcbcrConversionImageFormatPropertiesKHR.html>"]
pub type SamplerYcbcrConversionImageFormatPropertiesKHR<'a> =
    SamplerYcbcrConversionImageFormatProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceSamplerFilterMinmaxPropertiesEXT.html>"]
pub type PhysicalDeviceSamplerFilterMinmaxPropertiesEXT<'a> =
    PhysicalDeviceSamplerFilterMinmaxProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSamplerReductionModeCreateInfoEXT.html>"]
pub type SamplerReductionModeCreateInfoEXT<'a> = SamplerReductionModeCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceInlineUniformBlockFeaturesEXT.html>"]
pub type PhysicalDeviceInlineUniformBlockFeaturesEXT<'a> =
    PhysicalDeviceInlineUniformBlockFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceInlineUniformBlockPropertiesEXT.html>"]
pub type PhysicalDeviceInlineUniformBlockPropertiesEXT<'a> =
    PhysicalDeviceInlineUniformBlockProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkWriteDescriptorSetInlineUniformBlockEXT.html>"]
pub type WriteDescriptorSetInlineUniformBlockEXT<'a> = WriteDescriptorSetInlineUniformBlock<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDescriptorPoolInlineUniformBlockCreateInfoEXT.html>"]
pub type DescriptorPoolInlineUniformBlockCreateInfoEXT<'a> =
    DescriptorPoolInlineUniformBlockCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageFormatListCreateInfoKHR.html>"]
pub type ImageFormatListCreateInfoKHR<'a> = ImageFormatListCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMaintenance3PropertiesKHR.html>"]
pub type PhysicalDeviceMaintenance3PropertiesKHR<'a> = PhysicalDeviceMaintenance3Properties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMaintenance4FeaturesKHR.html>"]
pub type PhysicalDeviceMaintenance4FeaturesKHR<'a> = PhysicalDeviceMaintenance4Features<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMaintenance4PropertiesKHR.html>"]
pub type PhysicalDeviceMaintenance4PropertiesKHR<'a> = PhysicalDeviceMaintenance4Properties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMaintenance5FeaturesKHR.html>"]
pub type PhysicalDeviceMaintenance5FeaturesKHR<'a> = PhysicalDeviceMaintenance5Features<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMaintenance5PropertiesKHR.html>"]
pub type PhysicalDeviceMaintenance5PropertiesKHR<'a> = PhysicalDeviceMaintenance5Properties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMaintenance6FeaturesKHR.html>"]
pub type PhysicalDeviceMaintenance6FeaturesKHR<'a> = PhysicalDeviceMaintenance6Features<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMaintenance6PropertiesKHR.html>"]
pub type PhysicalDeviceMaintenance6PropertiesKHR<'a> = PhysicalDeviceMaintenance6Properties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRenderingAreaInfoKHR.html>"]
pub type RenderingAreaInfoKHR<'a> = RenderingAreaInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDescriptorSetLayoutSupportKHR.html>"]
pub type DescriptorSetLayoutSupportKHR<'a> = DescriptorSetLayoutSupport<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderDrawParameterFeatures.html>"]
pub type PhysicalDeviceShaderDrawParameterFeatures<'a> =
    PhysicalDeviceShaderDrawParametersFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderFloat16Int8FeaturesKHR.html>"]
pub type PhysicalDeviceShaderFloat16Int8FeaturesKHR<'a> =
    PhysicalDeviceShaderFloat16Int8Features<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceFloat16Int8FeaturesKHR.html>"]
pub type PhysicalDeviceFloat16Int8FeaturesKHR<'a> = PhysicalDeviceShaderFloat16Int8Features<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceFloatControlsPropertiesKHR.html>"]
pub type PhysicalDeviceFloatControlsPropertiesKHR<'a> = PhysicalDeviceFloatControlsProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceHostQueryResetFeaturesEXT.html>"]
pub type PhysicalDeviceHostQueryResetFeaturesEXT<'a> = PhysicalDeviceHostQueryResetFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceQueueGlobalPriorityCreateInfoKHR.html>"]
pub type DeviceQueueGlobalPriorityCreateInfoKHR<'a> = DeviceQueueGlobalPriorityCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceQueueGlobalPriorityCreateInfoEXT.html>"]
pub type DeviceQueueGlobalPriorityCreateInfoEXT<'a> = DeviceQueueGlobalPriorityCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceGlobalPriorityQueryFeaturesKHR.html>"]
pub type PhysicalDeviceGlobalPriorityQueryFeaturesKHR<'a> =
    PhysicalDeviceGlobalPriorityQueryFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceGlobalPriorityQueryFeaturesEXT.html>"]
pub type PhysicalDeviceGlobalPriorityQueryFeaturesEXT<'a> =
    PhysicalDeviceGlobalPriorityQueryFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkQueueFamilyGlobalPriorityPropertiesKHR.html>"]
pub type QueueFamilyGlobalPriorityPropertiesKHR<'a> = QueueFamilyGlobalPriorityProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkQueueFamilyGlobalPriorityPropertiesEXT.html>"]
pub type QueueFamilyGlobalPriorityPropertiesEXT<'a> = QueueFamilyGlobalPriorityProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCalibratedTimestampInfoEXT.html>"]
pub type CalibratedTimestampInfoEXT<'a> = CalibratedTimestampInfoKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceDescriptorIndexingFeaturesEXT.html>"]
pub type PhysicalDeviceDescriptorIndexingFeaturesEXT<'a> =
    PhysicalDeviceDescriptorIndexingFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceDescriptorIndexingPropertiesEXT.html>"]
pub type PhysicalDeviceDescriptorIndexingPropertiesEXT<'a> =
    PhysicalDeviceDescriptorIndexingProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDescriptorSetLayoutBindingFlagsCreateInfoEXT.html>"]
pub type DescriptorSetLayoutBindingFlagsCreateInfoEXT<'a> =
    DescriptorSetLayoutBindingFlagsCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDescriptorSetVariableDescriptorCountAllocateInfoEXT.html>"]
pub type DescriptorSetVariableDescriptorCountAllocateInfoEXT<'a> =
    DescriptorSetVariableDescriptorCountAllocateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDescriptorSetVariableDescriptorCountLayoutSupportEXT.html>"]
pub type DescriptorSetVariableDescriptorCountLayoutSupportEXT<'a> =
    DescriptorSetVariableDescriptorCountLayoutSupport<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkAttachmentDescription2KHR.html>"]
pub type AttachmentDescription2KHR<'a> = AttachmentDescription2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkAttachmentReference2KHR.html>"]
pub type AttachmentReference2KHR<'a> = AttachmentReference2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSubpassDescription2KHR.html>"]
pub type SubpassDescription2KHR<'a> = SubpassDescription2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSubpassDependency2KHR.html>"]
pub type SubpassDependency2KHR<'a> = SubpassDependency2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRenderPassCreateInfo2KHR.html>"]
pub type RenderPassCreateInfo2KHR<'a> = RenderPassCreateInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSubpassBeginInfoKHR.html>"]
pub type SubpassBeginInfoKHR<'a> = SubpassBeginInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSubpassEndInfoKHR.html>"]
pub type SubpassEndInfoKHR<'a> = SubpassEndInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceTimelineSemaphoreFeaturesKHR.html>"]
pub type PhysicalDeviceTimelineSemaphoreFeaturesKHR<'a> =
    PhysicalDeviceTimelineSemaphoreFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceTimelineSemaphorePropertiesKHR.html>"]
pub type PhysicalDeviceTimelineSemaphorePropertiesKHR<'a> =
    PhysicalDeviceTimelineSemaphoreProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSemaphoreTypeCreateInfoKHR.html>"]
pub type SemaphoreTypeCreateInfoKHR<'a> = SemaphoreTypeCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkTimelineSemaphoreSubmitInfoKHR.html>"]
pub type TimelineSemaphoreSubmitInfoKHR<'a> = TimelineSemaphoreSubmitInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSemaphoreWaitInfoKHR.html>"]
pub type SemaphoreWaitInfoKHR<'a> = SemaphoreWaitInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSemaphoreSignalInfoKHR.html>"]
pub type SemaphoreSignalInfoKHR<'a> = SemaphoreSignalInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkVertexInputBindingDivisorDescriptionKHR.html>"]
pub type VertexInputBindingDivisorDescriptionKHR = VertexInputBindingDivisorDescription;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkVertexInputBindingDivisorDescriptionEXT.html>"]
pub type VertexInputBindingDivisorDescriptionEXT = VertexInputBindingDivisorDescription;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineVertexInputDivisorStateCreateInfoKHR.html>"]
pub type PipelineVertexInputDivisorStateCreateInfoKHR<'a> =
    PipelineVertexInputDivisorStateCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineVertexInputDivisorStateCreateInfoEXT.html>"]
pub type PipelineVertexInputDivisorStateCreateInfoEXT<'a> =
    PipelineVertexInputDivisorStateCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceVertexAttributeDivisorPropertiesKHR.html>"]
pub type PhysicalDeviceVertexAttributeDivisorPropertiesKHR<'a> =
    PhysicalDeviceVertexAttributeDivisorProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevice8BitStorageFeaturesKHR.html>"]
pub type PhysicalDevice8BitStorageFeaturesKHR<'a> = PhysicalDevice8BitStorageFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceVulkanMemoryModelFeaturesKHR.html>"]
pub type PhysicalDeviceVulkanMemoryModelFeaturesKHR<'a> =
    PhysicalDeviceVulkanMemoryModelFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderAtomicInt64FeaturesKHR.html>"]
pub type PhysicalDeviceShaderAtomicInt64FeaturesKHR<'a> =
    PhysicalDeviceShaderAtomicInt64Features<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceVertexAttributeDivisorFeaturesKHR.html>"]
pub type PhysicalDeviceVertexAttributeDivisorFeaturesKHR<'a> =
    PhysicalDeviceVertexAttributeDivisorFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceVertexAttributeDivisorFeaturesEXT.html>"]
pub type PhysicalDeviceVertexAttributeDivisorFeaturesEXT<'a> =
    PhysicalDeviceVertexAttributeDivisorFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceDepthStencilResolvePropertiesKHR.html>"]
pub type PhysicalDeviceDepthStencilResolvePropertiesKHR<'a> =
    PhysicalDeviceDepthStencilResolveProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSubpassDescriptionDepthStencilResolveKHR.html>"]
pub type SubpassDescriptionDepthStencilResolveKHR<'a> = SubpassDescriptionDepthStencilResolve<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceComputeShaderDerivativesFeaturesNV.html>"]
pub type PhysicalDeviceComputeShaderDerivativesFeaturesNV<'a> =
    PhysicalDeviceComputeShaderDerivativesFeaturesKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceFragmentShaderBarycentricFeaturesNV.html>"]
pub type PhysicalDeviceFragmentShaderBarycentricFeaturesNV<'a> =
    PhysicalDeviceFragmentShaderBarycentricFeaturesKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceCopyMemoryIndirectPropertiesNV.html>"]
pub type PhysicalDeviceCopyMemoryIndirectPropertiesNV<'a> =
    PhysicalDeviceCopyMemoryIndirectPropertiesKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMemoryDecompressionFeaturesNV.html>"]
pub type PhysicalDeviceMemoryDecompressionFeaturesNV<'a> =
    PhysicalDeviceMemoryDecompressionFeaturesEXT<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMemoryDecompressionPropertiesNV.html>"]
pub type PhysicalDeviceMemoryDecompressionPropertiesNV<'a> =
    PhysicalDeviceMemoryDecompressionPropertiesEXT<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageStencilUsageCreateInfoEXT.html>"]
pub type ImageStencilUsageCreateInfoEXT<'a> = ImageStencilUsageCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceFragmentDensityMapOffsetFeaturesQCOM.html>"]
pub type PhysicalDeviceFragmentDensityMapOffsetFeaturesQCOM<'a> =
    PhysicalDeviceFragmentDensityMapOffsetFeaturesEXT<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceFragmentDensityMapOffsetPropertiesQCOM.html>"]
pub type PhysicalDeviceFragmentDensityMapOffsetPropertiesQCOM<'a> =
    PhysicalDeviceFragmentDensityMapOffsetPropertiesEXT<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSubpassFragmentDensityMapOffsetEndInfoQCOM.html>"]
pub type SubpassFragmentDensityMapOffsetEndInfoQCOM<'a> =
    RenderPassFragmentDensityMapOffsetEndInfoEXT<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceScalarBlockLayoutFeaturesEXT.html>"]
pub type PhysicalDeviceScalarBlockLayoutFeaturesEXT<'a> =
    PhysicalDeviceScalarBlockLayoutFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceUniformBufferStandardLayoutFeaturesKHR.html>"]
pub type PhysicalDeviceUniformBufferStandardLayoutFeaturesKHR<'a> =
    PhysicalDeviceUniformBufferStandardLayoutFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceBufferDeviceAddressFeaturesKHR.html>"]
pub type PhysicalDeviceBufferDeviceAddressFeaturesKHR<'a> =
    PhysicalDeviceBufferDeviceAddressFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceBufferAddressFeaturesEXT.html>"]
pub type PhysicalDeviceBufferAddressFeaturesEXT<'a> =
    PhysicalDeviceBufferDeviceAddressFeaturesEXT<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBufferDeviceAddressInfoKHR.html>"]
pub type BufferDeviceAddressInfoKHR<'a> = BufferDeviceAddressInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBufferDeviceAddressInfoEXT.html>"]
pub type BufferDeviceAddressInfoEXT<'a> = BufferDeviceAddressInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBufferOpaqueCaptureAddressCreateInfoKHR.html>"]
pub type BufferOpaqueCaptureAddressCreateInfoKHR<'a> = BufferOpaqueCaptureAddressCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceImagelessFramebufferFeaturesKHR.html>"]
pub type PhysicalDeviceImagelessFramebufferFeaturesKHR<'a> =
    PhysicalDeviceImagelessFramebufferFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkFramebufferAttachmentsCreateInfoKHR.html>"]
pub type FramebufferAttachmentsCreateInfoKHR<'a> = FramebufferAttachmentsCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkFramebufferAttachmentImageInfoKHR.html>"]
pub type FramebufferAttachmentImageInfoKHR<'a> = FramebufferAttachmentImageInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRenderPassAttachmentBeginInfoKHR.html>"]
pub type RenderPassAttachmentBeginInfoKHR<'a> = RenderPassAttachmentBeginInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceTextureCompressionASTCHDRFeaturesEXT.html>"]
pub type PhysicalDeviceTextureCompressionASTCHDRFeaturesEXT<'a> =
    PhysicalDeviceTextureCompressionASTCHDRFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineCreationFeedbackEXT.html>"]
pub type PipelineCreationFeedbackEXT = PipelineCreationFeedback;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineCreationFeedbackCreateInfoEXT.html>"]
pub type PipelineCreationFeedbackCreateInfoEXT<'a> = PipelineCreationFeedbackCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkQueryPoolCreateInfoINTEL.html>"]
pub type QueryPoolCreateInfoINTEL<'a> = QueryPoolPerformanceQueryCreateInfoINTEL<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceIndexTypeUint8FeaturesKHR.html>"]
pub type PhysicalDeviceIndexTypeUint8FeaturesKHR<'a> = PhysicalDeviceIndexTypeUint8Features<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceIndexTypeUint8FeaturesEXT.html>"]
pub type PhysicalDeviceIndexTypeUint8FeaturesEXT<'a> = PhysicalDeviceIndexTypeUint8Features<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceSeparateDepthStencilLayoutsFeaturesKHR.html>"]
pub type PhysicalDeviceSeparateDepthStencilLayoutsFeaturesKHR<'a> =
    PhysicalDeviceSeparateDepthStencilLayoutsFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkAttachmentReferenceStencilLayoutKHR.html>"]
pub type AttachmentReferenceStencilLayoutKHR<'a> = AttachmentReferenceStencilLayout<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkAttachmentDescriptionStencilLayoutKHR.html>"]
pub type AttachmentDescriptionStencilLayoutKHR<'a> = AttachmentDescriptionStencilLayout<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineInfoEXT.html>"]
pub type PipelineInfoEXT<'a> = PipelineInfoKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderDemoteToHelperInvocationFeaturesEXT.html>"]
pub type PhysicalDeviceShaderDemoteToHelperInvocationFeaturesEXT<'a> =
    PhysicalDeviceShaderDemoteToHelperInvocationFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceTexelBufferAlignmentPropertiesEXT.html>"]
pub type PhysicalDeviceTexelBufferAlignmentPropertiesEXT<'a> =
    PhysicalDeviceTexelBufferAlignmentProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceSubgroupSizeControlFeaturesEXT.html>"]
pub type PhysicalDeviceSubgroupSizeControlFeaturesEXT<'a> =
    PhysicalDeviceSubgroupSizeControlFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceSubgroupSizeControlPropertiesEXT.html>"]
pub type PhysicalDeviceSubgroupSizeControlPropertiesEXT<'a> =
    PhysicalDeviceSubgroupSizeControlProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineShaderStageRequiredSubgroupSizeCreateInfoEXT.html>"]
pub type PipelineShaderStageRequiredSubgroupSizeCreateInfoEXT<'a> =
    PipelineShaderStageRequiredSubgroupSizeCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkShaderRequiredSubgroupSizeCreateInfoEXT.html>"]
pub type ShaderRequiredSubgroupSizeCreateInfoEXT<'a> =
    PipelineShaderStageRequiredSubgroupSizeCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryOpaqueCaptureAddressAllocateInfoKHR.html>"]
pub type MemoryOpaqueCaptureAddressAllocateInfoKHR<'a> = MemoryOpaqueCaptureAddressAllocateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceMemoryOpaqueCaptureAddressInfoKHR.html>"]
pub type DeviceMemoryOpaqueCaptureAddressInfoKHR<'a> = DeviceMemoryOpaqueCaptureAddressInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceLineRasterizationFeaturesKHR.html>"]
pub type PhysicalDeviceLineRasterizationFeaturesKHR<'a> =
    PhysicalDeviceLineRasterizationFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceLineRasterizationFeaturesEXT.html>"]
pub type PhysicalDeviceLineRasterizationFeaturesEXT<'a> =
    PhysicalDeviceLineRasterizationFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceLineRasterizationPropertiesKHR.html>"]
pub type PhysicalDeviceLineRasterizationPropertiesKHR<'a> =
    PhysicalDeviceLineRasterizationProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceLineRasterizationPropertiesEXT.html>"]
pub type PhysicalDeviceLineRasterizationPropertiesEXT<'a> =
    PhysicalDeviceLineRasterizationProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRasterizationLineStateCreateInfoKHR.html>"]
pub type PipelineRasterizationLineStateCreateInfoKHR<'a> =
    PipelineRasterizationLineStateCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRasterizationLineStateCreateInfoEXT.html>"]
pub type PipelineRasterizationLineStateCreateInfoEXT<'a> =
    PipelineRasterizationLineStateCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevicePipelineCreationCacheControlFeaturesEXT.html>"]
pub type PhysicalDevicePipelineCreationCacheControlFeaturesEXT<'a> =
    PhysicalDevicePipelineCreationCacheControlFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceToolPropertiesEXT.html>"]
pub type PhysicalDeviceToolPropertiesEXT<'a> = PhysicalDeviceToolProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkAabbPositionsNV.html>"]
pub type AabbPositionsNV = AabbPositionsKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkTransformMatrixNV.html>"]
pub type TransformMatrixNV = TransformMatrixKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkAccelerationStructureInstanceNV.html>"]
pub type AccelerationStructureInstanceNV = AccelerationStructureInstanceKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceZeroInitializeWorkgroupMemoryFeaturesKHR.html>"]
pub type PhysicalDeviceZeroInitializeWorkgroupMemoryFeaturesKHR<'a> =
    PhysicalDeviceZeroInitializeWorkgroupMemoryFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceRobustness2FeaturesEXT.html>"]
pub type PhysicalDeviceRobustness2FeaturesEXT<'a> = PhysicalDeviceRobustness2FeaturesKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceRobustness2PropertiesEXT.html>"]
pub type PhysicalDeviceRobustness2PropertiesEXT<'a> = PhysicalDeviceRobustness2PropertiesKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceImageRobustnessFeaturesEXT.html>"]
pub type PhysicalDeviceImageRobustnessFeaturesEXT<'a> = PhysicalDeviceImageRobustnessFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBufferCopy2KHR.html>"]
pub type BufferCopy2KHR<'a> = BufferCopy2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageCopy2KHR.html>"]
pub type ImageCopy2KHR<'a> = ImageCopy2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageBlit2KHR.html>"]
pub type ImageBlit2KHR<'a> = ImageBlit2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBufferImageCopy2KHR.html>"]
pub type BufferImageCopy2KHR<'a> = BufferImageCopy2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageResolve2KHR.html>"]
pub type ImageResolve2KHR<'a> = ImageResolve2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCopyBufferInfo2KHR.html>"]
pub type CopyBufferInfo2KHR<'a> = CopyBufferInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCopyImageInfo2KHR.html>"]
pub type CopyImageInfo2KHR<'a> = CopyImageInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBlitImageInfo2KHR.html>"]
pub type BlitImageInfo2KHR<'a> = BlitImageInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCopyBufferToImageInfo2KHR.html>"]
pub type CopyBufferToImageInfo2KHR<'a> = CopyBufferToImageInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCopyImageToBufferInfo2KHR.html>"]
pub type CopyImageToBufferInfo2KHR<'a> = CopyImageToBufferInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkResolveImageInfo2KHR.html>"]
pub type ResolveImageInfo2KHR<'a> = ResolveImageInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderTerminateInvocationFeaturesKHR.html>"]
pub type PhysicalDeviceShaderTerminateInvocationFeaturesKHR<'a> =
    PhysicalDeviceShaderTerminateInvocationFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMutableDescriptorTypeFeaturesVALVE.html>"]
pub type PhysicalDeviceMutableDescriptorTypeFeaturesVALVE<'a> =
    PhysicalDeviceMutableDescriptorTypeFeaturesEXT<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMutableDescriptorTypeListVALVE.html>"]
pub type MutableDescriptorTypeListVALVE = MutableDescriptorTypeListEXT;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMutableDescriptorTypeCreateInfoVALVE.html>"]
pub type MutableDescriptorTypeCreateInfoVALVE<'a> = MutableDescriptorTypeCreateInfoEXT<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryBarrier2KHR.html>"]
pub type MemoryBarrier2KHR<'a> = MemoryBarrier2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageMemoryBarrier2KHR.html>"]
pub type ImageMemoryBarrier2KHR<'a> = ImageMemoryBarrier2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBufferMemoryBarrier2KHR.html>"]
pub type BufferMemoryBarrier2KHR<'a> = BufferMemoryBarrier2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDependencyInfoKHR.html>"]
pub type DependencyInfoKHR<'a> = DependencyInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSemaphoreSubmitInfoKHR.html>"]
pub type SemaphoreSubmitInfoKHR<'a> = SemaphoreSubmitInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCommandBufferSubmitInfoKHR.html>"]
pub type CommandBufferSubmitInfoKHR<'a> = CommandBufferSubmitInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSubmitInfo2KHR.html>"]
pub type SubmitInfo2KHR<'a> = SubmitInfo2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceSynchronization2FeaturesKHR.html>"]
pub type PhysicalDeviceSynchronization2FeaturesKHR<'a> = PhysicalDeviceSynchronization2Features<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceHostImageCopyFeaturesEXT.html>"]
pub type PhysicalDeviceHostImageCopyFeaturesEXT<'a> = PhysicalDeviceHostImageCopyFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceHostImageCopyPropertiesEXT.html>"]
pub type PhysicalDeviceHostImageCopyPropertiesEXT<'a> = PhysicalDeviceHostImageCopyProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryToImageCopyEXT.html>"]
pub type MemoryToImageCopyEXT<'a> = MemoryToImageCopy<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageToMemoryCopyEXT.html>"]
pub type ImageToMemoryCopyEXT<'a> = ImageToMemoryCopy<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCopyMemoryToImageInfoEXT.html>"]
pub type CopyMemoryToImageInfoEXT<'a> = CopyMemoryToImageInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCopyImageToMemoryInfoEXT.html>"]
pub type CopyImageToMemoryInfoEXT<'a> = CopyImageToMemoryInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCopyImageToImageInfoEXT.html>"]
pub type CopyImageToImageInfoEXT<'a> = CopyImageToImageInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkHostImageLayoutTransitionInfoEXT.html>"]
pub type HostImageLayoutTransitionInfoEXT<'a> = HostImageLayoutTransitionInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSubresourceHostMemcpySizeEXT.html>"]
pub type SubresourceHostMemcpySizeEXT<'a> = SubresourceHostMemcpySize<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkHostImageCopyDevicePerformanceQueryEXT.html>"]
pub type HostImageCopyDevicePerformanceQueryEXT<'a> = HostImageCopyDevicePerformanceQuery<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevicePipelineProtectedAccessFeaturesEXT.html>"]
pub type PhysicalDevicePipelineProtectedAccessFeaturesEXT<'a> =
    PhysicalDevicePipelineProtectedAccessFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderIntegerDotProductFeaturesKHR.html>"]
pub type PhysicalDeviceShaderIntegerDotProductFeaturesKHR<'a> =
    PhysicalDeviceShaderIntegerDotProductFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderIntegerDotProductPropertiesKHR.html>"]
pub type PhysicalDeviceShaderIntegerDotProductPropertiesKHR<'a> =
    PhysicalDeviceShaderIntegerDotProductProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkFormatProperties3KHR.html>"]
pub type FormatProperties3KHR<'a> = FormatProperties3<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRenderingCreateInfoKHR.html>"]
pub type PipelineRenderingCreateInfoKHR<'a> = PipelineRenderingCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRenderingInfoKHR.html>"]
pub type RenderingInfoKHR<'a> = RenderingInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRenderingEndInfoEXT.html>"]
pub type RenderingEndInfoEXT<'a> = RenderingEndInfoKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRenderingAttachmentInfoKHR.html>"]
pub type RenderingAttachmentInfoKHR<'a> = RenderingAttachmentInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceDynamicRenderingFeaturesKHR.html>"]
pub type PhysicalDeviceDynamicRenderingFeaturesKHR<'a> = PhysicalDeviceDynamicRenderingFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkCommandBufferInheritanceRenderingInfoKHR.html>"]
pub type CommandBufferInheritanceRenderingInfoKHR<'a> = CommandBufferInheritanceRenderingInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkAttachmentSampleCountInfoNV.html>"]
pub type AttachmentSampleCountInfoNV<'a> = AttachmentSampleCountInfoAMD<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceRasterizationOrderAttachmentAccessFeaturesARM.html>"]
pub type PhysicalDeviceRasterizationOrderAttachmentAccessFeaturesARM<'a> =
    PhysicalDeviceRasterizationOrderAttachmentAccessFeaturesEXT<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageSubresource2KHR.html>"]
pub type ImageSubresource2KHR<'a> = ImageSubresource2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageSubresource2EXT.html>"]
pub type ImageSubresource2EXT<'a> = ImageSubresource2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSubresourceLayout2KHR.html>"]
pub type SubresourceLayout2KHR<'a> = SubresourceLayout2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSubresourceLayout2EXT.html>"]
pub type SubresourceLayout2EXT<'a> = SubresourceLayout2<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevicePipelineRobustnessFeaturesEXT.html>"]
pub type PhysicalDevicePipelineRobustnessFeaturesEXT<'a> =
    PhysicalDevicePipelineRobustnessFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRobustnessCreateInfoEXT.html>"]
pub type PipelineRobustnessCreateInfoEXT<'a> = PipelineRobustnessCreateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevicePipelineRobustnessPropertiesEXT.html>"]
pub type PhysicalDevicePipelineRobustnessPropertiesEXT<'a> =
    PhysicalDevicePipelineRobustnessProperties<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceDepthClampZeroOneFeaturesEXT.html>"]
pub type PhysicalDeviceDepthClampZeroOneFeaturesEXT<'a> =
    PhysicalDeviceDepthClampZeroOneFeaturesKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceFaultAddressInfoEXT.html>"]
pub type DeviceFaultAddressInfoEXT = DeviceFaultAddressInfoKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceFaultVendorInfoEXT.html>"]
pub type DeviceFaultVendorInfoEXT = DeviceFaultVendorInfoKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceFaultVendorBinaryHeaderVersionOneEXT.html>"]
pub type DeviceFaultVendorBinaryHeaderVersionOneEXT = DeviceFaultVendorBinaryHeaderVersionOneKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSurfacePresentModeEXT.html>"]
pub type SurfacePresentModeEXT<'a> = SurfacePresentModeKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSurfacePresentScalingCapabilitiesEXT.html>"]
pub type SurfacePresentScalingCapabilitiesEXT<'a> = SurfacePresentScalingCapabilitiesKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSurfacePresentModeCompatibilityEXT.html>"]
pub type SurfacePresentModeCompatibilityEXT<'a> = SurfacePresentModeCompatibilityKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceSwapchainMaintenance1FeaturesEXT.html>"]
pub type PhysicalDeviceSwapchainMaintenance1FeaturesEXT<'a> =
    PhysicalDeviceSwapchainMaintenance1FeaturesKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSwapchainPresentFenceInfoEXT.html>"]
pub type SwapchainPresentFenceInfoEXT<'a> = SwapchainPresentFenceInfoKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSwapchainPresentModesCreateInfoEXT.html>"]
pub type SwapchainPresentModesCreateInfoEXT<'a> = SwapchainPresentModesCreateInfoKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSwapchainPresentModeInfoEXT.html>"]
pub type SwapchainPresentModeInfoEXT<'a> = SwapchainPresentModeInfoKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkSwapchainPresentScalingCreateInfoEXT.html>"]
pub type SwapchainPresentScalingCreateInfoEXT<'a> = SwapchainPresentScalingCreateInfoKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkReleaseSwapchainImagesInfoEXT.html>"]
pub type ReleaseSwapchainImagesInfoEXT<'a> = ReleaseSwapchainImagesInfoKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceImageSubresourceInfoKHR.html>"]
pub type DeviceImageSubresourceInfoKHR<'a> = DeviceImageSubresourceInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryMapInfoKHR.html>"]
pub type MemoryMapInfoKHR<'a> = MemoryMapInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryUnmapInfoKHR.html>"]
pub type MemoryUnmapInfoKHR<'a> = MemoryUnmapInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBindMemoryStatusKHR.html>"]
pub type BindMemoryStatusKHR<'a> = BindMemoryStatus<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkBindDescriptorSetsInfoKHR.html>"]
pub type BindDescriptorSetsInfoKHR<'a> = BindDescriptorSetsInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPushConstantsInfoKHR.html>"]
pub type PushConstantsInfoKHR<'a> = PushConstantsInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPushDescriptorSetInfoKHR.html>"]
pub type PushDescriptorSetInfoKHR<'a> = PushDescriptorSetInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPushDescriptorSetWithTemplateInfoKHR.html>"]
pub type PushDescriptorSetWithTemplateInfoKHR<'a> = PushDescriptorSetWithTemplateInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderSubgroupRotateFeaturesKHR.html>"]
pub type PhysicalDeviceShaderSubgroupRotateFeaturesKHR<'a> =
    PhysicalDeviceShaderSubgroupRotateFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderExpectAssumeFeaturesKHR.html>"]
pub type PhysicalDeviceShaderExpectAssumeFeaturesKHR<'a> =
    PhysicalDeviceShaderExpectAssumeFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceShaderFloatControls2FeaturesKHR.html>"]
pub type PhysicalDeviceShaderFloatControls2FeaturesKHR<'a> =
    PhysicalDeviceShaderFloatControls2Features<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceDynamicRenderingLocalReadFeaturesKHR.html>"]
pub type PhysicalDeviceDynamicRenderingLocalReadFeaturesKHR<'a> =
    PhysicalDeviceDynamicRenderingLocalReadFeatures<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRenderingAttachmentLocationInfoKHR.html>"]
pub type RenderingAttachmentLocationInfoKHR<'a> = RenderingAttachmentLocationInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkRenderingInputAttachmentIndexInfoKHR.html>"]
pub type RenderingInputAttachmentIndexInfoKHR<'a> = RenderingInputAttachmentIndexInfo<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDevicePresentModeFifoLatestReadyFeaturesEXT.html>"]
pub type PhysicalDevicePresentModeFifoLatestReadyFeaturesEXT<'a> =
    PhysicalDevicePresentModeFifoLatestReadyFeaturesKHR<'a>;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceAddressRangeEXT.html>"]
pub type DeviceAddressRangeEXT = DeviceAddressRangeKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkResetQueryPoolEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkResetQueryPoolEXT = PFN_vkResetQueryPool;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetRenderingAreaGranularityKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetRenderingAreaGranularityKHR = PFN_vkGetRenderingAreaGranularity;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPhysicalDeviceFeatures2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceFeatures2KHR = PFN_vkGetPhysicalDeviceFeatures2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPhysicalDeviceProperties2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceProperties2KHR = PFN_vkGetPhysicalDeviceProperties2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPhysicalDeviceFormatProperties2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceFormatProperties2KHR = PFN_vkGetPhysicalDeviceFormatProperties2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPhysicalDeviceImageFormatProperties2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceImageFormatProperties2KHR =
    PFN_vkGetPhysicalDeviceImageFormatProperties2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPhysicalDeviceQueueFamilyProperties2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceQueueFamilyProperties2KHR =
    PFN_vkGetPhysicalDeviceQueueFamilyProperties2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPhysicalDeviceMemoryProperties2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceMemoryProperties2KHR = PFN_vkGetPhysicalDeviceMemoryProperties2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPhysicalDeviceSparseImageFormatProperties2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceSparseImageFormatProperties2KHR =
    PFN_vkGetPhysicalDeviceSparseImageFormatProperties2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdPushDescriptorSetKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPushDescriptorSetKHR = PFN_vkCmdPushDescriptorSet;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkTrimCommandPoolKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkTrimCommandPoolKHR = PFN_vkTrimCommandPool;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPhysicalDeviceExternalBufferPropertiesKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceExternalBufferPropertiesKHR =
    PFN_vkGetPhysicalDeviceExternalBufferProperties;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPhysicalDeviceExternalSemaphorePropertiesKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceExternalSemaphorePropertiesKHR =
    PFN_vkGetPhysicalDeviceExternalSemaphoreProperties;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPhysicalDeviceExternalFencePropertiesKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceExternalFencePropertiesKHR =
    PFN_vkGetPhysicalDeviceExternalFenceProperties;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkEnumeratePhysicalDeviceGroupsKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkEnumeratePhysicalDeviceGroupsKHR = PFN_vkEnumeratePhysicalDeviceGroups;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetDeviceGroupPeerMemoryFeaturesKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceGroupPeerMemoryFeaturesKHR = PFN_vkGetDeviceGroupPeerMemoryFeatures;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkBindBufferMemory2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkBindBufferMemory2KHR = PFN_vkBindBufferMemory2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkBindImageMemory2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkBindImageMemory2KHR = PFN_vkBindImageMemory2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetDeviceMaskKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDeviceMaskKHR = PFN_vkCmdSetDeviceMask;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdDispatchBaseKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDispatchBaseKHR = PFN_vkCmdDispatchBase;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCreateDescriptorUpdateTemplateKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateDescriptorUpdateTemplateKHR = PFN_vkCreateDescriptorUpdateTemplate;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkDestroyDescriptorUpdateTemplateKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyDescriptorUpdateTemplateKHR = PFN_vkDestroyDescriptorUpdateTemplate;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkUpdateDescriptorSetWithTemplateKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkUpdateDescriptorSetWithTemplateKHR = PFN_vkUpdateDescriptorSetWithTemplate;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdPushDescriptorSetWithTemplateKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPushDescriptorSetWithTemplateKHR = PFN_vkCmdPushDescriptorSetWithTemplate;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetBufferMemoryRequirements2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetBufferMemoryRequirements2KHR = PFN_vkGetBufferMemoryRequirements2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetImageMemoryRequirements2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageMemoryRequirements2KHR = PFN_vkGetImageMemoryRequirements2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetImageSparseMemoryRequirements2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageSparseMemoryRequirements2KHR = PFN_vkGetImageSparseMemoryRequirements2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetDeviceBufferMemoryRequirementsKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceBufferMemoryRequirementsKHR = PFN_vkGetDeviceBufferMemoryRequirements;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetDeviceImageMemoryRequirementsKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceImageMemoryRequirementsKHR = PFN_vkGetDeviceImageMemoryRequirements;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetDeviceImageSparseMemoryRequirementsKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceImageSparseMemoryRequirementsKHR =
    PFN_vkGetDeviceImageSparseMemoryRequirements;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCreateSamplerYcbcrConversionKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateSamplerYcbcrConversionKHR = PFN_vkCreateSamplerYcbcrConversion;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkDestroySamplerYcbcrConversionKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroySamplerYcbcrConversionKHR = PFN_vkDestroySamplerYcbcrConversion;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetDescriptorSetLayoutSupportKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDescriptorSetLayoutSupportKHR = PFN_vkGetDescriptorSetLayoutSupport;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPhysicalDeviceCalibrateableTimeDomainsEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceCalibrateableTimeDomainsEXT =
    PFN_vkGetPhysicalDeviceCalibrateableTimeDomainsKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetCalibratedTimestampsEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetCalibratedTimestampsEXT = PFN_vkGetCalibratedTimestampsKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCreateRenderPass2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreateRenderPass2KHR = PFN_vkCreateRenderPass2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdBeginRenderPass2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginRenderPass2KHR = PFN_vkCmdBeginRenderPass2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdNextSubpass2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdNextSubpass2KHR = PFN_vkCmdNextSubpass2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdEndRenderPass2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndRenderPass2KHR = PFN_vkCmdEndRenderPass2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetSemaphoreCounterValueKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetSemaphoreCounterValueKHR = PFN_vkGetSemaphoreCounterValue;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkWaitSemaphoresKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkWaitSemaphoresKHR = PFN_vkWaitSemaphores;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkSignalSemaphoreKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkSignalSemaphoreKHR = PFN_vkSignalSemaphore;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdDrawIndirectCountKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndirectCountKHR = PFN_vkCmdDrawIndirectCount;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdDrawIndirectCountAMD.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndirectCountAMD = PFN_vkCmdDrawIndirectCount;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdDrawIndexedIndirectCountKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndexedIndirectCountKHR = PFN_vkCmdDrawIndexedIndirectCount;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdDrawIndexedIndirectCountAMD.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdDrawIndexedIndirectCountAMD = PFN_vkCmdDrawIndexedIndirectCount;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetRayTracingShaderGroupHandlesNV.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetRayTracingShaderGroupHandlesNV = PFN_vkGetRayTracingShaderGroupHandlesKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetBufferOpaqueCaptureAddressKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetBufferOpaqueCaptureAddressKHR = PFN_vkGetBufferOpaqueCaptureAddress;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetBufferDeviceAddressKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetBufferDeviceAddressKHR = PFN_vkGetBufferDeviceAddress;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetBufferDeviceAddressEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetBufferDeviceAddressEXT = PFN_vkGetBufferDeviceAddress;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetDeviceMemoryOpaqueCaptureAddressKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceMemoryOpaqueCaptureAddressKHR = PFN_vkGetDeviceMemoryOpaqueCaptureAddress;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetLineStippleKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetLineStippleKHR = PFN_vkCmdSetLineStipple;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetLineStippleEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetLineStippleEXT = PFN_vkCmdSetLineStipple;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPhysicalDeviceToolPropertiesEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPhysicalDeviceToolPropertiesEXT = PFN_vkGetPhysicalDeviceToolProperties;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetCullModeEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetCullModeEXT = PFN_vkCmdSetCullMode;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetFrontFaceEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetFrontFaceEXT = PFN_vkCmdSetFrontFace;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetPrimitiveTopologyEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetPrimitiveTopologyEXT = PFN_vkCmdSetPrimitiveTopology;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetViewportWithCountEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetViewportWithCountEXT = PFN_vkCmdSetViewportWithCount;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetScissorWithCountEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetScissorWithCountEXT = PFN_vkCmdSetScissorWithCount;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdBindIndexBuffer2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindIndexBuffer2KHR = PFN_vkCmdBindIndexBuffer2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdBindVertexBuffers2EXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindVertexBuffers2EXT = PFN_vkCmdBindVertexBuffers2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetDepthTestEnableEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthTestEnableEXT = PFN_vkCmdSetDepthTestEnable;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetDepthWriteEnableEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthWriteEnableEXT = PFN_vkCmdSetDepthWriteEnable;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetDepthCompareOpEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthCompareOpEXT = PFN_vkCmdSetDepthCompareOp;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetDepthBoundsTestEnableEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthBoundsTestEnableEXT = PFN_vkCmdSetDepthBoundsTestEnable;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetStencilTestEnableEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetStencilTestEnableEXT = PFN_vkCmdSetStencilTestEnable;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetStencilOpEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetStencilOpEXT = PFN_vkCmdSetStencilOp;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetRasterizerDiscardEnableEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetRasterizerDiscardEnableEXT = PFN_vkCmdSetRasterizerDiscardEnable;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetDepthBiasEnableEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetDepthBiasEnableEXT = PFN_vkCmdSetDepthBiasEnable;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetPrimitiveRestartEnableEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetPrimitiveRestartEnableEXT = PFN_vkCmdSetPrimitiveRestartEnable;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCreatePrivateDataSlotEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCreatePrivateDataSlotEXT = PFN_vkCreatePrivateDataSlot;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkDestroyPrivateDataSlotEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkDestroyPrivateDataSlotEXT = PFN_vkDestroyPrivateDataSlot;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkSetPrivateDataEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkSetPrivateDataEXT = PFN_vkSetPrivateData;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetPrivateDataEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetPrivateDataEXT = PFN_vkGetPrivateData;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdCopyBuffer2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyBuffer2KHR = PFN_vkCmdCopyBuffer2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdCopyImage2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyImage2KHR = PFN_vkCmdCopyImage2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdBlitImage2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBlitImage2KHR = PFN_vkCmdBlitImage2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdCopyBufferToImage2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyBufferToImage2KHR = PFN_vkCmdCopyBufferToImage2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdCopyImageToBuffer2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdCopyImageToBuffer2KHR = PFN_vkCmdCopyImageToBuffer2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdResolveImage2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdResolveImage2KHR = PFN_vkCmdResolveImage2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetEvent2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetEvent2KHR = PFN_vkCmdSetEvent2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdResetEvent2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdResetEvent2KHR = PFN_vkCmdResetEvent2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdWaitEvents2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdWaitEvents2KHR = PFN_vkCmdWaitEvents2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdPipelineBarrier2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPipelineBarrier2KHR = PFN_vkCmdPipelineBarrier2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkQueueSubmit2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkQueueSubmit2KHR = PFN_vkQueueSubmit2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdWriteTimestamp2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdWriteTimestamp2KHR = PFN_vkCmdWriteTimestamp2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCopyMemoryToImageEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCopyMemoryToImageEXT = PFN_vkCopyMemoryToImage;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCopyImageToMemoryEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCopyImageToMemoryEXT = PFN_vkCopyImageToMemory;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCopyImageToImageEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCopyImageToImageEXT = PFN_vkCopyImageToImage;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkTransitionImageLayoutEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkTransitionImageLayoutEXT = PFN_vkTransitionImageLayout;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdBeginRenderingKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBeginRenderingKHR = PFN_vkCmdBeginRendering;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdEndRendering2EXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndRendering2EXT = PFN_vkCmdEndRendering2KHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdEndRenderingKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdEndRenderingKHR = PFN_vkCmdEndRendering;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetImageSubresourceLayout2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageSubresourceLayout2KHR = PFN_vkGetImageSubresourceLayout2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetImageSubresourceLayout2EXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetImageSubresourceLayout2EXT = PFN_vkGetImageSubresourceLayout2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkReleaseSwapchainImagesEXT.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkReleaseSwapchainImagesEXT = PFN_vkReleaseSwapchainImagesKHR;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkGetDeviceImageSubresourceLayoutKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkGetDeviceImageSubresourceLayoutKHR = PFN_vkGetDeviceImageSubresourceLayout;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkMapMemory2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkMapMemory2KHR = PFN_vkMapMemory2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkUnmapMemory2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkUnmapMemory2KHR = PFN_vkUnmapMemory2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdBindDescriptorSets2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdBindDescriptorSets2KHR = PFN_vkCmdBindDescriptorSets2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdPushConstants2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPushConstants2KHR = PFN_vkCmdPushConstants2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdPushDescriptorSet2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPushDescriptorSet2KHR = PFN_vkCmdPushDescriptorSet2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdPushDescriptorSetWithTemplate2KHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdPushDescriptorSetWithTemplate2KHR = PFN_vkCmdPushDescriptorSetWithTemplate2;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetRenderingAttachmentLocationsKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetRenderingAttachmentLocationsKHR = PFN_vkCmdSetRenderingAttachmentLocations;
#[doc = "<https://docs.vulkan.org/refpages/latest/refpages/source/PFN_vkCmdSetRenderingInputAttachmentIndicesKHR.html>"]
#[allow(non_camel_case_types)]
pub type PFN_vkCmdSetRenderingInputAttachmentIndicesKHR =
    PFN_vkCmdSetRenderingInputAttachmentIndices;
