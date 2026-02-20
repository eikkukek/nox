pub use ash::vk::*;

use core::{
    ops::{Deref, DerefMut},
    ffi,
    marker::PhantomData,
    ptr,
};

use paste::paste;

pub trait StructureTypeExt {

    const SURFACE_CAPABILITIES_PRESENT_ID_2_KHR: Self;
    const PRESENT_ID_2_KHR: Self;
    const PHYSICAL_DEVICE_PRESENT_ID_2_FEATURES_KHR: Self;

    const SURFACE_CAPABILITIES_PRESENT_WAIT_2_KHR: Self;
    const PHYSICAL_DEVICE_PRESENT_WAIT_2_FEATURES_KHR: Self;
    const PRESENT_WAIT_INFO_2_KHR: Self;
}

pub trait SwapchainCreateFlagsKHRExt {

    const PRESENT_ID_2: Self;
    const PRESENT_WAIT_2: Self;
}

pub trait ExtendsStructure: Send + Sync {

    type What: ?Sized;

    /// Extracts the structure type field of self.
    fn s_type(&self) -> StructureType;

    /// Clears the `p_next` field of self.
    ///
    /// This along with [`ExtendsStructure::as_mut`] can be used to dynamically and safely build
    /// p_next chains of Vulkan structures.
    fn clear_p_next(&mut self);

    /// Gets a mutable reference to self as the underlying *Extends...* trait. For example, if this
    /// extends [`PhysicalDeviceFeatures2`] this returns a mutable reference to
    /// [`ExtendsPhysicalDeviceFeatures2`] trait object.
    ///
    /// This along with [`ExtendsStructure::clear_p_next`] can be used to dynamically and safely
    /// build p_next chains of Vulkan structures.
    fn as_mut(&mut self) -> &mut Self::What;

    /// Clones self to a box.
    fn boxed(&self) -> Box<dyn ExtendsStructure<What = Self::What>>;

    /// Gets a pointer to the underlying structure.
    fn as_ptr(&self) -> *const ();
}

pub trait ExtendsStructureExt: ExtendsStructure + TaggedStructure + Sized {

    type ExtendsObj;

    /// Boxes [`self`] as an Extends* trait objects.
    fn boxed_default() -> Self::ExtendsObj;

    /// Makes getting a reference to self easier.
    ///
    /// # Safety
    /// Only checks if the structure type of `obj` matches [`self`].
    unsafe fn as_ref(obj: &Self::ExtendsObj) -> Option<&Self>;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SurfaceCapabilitiesPresentId2KHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const ffi::c_void,
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PresentId2KHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const ffi::c_void,
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PhysicalDevicePresentId2FeaturesKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const ffi::c_void,
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SurfaceCapabilitiesPresentWait2KHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const ffi::c_void,
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PhysicalDevicePresentWait2FeaturesKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const ffi::c_void,
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PresentWait2InfoKHR<'a> {
    pub s_type: StructureType,
    pub p_next: *const ffi::c_void,
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

macro_rules! impl_extends_vk {
    (   $((
            $name:ident,
            $extends:ident,
            $(,)?
        )),+
        $(,)?
    ) => {
        $(

            impl ExtendsStructure for $name<'static> {

                type What = dyn $extends;

                #[inline(always)]
                fn s_type(&self) -> StructureType {
                    self.s_type
                }

                #[inline(always)]
                fn clear_p_next(&mut self) {
                    self.p_next = core::ptr::null_mut();
                }

                #[inline(always)]
                fn as_mut(&mut self) -> &mut Self::What {
                    self
                }

                #[inline(always)]
                fn boxed(&self) -> Box<dyn ExtendsStructure<What = Self::What>> {
                    Box::new(*self)
                }

                #[inline(always)]
                fn as_ptr(&self) -> *const () {
                    (self as *const Self).cast()
                }
            }

            impl ExtendsStructureExt for $name<'static> {

                paste! {

                    type ExtendsObj = [<$extends Obj>];
                    
                    #[inline(always)]
                    fn boxed_default() -> Self::ExtendsObj {
                        [<$extends Obj>]::new(Self {
                            s_type: Self::STRUCTURE_TYPE,
                            ..Default::default()
                        })
                    }

                    #[inline(always)]
                    unsafe fn as_ref(obj: &Self::ExtendsObj) -> Option<&Self> {
                        (obj.s_type() == Self::STRUCTURE_TYPE).then(|| unsafe {
                            obj.as_ptr()
                            .cast::<Self>()
                            .as_ref().unwrap()
                        })
                    }
                }
            }
        )+
    };
}

