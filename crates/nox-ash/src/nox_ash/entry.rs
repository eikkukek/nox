use {
    std::ffi::os_str::OsStr,
    core::ops::Deref,
    crate::{
        LoadingError,
        vk,
        prelude::VkResult,
    },
    super::Instance,
};

/// Holds the Vulkan functions independent of a particular instance.
///
/// There are slight modifications to the [`ash`] version, but is basically the same.
#[derive(Clone)]
pub struct Entry {
    entry: ash::Entry,
}

impl Deref for Entry {

    type Target = ash::Entry;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl Entry {

    /// Loads the default Vulkan library for the current platform.
    ///
    /// # Safety
    /// *From [`ash`] docs:*
    ///
    /// `dlopen`ing native libraries is inherently unsafe. The safety guidelines
    /// for [`Library::new()`] and [`Library::get()`] apply here.
    /// No Vulkan functions loaded directly or indirectly from this [`Entry`]
    /// may be called after it is dropped.
    #[inline(always)]
    pub unsafe fn load() -> Result<Self, LoadingError> {
        unsafe {
            Ok(Self {
                entry: ash::Entry::load()?
            })
        }
    }

    /// Loads Vulkan library at `path`.
    ///
    /// # Safety
    /// *From [`ash`] docs:*
    ///
    /// `dlopen`ing native libraries is inherently unsafe. The safety guidelines
    /// for [`Library::new()`] and [`Library::get()`] apply here.
    /// No Vulkan functions loaded directly or indirectly from this [`Entry`]
    /// may be called after it is dropped.
    #[inline(always)]
    pub unsafe fn load_from<S>(path: S) -> Result<Self, LoadingError>
        where S: AsRef<OsStr>
    {
        unsafe {
            Ok(Self {
                entry: ash::Entry::load_from(path)?
            })
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCreateInstance.html>
    ///
    /// # Safety
    /// *From [`ash`] docs:*
    ///
    /// The resulting Instance and any function-pointer objects
    /// (e.g. Device and extensions like khr::swapchain::Device) loaded from it
    /// may not be used after this Entry object is dropped,
    /// unless it was crated using Entry::linked() or Entry::from_parts_1_1().
    ///  Instance does not implement drop semantics and can only be destroyed via destroy_instance().
    #[inline(always)]
    pub unsafe fn create_instance(
        &self,
        create_info: &vk::InstanceCreateInfo<'_>,
        allocation_callbacks: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<Instance> {
        let instance = unsafe { 
            self.entry.create_instance(create_info, allocation_callbacks)?
        };
        Ok(Instance {
            instance,
        })
    }
}
