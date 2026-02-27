use std::sync::Arc;

use core::num::NonZeroU32;

use winit::dpi::PhysicalSize;
use raw_window_handle::{HasWindowHandle, HasDisplayHandle};

use compact_str::format_compact;

use nox_ash::vk;
use nox_mem::{
    conditional::True,
    vec::{NonNullVec32, Vector},
};

use nox_alloc::arena::RwArena;

use crate::{
    dev::error::*,
    win::WinitWindow,
};

use super::{
    commands::scheduler::cmd_pipeline_barrier,
    *
};

#[derive(Clone, Copy)]
enum SwapchainState {
    Valid,
    OutOfDate {
        buffered_frames: NonZeroU32,
        size: PhysicalSize<u32>,
    },
}

pub struct AcquireImageData {
    pub image: ImageId,
    pub semaphore: vk::Semaphore,
    pub recreated: bool,
}

pub(crate) struct Surface {
    gpu: Gpu,
    window: Arc<WinitWindow>,
    id: SurfaceId,
    handle: vk::SurfaceKHR,
    swapchain_state: SwapchainState,
    swapchain: Option<Swapchain>,
    real_buffered_frames: u32,
    buffered_frames: u32,
    image_ids: NonNullVec32<'static, ImageId>,
    frame_index: u32,
    image_index: u32,
    alloc: RwArena,
    tmp_alloc: RwArena<True>,
    present_queue: vk::Queue,
    present_queue_family_index: u32,
    destroyed: bool,
}

impl Surface {

