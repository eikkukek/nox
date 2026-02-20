use std::sync::Arc;

use core::num::NonZeroU32;

use winit::{window::Window, dpi::PhysicalSize};
use raw_window_handle::{HasWindowHandle, HasDisplayHandle};

use compact_str::format_compact;

use nox_ash::vk;
use nox_mem::conditional::True;

use nox_alloc::arena::RwArena;

use crate::dev::error::*;

use super::{
    Swapchain,
    Vulkan,
    MemoryLayout,
};

#[derive(Clone, Copy)]
enum SwapchainState {
    Valid,
    OutOfDate {
        buffered_frames: NonZeroU32,
        size: PhysicalSize<u32>,
    },
}

pub(crate) struct Surface {
    vk: Arc<Vulkan>,
    handle: vk::SurfaceKHR,
    swapchain_state: SwapchainState,
    swapchain: Option<Swapchain>,
    alloc: RwArena,
    tmp_alloc: RwArena<True>,
    present_queue: vk::Queue,
    destroyed: bool,
}

impl Surface {

    pub fn new(
        window: &Window,
        vk: Arc<Vulkan>,
        buffered_frames: u32,
        layout: MemoryLayout
    ) -> Result<Self>
    {
        let alloc = RwArena::new(layout.swapchain_size())?;
        let tmp_alloc = RwArena::with_fallback(layout.tmp_arena_size())?;
        let handle = unsafe {
            ash_window
            ::create_surface(
                vk.entry(),
                vk.instance(),
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None
            )
            .context("failed to create vulkan surface")?
        };

        let properties = unsafe { vk.instance()
            .get_physical_device_queue_family_properties(vk.physical_device())
        };

        let mut queue_family_index = None;

        for (i, _) in properties.iter().enumerate() {
            let present_supported = unsafe {
                vk.surface_instance()
                    .get_physical_device_surface_support(vk.physical_device(), i as u32, handle)
                    .context("failed to query vulkan surface support")?
            };
            if present_supported {
                queue_family_index = Some(i as u32);
                break
            }
        }

        let Some(queue_family_index) = queue_family_index else {
            return Err(Error::just_context(format_compact!(
                "failed to find suitable present queue for window surface (window id {:?})",
                window.id(),
            )))
        };
        let present_queue = unsafe { vk.device().get_device_queue(queue_family_index, 0) };
        Ok(Self {
            vk,
            handle,
            swapchain: None,
            swapchain_state: SwapchainState::OutOfDate {
                buffered_frames: NonZeroU32::new(buffered_frames).unwrap(),
                size: window.inner_size(),
            },
            present_queue,
            alloc,
            tmp_alloc,
            destroyed: false,
        })
    }

    pub fn request_swapchain_update(
        &mut self,
        buffered_frames: u32,
        size: PhysicalSize<u32>,
    ) {
        self.swapchain_state = SwapchainState::OutOfDate {
            buffered_frames: NonZeroU32::new(buffered_frames).unwrap(),
            size,
        };
    }

    fn update_swapchain(
        &mut self,
        framebuffer_size: PhysicalSize<u32>,
        buffered_frames: u32,
    ) -> Result<()> {
        if let Some(mut swapchain) = self.swapchain.take() {
            swapchain.destroy(
                &self.vk,
                self.present_queue,
            );
        }
        unsafe {
            self.alloc.clear();
        }
        self.swapchain = Some(Swapchain::new(
            &self.vk,
            self.handle,
            vk::Extent2D { width: framebuffer_size.width, height: framebuffer_size.height },
            buffered_frames,
            &self.alloc,
            &self.tmp_alloc,
        ).context("failed to create swapchain")?);
        self.swapchain_state = SwapchainState::Valid;
        Ok(())
    }

    pub fn get_or_init_swapchain(
        &mut self,
    ) -> Result<(&mut Swapchain, bool)> {
        let mut recreated = false;
        match self.swapchain_state {
            SwapchainState::Valid => {},
            SwapchainState::OutOfDate { buffered_frames, size, } => {
                recreated = true;
                self.update_swapchain(
                    size,
                    buffered_frames.get(),
                )?;
            },
        }
        Ok((self.swapchain.as_mut().unwrap(), recreated))
    }

    pub fn get_swapchain_context(
        &mut self
    ) -> Option<&mut Swapchain> {
        self.swapchain.as_mut()
    }

    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }

    pub fn clean_up(&mut self) {
        if let Some(mut swapchain) = self.swapchain.take() {
            swapchain.destroy(
                &self.vk,
                self.present_queue,
            );
        }
        unsafe {
            self.vk.surface_instance()
                .destroy_surface(self.handle, None);
        }
    }
}