impl_extends_vk!(
    (
        PhysicalDeviceRobustness2FeaturesEXT,
        ExtendsPhysicalDeviceFeatures2,
    ),
    (
        PhysicalDeviceDynamicRenderingFeatures,
        ExtendsPhysicalDeviceFeatures2,
    ),
    (
        PhysicalDeviceTimelineSemaphoreFeatures, 
        ExtendsPhysicalDeviceFeatures2,
    ),
    (
        PhysicalDeviceSynchronization2Features,
        ExtendsPhysicalDeviceFeatures2,
    ),
    (
        PhysicalDeviceVulkan11Properties,
        ExtendsPhysicalDeviceProperties2,
    ),
    (
        PhysicalDevicePushDescriptorPropertiesKHR,
        ExtendsPhysicalDeviceProperties2,
    ),
    (
        PhysicalDevicePresentId2FeaturesKHR,
        ExtendsPhysicalDeviceFeatures2,
    ),
    (
        PhysicalDevicePresentWait2FeaturesKHR,
        ExtendsPhysicalDeviceFeatures2,
    ),
);

macro_rules! impl_extends_vk_obj {
    ($($name:ident),+ $(,)?) => {
        $(paste! {
            pub struct [<$name Obj>](
                Box<dyn ExtendsStructure<What = dyn $name>>
            );

            impl [<$name Obj>] {

                #[inline(always)]
                pub fn new(
                    s: impl ExtendsStructure<What = dyn $name> + 'static,
                ) -> Self {
                    Self(Box::new(s))
                }
            }

            impl Deref for [<$name Obj>] {

                type Target = dyn ExtendsStructure<What = dyn $name>;

                #[inline(always)]
                fn deref(&self) -> &Self::Target {
                    &*self.0
                }
            }

            impl DerefMut for [<$name Obj>] {

                #[inline(always)]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut *self.0
                }
            }

            impl Clone for [<$name Obj>] {
                
                #[inline(always)]
                fn clone(&self) -> Self {
                    Self(self.0.boxed())
                }
            }
        })+
    };
}

impl_extends_vk_obj!(ExtendsPhysicalDeviceFeatures2, ExtendsPhysicalDeviceProperties2);

impl StructureTypeExt for StructureType {

    const SURFACE_CAPABILITIES_PRESENT_ID_2_KHR: Self = Self::from_raw(1000479000);
    const PRESENT_ID_2_KHR: Self = Self::from_raw(1000479001);
    const PHYSICAL_DEVICE_PRESENT_ID_2_FEATURES_KHR: Self = Self::from_raw(1000479002);

    const SURFACE_CAPABILITIES_PRESENT_WAIT_2_KHR: Self = Self::from_raw(1000480000);
    const PHYSICAL_DEVICE_PRESENT_WAIT_2_FEATURES_KHR: Self = Self::from_raw(1000480001);
    const PRESENT_WAIT_INFO_2_KHR: Self = Self::from_raw(1000480002);
}

impl SwapchainCreateFlagsKHRExt for SwapchainCreateFlagsKHR {

    const PRESENT_ID_2: Self = Self::from_raw(0x00000040);
    const PRESENT_WAIT_2: Self = Self::from_raw(0x00000080);
}

/// VK_KHR_PRESENT_id2
pub const KHR_PRESENT_ID_2_NAME: &ffi::CStr = c"VK_KHR_present_id2";
pub const KHR_PRESENT_ID_2_SPEC_VERSION: u32 = 1;

/// VK_KHR_PRESENT_present_wait2
pub const KHR_PRESENT_WAIT_2_NAME: &ffi::CStr = c"VK_KHR_present_wait2";
pub const KHR_PRESENT_WAIT_2_SPEC_VERSION: u32 = 1;

#[allow(non_camel_case_types)]
pub type PFN_vkWaitForPresent2KHR = unsafe extern "system" fn(
    device: Device,
    swapchain: SwapchainKHR,
    p_present_wait2_info: *const PresentWait2InfoKHR,
) -> Result;