    pub fn new(
        window: Arc<WinitWindow>,
        gpu: Gpu,
        buffered_frames: u32,
        layout: MemoryLayout
    ) -> Result<Self>
    {
        let alloc = RwArena::new(layout.swapchain_size())?;
        let tmp_alloc = RwArena::with_fallback(layout.tmp_arena_size())?;
        let handle = unsafe {
            ash_window
            ::create_surface(
                gpu.vk().entry(),
                gpu.vk().instance(),
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None
            )
            .context("failed to create vulkan surface")?
        };

        let properties = unsafe { gpu.vk().instance()
            .get_physical_device_queue_family_properties(gpu.vk().physical_device())
        };

        let mut queue_family_index = None;

        for (i, _) in properties.iter().enumerate() {
            let present_supported = unsafe {
                gpu.vk().surface_instance()
                    .get_physical_device_surface_support(gpu.vk().physical_device(), i as u32, handle)
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
        let present_queue = unsafe { gpu.vk().device().get_device_queue(queue_family_index, 0) };
        Ok(Self {
            gpu,
            window,
            id: Default::default(),
            handle,
            swapchain: None,
            swapchain_state: SwapchainState::OutOfDate {
                buffered_frames: NonZeroU32::new(buffered_frames).unwrap(),
                size: window.inner_size(),
            },
            present_queue,
            present_queue_family_index: queue_family_index,
            image_ids: Default::default(),
            frame_index: 0,
            image_index: 0,
            alloc,
            tmp_alloc,
            destroyed: false,
            real_buffered_frames: 0,
            buffered_frames: 0,
        })
    }

    #[inline(always)]
    pub(super) unsafe fn set_id(&mut self, id: SurfaceId) {
        self.id = id;
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
        mut buffered_frames: u32,
    ) -> Result<&mut Swapchain> {
        if let Some(mut swapchain) = self.swapchain.take() {
            swapchain.destroy(
                &self.gpu.vk(),
                self.present_queue,
            );
        }
        self.gpu.destroy_resources(&[], &self.image_ids)?;
        unsafe {
            self.alloc.clear();
        }
        self.buffered_frames = buffered_frames;
        let swapchain = self.swapchain.insert(Swapchain::new(
            self.gpu.vk(),
            self.handle,
            vk::Extent2D { width: framebuffer_size.width, height: framebuffer_size.height },
            &mut buffered_frames,
            &self.alloc,
            &self.tmp_alloc,
        ).context("failed to create swapchain")?);
        self.swapchain_state = SwapchainState::Valid;
        self.real_buffered_frames = buffered_frames;
        let images = swapchain.images();
        let n_images = images.handles.len() as u32;
        self.image_ids = NonNullVec32::with_capacity(
            n_images,
            &self.alloc
        )?.into_static();
        self.image_ids.resize(n_images, Default::default());
        unsafe {
            self.gpu.create_swapchain_images(
                &images,
                &mut self.image_ids,
                &self.alloc
            ).context("failed to create swapchain images")?;
        }
        Ok(swapchain)
    }

    pub fn acquire_next_image(
        &mut self,
    ) -> Result<AcquireImageData> {
        let mut recreated = false;
        let swapchain = match self.swapchain_state {
            SwapchainState::Valid => unsafe { self.swapchain.as_mut().unwrap_unchecked() },
            SwapchainState::OutOfDate { buffered_frames, size, } => {
                recreated = true;
                self.update_swapchain(
                    size,
                    buffered_frames.get(),
                )?
            },
        };
        unsafe {
            let Some(data) = swapchain.acquire_next_image(
                self.gpu.vk(), self.frame_index,
                ).context("failed to acquire next swapchain image")? else
            {
                self.request_swapchain_update(
                    self.buffered_frames,
                    self.window.inner_size(),
                );
                return self.acquire_next_image()
            };
            if data.suboptimal {
                self.request_swapchain_update(
                    self.buffered_frames,
                    self.window.inner_size()
                );
            }
            self.image_index = data.image_index;
            Ok(AcquireImageData {
                image: self.image_ids[data.image_index as usize],
                semaphore: data.acquire_image_semaphore,
                recreated 
            })
        }
    }

    pub fn record_present(
        &mut self,
        recorder: &mut CommandRecorder<'_>,
        command_buffer: vk::CommandBuffer,
    ) -> Result<()> {
        let Some(swapchain) = self.swapchain else {
            return Ok(())
        };
        let image = recorder
            .register_image(self.image_ids[self.image_index as usize])
            .context("failed to get swapchain image")?;
        let wait_stage_mask = image.get_states(
            ImageAspect::COLOR,
            0,
        ).unwrap()[0].state.stage_mask;
        let barriers = image.memory_barrier(
            ImageSubresourceState {
                stage_mask: vk::PipelineStageFlags2::NONE,
                access_mask: vk::AccessFlags2::NONE,
                layout: vk::ImageLayout::PRESENT_SRC_KHR,
                queue_family_index: self.present_queue_family_index,
                command_index: COMMAND_INDEX_IGNORED,
                command_timeline_value: 0,
            },
            None,
            true,
            &mut unsafe { recorder.cache().as_mut() }.graphics_command_cache.image_memory_barrier_cache,
        ).context("swapchain image memory barrier failed")?;
        let tmp_alloc = recorder.gpu().tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        unsafe {
            cmd_pipeline_barrier(
                recorder.gpu().vk(),
                command_buffer,
                &[],
                &[(image.handle(), barriers)],
                COMMAND_INDEX_IGNORED,
                0,
                &tmp_alloc,
            ).context("swapchain image memory barrier failed")?;
        };
        unsafe { recorder.cache().as_mut().present_submits
            .add_swapchain(self.swapchain.map(|s| s.handle()).unwrap_unchecked(), self.image_index);
        }
        Ok(())
    }

    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }

    pub fn clean_up(&mut self) {
        if let Some(mut swapchain) = self.swapchain.take() {
            swapchain.destroy(
                &self.gpu.vk(),
                self.present_queue,
            );
        }
        self.gpu.destroy_resources(&[], &self.image_ids);
        unsafe {
            self.gpu.vk().surface_instance()
                .destroy_surface(self.handle, None);
        }
    }
}
