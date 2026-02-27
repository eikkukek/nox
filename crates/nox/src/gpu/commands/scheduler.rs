use core::{
    ptr::NonNull,
    cell::UnsafeCell,
};

use nox_ash::vk;
use parking_lot::{RwLock, RwLockWriteGuard, RwLockReadGuard};
use compact_str::format_compact;
use ahash::{AHashMap, AHashSet};

use nox_mem::{
    alloc::{LocalAlloc, LocalAllocExt, Layout},
    slot_map::*,
    vec::{Vec32, FixedVec32, NonNullVec32, Vector},
    vec32,
    option::OptionExt,
    conditional::True,
};

use nox_proc::Display;

use nox_alloc::arena::RwArena;

use crate::sync::Arc;

use crate::{
    error,
    dev::error::*,
    gpu::{
        surface,
        prelude::*
    },
};

use super::prelude::GraphicsCommandCache;

/// An ID to a [`Command`], which can be used as a dependency for other commands.
///
/// Note that this ID is ephemeral and becomes invalid once the frame finishes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)] #[display("{0}")]
pub struct CommandId(SlotIndex<CommandFrameResources>);

impl CommandId {

    /// Gets the index part of [`CommandId`].
    #[inline(always)]
    pub fn index(self) -> u32 {
        self.0.index()
    }
}

#[derive(Clone, Copy)]
pub struct CommandDependency {
    pub dependency: CommandId,
    pub hint: MemoryDependencyHint,
}

impl CommandDependency {

    #[inline(always)]
    pub fn new(
        dependency: CommandId,
        hint: MemoryDependencyHint,
    ) -> Self {
        Self {
            dependency,
            hint,
        }
    }
}

#[derive(Clone, Copy, Debug, Display)]
pub(crate) enum CommandBufferKind {
    #[display("Graphics")]
    Graphics,
    #[display("Transfer")]
    Transfer,
    #[display("Compute")]
    Compute,
}

enum CommandInner {
    Graphics {
        fp: NonNull<dyn FnMut(&mut GraphicsCommands)-> error::Result<()> + Send + Sync>,
    },
    Transfer {
        fp: NonNull<dyn FnMut(&mut TransferCommands) -> error::Result<()> + Send + Sync>,
        staging_binder: ResourceBinder,
    },
    Compute {
        fp: NonNull<dyn FnMut(&mut ComputeCommands) -> error::Result<()> + Send + Sync>,
    },
}

pub(crate) struct SubmitInfo<'a, Alloc>
    where Alloc: LocalAlloc,
{
    queue: vk::Queue,
    wait_semaphore_infos: NonNullVec32<'a, vk::SemaphoreSubmitInfo<'static>>,
    command_buffer_infos: NonNullVec32<'a, vk::CommandBufferSubmitInfo<'static>>,
    signal_semaphore_infos: NonNullVec32<'a, vk::SemaphoreSubmitInfo<'static>>,
    alloc: &'a Alloc,
}

impl<'a, Alloc> Drop for SubmitInfo<'a, Alloc>
    where Alloc: LocalAlloc,
{

    fn drop(&mut self) {
        unsafe {
            self.wait_semaphore_infos.drop_and_free(self.alloc);
            self.command_buffer_infos.drop_and_free(self.alloc);
            self.signal_semaphore_infos.drop_and_free(self.alloc);
        }
    }
}

struct CommandPool {
    pool: vk::CommandPool,
    primaries: Vec32<vk::CommandBuffer>,
    next_primary: u32,
    secondaries: Vec32<vk::CommandBuffer>,
    next_secondary: u32,
}

impl CommandPool {

    #[inline(always)]
    fn new(
        vk: &Vulkan,
        queue_family_index: u32,
    ) -> Result<Self>
    {
        let create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            flags: vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index,
            ..Default::default()
        };
        let pool = unsafe {
            vk.device()
                .create_command_pool(&create_info, None)
                .context("failed to create command pool")?
        };
        Ok(Self {
            pool,
            primaries: vec32![],
            next_primary: 0,
            secondaries: vec32![],
            next_secondary: 0,
        })
    }

    fn allocate_primaries(
        &mut self,
        vk: &Vulkan,
        n: u32,
    ) -> Result<&[vk::CommandBuffer]> {
        let new_next_primary = self.next_primary + n;
        if new_next_primary > self.primaries.len() {
            let old_n = self.primaries.len();
            let new_n = (old_n + 1).next_power_of_two();
            self.primaries.resize(new_n, Default::default());
            let new_buffers = &mut self.primaries[old_n as usize..new_n as usize];
            let n_alloc = new_buffers.len() as u32;
            let alloc_info = vk::CommandBufferAllocateInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
                command_pool: self.pool,
                level: vk::CommandBufferLevel::PRIMARY,
                command_buffer_count: n_alloc,
                ..Default::default()
            };
            unsafe {
                vk.device().allocate_command_buffers(
                    &alloc_info,
                    new_buffers
                ).context("failed to allocate command buffers")?;
            }
        }
        let buffers = &self.primaries[self.next_primary as usize..new_next_primary as usize];
        self.next_primary = new_next_primary;
        Ok(buffers)
    }

    fn allocate_secondaries(
        &mut self,
        vk: &Vulkan,
        n: u32,
    ) -> Result<&[vk::CommandBuffer]> {
        let new_next_secondary = self.next_secondary + n;
        if new_next_secondary > self.secondaries.len() {
            let old_n = self.secondaries.len();
            let new_n = (old_n + 1).next_power_of_two();
            self.secondaries.resize(new_n, Default::default());
            let new_buffers = &mut self.secondaries[old_n as usize..new_n as usize];
            let n_alloc = new_buffers.len() as u32;
            let alloc_info = vk::CommandBufferAllocateInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
                command_pool: self.pool,
                level: vk::CommandBufferLevel::PRIMARY,
                command_buffer_count: n_alloc,
                ..Default::default()
            };
            unsafe {
                vk.device().allocate_command_buffers(
                    &alloc_info,
                    new_buffers
                ).context("failed to allocate command buffers")?;
            }
        }
        let buffers = &self.secondaries[self.next_secondary as usize..new_next_secondary as usize];
        self.next_secondary = new_next_secondary;
        Ok(buffers)
    }

    #[inline(always)]
    unsafe fn reset(
        &mut self,
        vk: &Vulkan,
    ) -> Result<()> {
        unsafe {
            vk.device().reset_command_pool(
                self.pool, vk::CommandPoolResetFlags::empty(),
            ).context("failed to reset command pool")?;
            self.next_primary = 0;
            self.next_secondary = 0;
        }
        Ok(())
    }

    #[inline(always)]
    unsafe fn destroy(&mut self, vk: &Vulkan) {
        unsafe {
            vk.device().destroy_command_pool(self.pool, None);
        }
    }
}

enum CommandPoolResetResult {
    Ready(TimelineSemaphoreId, u64),
    Pending(u64),
}

pub(crate) struct CommandPoolSet {
    gpu: Gpu,
    graphics: CommandPool,
    transfer: CommandPool,
    compute: CommandPool,
    semaphore_id: TimelineSemaphoreId,
    timeline_value: u64,
    last_reset: u64,
}

impl CommandPoolSet {

    fn new(gpu: Gpu) -> Result<Self> {
        let vk = gpu.vk();
        let indices = vk.queue_family_indices();
        let mut semaphore_id = Default::default();
        gpu.create_timeline_semaphores((0..1).map(|_| 0), |_, id| semaphore_id = id)?;
        Ok(Self {
            graphics: CommandPool::new(vk, indices.graphics_index())?,
            transfer: CommandPool::new(vk, indices.transfer_index())?,
            compute: CommandPool::new(vk, indices.compute_index())?,
            gpu,
            semaphore_id,
            timeline_value: 0,
            last_reset: 0,
        })
    }

    #[inline(always)]
    pub fn allocate_primaries(
        &mut self,
        kind: CommandBufferKind,
        n: u32,
    ) -> Result<&[vk::CommandBuffer]> {
        let vk = self.gpu.vk();
        match kind {
            CommandBufferKind::Graphics => self.graphics.allocate_primaries(vk, n),
            CommandBufferKind::Transfer => self.transfer.allocate_primaries(vk, n),
            CommandBufferKind::Compute => self.compute.allocate_primaries(vk, n),
        }.context_with(|| format_compact!("failed to allocate {kind} primary command buffers"))
    }

    #[inline(always)]
    pub fn allocate_secondaries(
        &mut self,
        kind: CommandBufferKind,
        n: u32,
    ) -> Result<&[vk::CommandBuffer]> {
        let vk = self.gpu.vk();
        match kind {
            CommandBufferKind::Graphics => self.graphics.allocate_secondaries(vk, n),
            CommandBufferKind::Transfer => self.transfer.allocate_secondaries(vk, n),
            CommandBufferKind::Compute => self.compute.allocate_secondaries(vk, n),
        }.context_with(|| format_compact!("failed to allocate {kind} secondary command buffers"))
    }

    #[inline(always)]
    pub fn reset(
        &mut self,
        current_frame: u64,
    ) -> Result<CommandPoolResetResult> {
        if self.gpu.get_semaphore_counter_value(self.semaphore_id)? >= self.timeline_value {
            let vk = self.gpu.vk();
            unsafe {
                self.graphics.reset(vk)?;
                self.transfer.reset(vk)?;
                self.compute.reset(vk)?;
            }
            self.timeline_value += 1;
            self.last_reset = current_frame;
            Ok(CommandPoolResetResult::Ready(self.semaphore_id, self.timeline_value))
        } else {
            Ok(CommandPoolResetResult::Pending(self.last_reset))
        }
    }

    #[inline(always)]
    pub fn wait_and_reset(
        &mut self,
        current_frame: u64,
    ) -> Result<(TimelineSemaphoreId, u64)> {
        if self.gpu.wait_for_semaphores(
            &[(self.semaphore_id, self.timeline_value)],
            self.gpu.vk().frame_timeout(),
        )? {
            self.timeline_value += 1;
            self.last_reset = current_frame;
            Ok((self.semaphore_id, self.timeline_value))
        } else {
            Err(Error::just_context(format_compact!(
                "frame timeout {} nanoseconds reached at {}", self.gpu.vk().frame_timeout(), location!(),
            )))
        }
    }
}

impl Drop for CommandPoolSet {

    fn drop(&mut self) {
        unsafe {
            let vk = self.gpu.vk();
            self.graphics.destroy(vk);
            self.transfer.destroy(vk);
            self.compute.destroy(vk);
            self.gpu.destroy_timeline_semaphores(&[self.semaphore_id]);
        }
    }
}

struct CommandFrameResources {
    inner: CommandInner,
    dep: NonNullVec32<'static, CommandDependency>,
    loc: Location,
}

unsafe impl Send for CommandFrameResources {}
unsafe impl Sync for CommandFrameResources {}

pub(super) struct CommandResources {
    semaphore_id: TimelineSemaphoreId,
    timeline_value: u64,
    dependencies: Vec32<CommandDependency>,
    signal_semaphores: Vec32<(TimelineSemaphoreId, u64)>,
    wait_semaphores: AHashMap<TimelineSemaphoreId, (u64, MemoryDependencyHint)>,
    wait_semaphore_cache: Vec32<TimelineSemaphoreId>,
    touches_swapchain_images: AHashSet<vk::Semaphore>,
}

impl CommandResources {

    fn new(semaphore_id: TimelineSemaphoreId) -> Self
    {
        Self {
            dependencies: vec32![],
            signal_semaphores: vec32![],
            semaphore_id,
            timeline_value: 0,
            wait_semaphores: AHashMap::default(),
            wait_semaphore_cache: vec32![],
            touches_swapchain_images: AHashSet::default(),
        }
    }

    #[inline(always)]
    pub fn semaphore_id(&self) -> TimelineSemaphoreId {
        self.semaphore_id
    }

    #[inline(always)]
    pub fn add_wait_for_semaphore(
        &mut self,
        id: TimelineSemaphoreId,
        value: u64,
        dependency_hint: MemoryDependencyHint,
    ) {
        self.wait_semaphores
            .entry(id)
            .and_modify(|(v, hint)| {
                *v = (*v).max(value);
                *hint |= dependency_hint;
            })
            .or_insert_with(|| {
                self.wait_semaphore_cache.push(id);
                (value, dependency_hint)
            });
    }

    #[inline(always)]
    pub fn finish(&mut self, timeline_value: u64) {
        self.timeline_value = timeline_value;
        self.dependencies.clear();
        self.signal_semaphores.clear();
        self.wait_semaphores.clear();
        self.wait_semaphore_cache.clear();
        self.touches_swapchain_images.clear();
    }
}

#[derive(Default)]
pub(super) struct FlushResources {
    registered_buffers: AHashSet<BufferId>,
    flush_buffers: Vec32<BufferId>,
    registered_images: AHashSet<ImageId>,
    flush_images: Vec32<ImageId>,
}

impl FlushResources {

    #[inline(always)]
    fn reset(&mut self) {
        self.registered_buffers.clear();
        self.flush_buffers.clear();
        self.registered_images.clear();
        self.flush_images.clear();
    }
}

struct Inner {
    gpu: Gpu,
    frame_semaphore: TimelineSemaphoreId,
    current_frame: u64,
    workers: Vec32<CommandPoolSet>,
    free_worker: u32,
    commands: SlotMap<CommandFrameResources>,
    stack: Arc<RwArena<True>>,
    command_resources: Vec32<CommandResources>,
    flush_resources: FlushResources,
    touched_images: AHashSet<ImageId>,
    touched_image_id_cache: Vec<ImageId>,
    touched_buffers: AHashSet<BufferId>,
    touched_buffer_id_cache: Vec<BufferId>,
}

crate::assert_send!(Inner);
crate::assert_sync!(Inner);

impl Inner {

    fn new(gpu: Gpu, num_workers: u32) -> Result<Self>
    {
        let mut frame_semaphore = Default::default();
        gpu.create_timeline_semaphores((0..1).map(|_| 0), |_, id| frame_semaphore = id)?;
        let mut workers = vec32![];
        workers.try_extend((0..num_workers).map(|_| {
            CommandPoolSet::new(gpu.clone())
        }))?;
        Ok(Self {
            frame_semaphore,
            current_frame: 0,
            workers,
            free_worker: 0,
            commands: SlotMap::with_capacity(8),
            stack: Arc::new(RwArena::with_fallback(gpu.memory_layout().tmp_arena_size())
                .context("failed to create arena alloc")?
            ),
            command_resources: vec32![],
            flush_resources: FlushResources::default(),
            gpu,
            touched_images: AHashSet::default(),
            touched_image_id_cache: vec![],
            touched_buffers: AHashSet::default(),
            touched_buffer_id_cache: vec![],
        })
    } 
}

impl Drop for Inner {

    fn drop(&mut self) {
        let semaphores: Vec32<_> = self.command_resources
            .iter()
            .map(|r| r.semaphore_id)
            .collect();
        self.gpu.destroy_timeline_semaphores(&semaphores);
    }
}

/// A handle to a command, which can be used to add dependencies to said command.
///
/// To get the ID of a command use [`Command::id`].
///
/// To create a command, you need to use the [`CommandScheduler`] provided by the event loop.
pub struct Command<'a, 'b> {
    id: CommandId,
    frame_resources: &'a mut CommandFrameResources,
    resources: &'a mut CommandResources,
}

impl<'a, 'b> Command<'a, 'b> {

    #[inline(always)]
    pub fn with_dependency(
        self,
        dependencies: CommandDependency,
    ) -> Self
    {
        self.resources.dependencies.push(dependencies);
        self
    }

    #[inline(always)]
    pub fn with_wait_semaphore(
        self,
        id: TimelineSemaphoreId,
        value: u64,
        dependency_hint: MemoryDependencyHint,
    ) -> Self 
    {
        self.resources.add_wait_for_semaphore(id, value, dependency_hint);
        self
    }

    #[inline(always)]
    pub fn with_signal_semaphore(
        self,
        id: TimelineSemaphoreId,
        value: u64,
    ) -> Self {
        self.resources.signal_semaphores.push((id, value));
        self
    }

    #[inline(always)]
    pub fn id(self) -> CommandId {
        self.id
    }
}

impl<'a, 'b> Drop for Command<'a, 'b> {

    fn drop(&mut self) {
        unsafe {
            self.frame_resources.dep =
            NonNullVec32::new(
                NonNull::new_unchecked(self.resources.dependencies.as_mut_ptr()),
                self.resources.dependencies.capacity()
            ).with_len(self.resources.dependencies.len())
        }
    }
}

pub struct CommandScheduler<'a> {
    gpu: Gpu,
    inner: UnsafeCell<RwLockWriteGuard<'a, Inner>>,
}

impl<'a> CommandScheduler<'a> {

    fn allocate_new_resources(&mut self) -> Result<()> {
        let n_new_resources = self.inner.get_mut().commands.capacity() -
            self.inner.get_mut().command_resources.len();
        let capacity = self.inner.get_mut().commands.capacity();
        self.inner.get_mut().command_resources.reserve(capacity);
        unsafe { &*self.inner.get() }.gpu.create_timeline_semaphores(
            (0..n_new_resources).map(|_| 0),
            |_, id| self.inner.get_mut().command_resources.push(CommandResources::new(id))
        )
    }

    fn make_graphics_f<F>(
        f: F
    ) -> impl FnMut(&mut GraphicsCommands) -> error::Result<()> + Send + Sync + 'static
        where F: FnOnce(&mut GraphicsCommands) -> error::Result<()> + Send + Sync + 'static
    {
        let mut f = Some(f);
        move |commands| {
            f.take().unwrap()(
                commands
            )
        }
    }

    fn make_transfer_f<F>(
        f: F
    ) -> impl FnMut(&mut TransferCommands) -> error::Result<()> + Send + Sync + 'static
        where F: FnOnce(&mut TransferCommands) -> error::Result<()> + Send + Sync + 'static
    {
        let mut f = Some(f);
        move |commands| {
            f.take().unwrap()(
                commands
            )
        }
    }

    fn make_compute_f<F>(
        f: F
    ) -> impl FnMut(&mut ComputeCommands) -> error::Result<()> + Send + Sync + 'static
        where F: FnOnce(&mut ComputeCommands) -> error::Result<()> + Send + Sync + 'static
    {
        let mut f = Some(f);
        move |commands| {
            f.take().unwrap()(
                commands
            )
        }
    }

    #[track_caller]
    pub fn new_graphics_command<F>(
        &mut self,
        f: F
    ) -> Result<Command<'_, 'a>>
        where F: FnOnce(&mut GraphicsCommands) -> error::Result<()> + Send + Sync + 'static
    {
        let fp = unsafe {
            let ptr = self.inner.get_mut().stack.allocate_uninit(1)?;
            ptr.write(Self::make_graphics_f(f));
            ptr
        };
        let idx = self.inner.get_mut().commands.insert(CommandFrameResources {
            inner: CommandInner::Graphics {
                fp,
            },
            dep: Default::default(),
            loc: caller!(),
        });
        if idx.index() >= self.inner.get_mut().command_resources.len() {
            self.allocate_new_resources()
                .context("failed to allocate command resources")?;
        }
        unsafe {
            let frame_resources = (*self.inner.get()).commands.get_mut(idx).unwrap();
            let resources = &mut (*self.inner.get()).command_resources[idx.index() as usize];
            Ok(Command { id: CommandId(idx), frame_resources, resources, })
        }
    }

    #[track_caller]
    pub fn new_transfer_command<F>(
        &mut self,
        f: F,
        staging_binder: LinearBinderId,
    ) -> Result<Command<'_, 'a>>
        where F: FnOnce(&mut TransferCommands) -> error::Result<()> + Send + Sync + 'static
    {
        let fp = unsafe {
            let ptr = self.inner.get_mut().stack.allocate_uninit(1)?;
            ptr.write(Self::make_transfer_f(f));
            ptr
        };
        let idx = self.inner.get_mut().commands.insert(CommandFrameResources {
            inner: CommandInner::Transfer {
                fp,
                staging_binder,
            },
            dep: Default::default(),
            loc: caller!(),
        });
        if idx.index() >= self.inner.get_mut().command_resources.len() {
            self.allocate_new_resources()
                .context("failed to allocate command resources")?;
        }
        unsafe {
            let frame_resources = (*self.inner.get()).commands.get_mut(idx).unwrap();
            let resources = &mut (*self.inner.get()).command_resources[idx.index() as usize];
            Ok(Command { id: CommandId(idx), frame_resources, resources, })
        }
    }

    #[track_caller]
    pub fn new_compute<F>(
        &mut self,
        f: F,
    ) -> Result<Command<'_, 'a>>
        where F: FnOnce(&mut ComputeCommands) -> error::Result<()> + Send + Sync + 'static
    {
        let fp = unsafe {
            let ptr = self.inner.get_mut().stack.allocate_uninit(1)?;
            ptr.write(Self::make_compute_f(f));
            ptr
        };
        let idx = self.inner.get_mut().commands.insert(CommandFrameResources {
            inner: CommandInner::Compute {
                fp,
            },
            dep: Default::default(),
            loc: caller!(),
        });
        if idx.index() >= self.inner.get_mut().command_resources.len() {
            self.allocate_new_resources()
                .context("failed to allocate command resources")?;
        }
        unsafe {
            let frame_resources = (*self.inner.get()).commands.get_mut(idx).unwrap();
            let resources = &mut (*self.inner.get()).command_resources[idx.index() as usize];
            Ok(Command { id: CommandId(idx), frame_resources, resources, })
        }
    }
}

#[derive(Clone)]
pub(crate) struct QueueScheduler {
    inner: Arc<RwLock<Inner>>,
}

pub struct QueueSchedulerReadLock<'a> {
    inner: RwLockReadGuard<'a, Inner>,
}

impl<'a> QueueSchedulerReadLock<'a> {

    #[inline(always)]
    pub fn get_command_semaphore_id(
        &self,
        command_index: u32
    ) -> Option<TimelineSemaphoreId> {
        self.inner.command_resources
            .get(command_index as usize)
            .map(|resources| resources.semaphore_id())
    }

    #[inline(always)]
    pub fn get_frame_semaphore_id(&self) -> TimelineSemaphoreId {
        self.inner.frame_semaphore
    }
}

impl QueueScheduler {

    #[inline(always)]
    pub fn new(
        gpu: Gpu,
        num_workers: u32,
    ) -> Result<Self>
    {
        Ok(Self {
            inner: Arc::new(RwLock::new(Inner::new(gpu, num_workers)?)),
        })
    }

    #[inline(always)]
    pub fn read(&self) -> QueueSchedulerReadLock<'_> {
        QueueSchedulerReadLock { inner: self.inner.read(), }
    }

    #[inline(always)]
    pub fn record<'a, Alloc>(
        &self,
        cache: &mut CommandRecorderCache,
        alloc: &'a Alloc,
    ) -> Result<FixedVec32<'a, SubmitInfo<'a, Alloc>, Alloc>>
        where Alloc: LocalAlloc<Error = Error>
    {
        let inner = self.inner.write();
        let resources = inner.gpu.clone();
        let mut recorder = CommandRecorder {
            surfaces: resources.write_surfaces(),
            buffers: resources.write_buffers(),
            images: resources.write_images(),
            inner,
            cache,
        };
        recorder.compile(alloc)
    }
}

pub struct PresentSubmits {
    swapchains: Vec32<(vk::SwapchainKHR, u32)>,
}

impl PresentSubmits {

    pub unsafe fn add_swapchain(
        &mut self,
        handle: vk::SwapchainKHR,
        image_index: u32,
    ) {
        self.swapchains.push((handle, image_index));
    }

    fn clear(&mut self) {
        self.swapchains.clear();
    }
}

#[derive(Default)]
pub(super) struct CommandRecorderCache {
    pub graphics_command_cache: GraphicsCommandCache,
    pub present_submits: PresentSubmits,
    acquired_images: Vec32<surface::AcquireImageData>,
    acquired_image_semaphores: AHashMap<ImageId, vk::Semaphore>,
}

pub(crate) struct CommandRecorder<'a> {
    inner: RwLockWriteGuard<'a, Inner>,
    surfaces: ResourceWriteGuard<'a, Surface, SurfaceId>,
    pub(super) buffers: ResourceWriteGuard<'a, BufferMeta, BufferId>,
    pub(super) images: ResourceWriteGuard<'a, ImageMeta, ImageId>,
    pub(super) cache: &'a mut CommandRecorderCache,
}

impl<'a> CommandRecorder<'a> {

    #[inline(always)]
    pub fn get_command_resources(
        &self,
        index: u32,
    ) -> &CommandResources
    {
        &self.inner.command_resources[index as usize]
    }

    #[inline(always)]
    pub fn get_command_resources_mut(
        &mut self,
        index: u32,
    ) -> &mut CommandResources
    {
        &mut self.inner.command_resources[index as usize]
    }

    #[inline(always)]
    pub fn gpu(&self) -> &Gpu {
        &self.inner.gpu
    }

    #[inline(always)]
    pub fn stack(&self) -> &Arc<RwArena<True>> {
        &self.inner.stack
    }

    #[inline(always)]
    pub fn cache(&mut self) -> NonNull<CommandRecorderCache> {
        NonNull::from_mut(self.cache)
    }

    #[inline(always)]
    pub fn get_current_worker(&mut self) -> &mut CommandPoolSet {
        let free_worker = self.inner.free_worker;
        &mut self.inner.workers[free_worker as usize]
    } 

    #[inline(always)]
    pub fn current_frame(&self) -> u64 {
        self.inner.current_frame
    }

    #[inline(always)]
    pub fn register_buffer(&mut self, id: BufferId) -> Result<&mut BufferMeta> {
        if self.inner.flush_resources.registered_buffers.insert(id) {
            self.inner.flush_resources.flush_buffers.push(id);
        }
        if self.inner.touched_buffers.insert(id) {
            self.inner.touched_buffer_id_cache.push(id);
        }
        self.buffers.get_mut(id)
    }

    #[inline(always)]
    pub fn register_image(
        &mut self,
        id: ImageId,
        command_index: u32,
    ) -> Result<&mut ImageMeta>
    {
        if self.inner.flush_resources.registered_images.insert(id) {
            self.inner.flush_resources.flush_images.push(id);
        }
        if self.inner.touched_images.insert(id) {
            self.inner.touched_image_id_cache.push(id);
        }
        let image = self.images.get_mut(id)?;
        if image.is_swapchain() {
            let Some(&semaphore) = self.cache.acquired_image_semaphores.get(&id) else {
                return Err(Error::just_context(format_compact!(
                    "attempting to use a swapchain image {id} that wasn't acquired this frame"
                )))
            };
            self.inner.command_resources[command_index as usize]
            .touches_swapchain_images.insert(semaphore);
        }
        Ok(image)
    }
    
    fn compile<'b, Alloc>(
        &mut self,
        alloc: &'b Alloc,
    ) -> Result<FixedVec32<'b, SubmitInfo<'b, Alloc>, Alloc>>
        where Alloc: LocalAlloc<Error = Error>,
    {
        if self.inner.commands.is_empty() {
            return Ok(FixedVec32::new(alloc))
        }
        self.cache.present_submits.clear();
        self.cache.acquired_images.clear();
        self.cache.acquired_image_semaphores.clear();
        for (id, surface) in self.surfaces.iter() {
            let image = surface
                .acquire_next_image()
                .context_with(|| format_compact!("failed to acquire surface {id} image"))?;
            self.cache.acquired_image_semaphores.insert(image.image, image.semaphore);
            self.cache.acquired_images.push(
                image,
            );
        }
        let max_index = self.inner.commands.max_index().unwrap() + 1;
        let mut in_degree = FixedVec32::with_len(max_index, 0, alloc)?;
        let mut dependents = FixedVec32::with_len_with(max_index, |_| None, alloc)?;
        let mut queue = FixedVec32::with_capacity(self.inner.commands.len(), alloc)?;
        for (idx, _) in &self.inner.commands {
            let resources = &self.inner.command_resources[idx.index() as usize];
            let in_deg = resources.dependencies.len();
            if in_deg == 0 {
                queue.push(idx);
            }
            in_degree[idx.index() as usize] = in_deg;
            for &dep in &resources.dependencies {
                let dep = &mut dependents[dep.dependency.0.index() as usize];
                let dep = match dep {
                    Some(dep) => dep,
                    None => dep.insert(FixedVec32::with_capacity(queue.capacity(), alloc)?),
                };
                dep.push(idx);
            }
        }
        let mut free_worker = None;
        let mut fallback_index = (0, u64::MAX);
        self.inner.current_frame += 1;
        let current_frame = self.inner.current_frame;
        for (i, worker) in self.inner.workers.iter_mut().enumerate() {
            let result = worker
                .reset(current_frame)
                .context("unexpected worker reset error")?;
            match result {
                CommandPoolResetResult::Ready(id, value) => {
                    free_worker = Some((i, id, value));
                    break
                },
                CommandPoolResetResult::Pending(last_reset) => if last_reset < fallback_index.1 {
                    fallback_index = (i, last_reset)
                }
            }
        }
        let &mut free_worker = free_worker.get_or_try_insert_with(|| {
            let (id, value) = self.inner.workers[fallback_index.0].wait_and_reset(current_frame)?;
            Ok((fallback_index.0, id, value))
        })?;
        self.inner.free_worker = free_worker.0 as u32;
        let mut sorted = FixedVec32::with_capacity(queue.capacity(), alloc)?;
        while let Some(node) = queue.pop() {
            sorted.push(node);
            for &dep in dependents[node.index() as usize]
                .as_deref()
                .unwrap_or_default()
            {
                let in_deg = &mut in_degree[dep.index() as usize];
                *in_deg -= 1;
                if *in_deg == 0 {
                    queue.push(dep);
                }
            }
        }
        if sorted.len() != self.inner.commands.len() {
            return Err(Error::just_context(
                "topological sort failed"
            ))
        } 
        let mut submits = FixedVec32::with_capacity(sorted.len(), alloc)?;
        let mut global_wait_semaphore_infos = NonNullVec32::with_capacity(sorted.len(), alloc)?;
        for &idx in &sorted {
            let index = idx.index() as usize;
            let id = CommandId(idx);
            let command = self.inner.commands.remove(idx).unwrap();
            let cmd_resources = &mut self.inner.command_resources[index];
            let mut timeline_value = cmd_resources.timeline_value + 1;
            cmd_resources.wait_semaphores.clear();
            for &dep in &command.dep {
                if !self.inner.commands.contains(dep.dependency.0) {
                    return Err(Error::just_context(format_compact!(
                        "command {} had a dependency with an invalid id {}",
                        command.loc, dep.dependency,
                    )))
                }
                let wait_for = &self.inner.command_resources[dep.dependency.0.index() as usize];
                let semaphore_id = wait_for.semaphore_id;
                let semaphore_value = wait_for.timeline_value;
                unsafe {
                    self.inner.command_resources.get_unchecked_mut(index)
                    .add_wait_for_semaphore(semaphore_id, semaphore_value, dep.hint);
                }
            }
            let command_result = match command.inner {
                CommandInner::Graphics { mut fp } => {
                    let (cmd_result, size, align) = {
                        let f = unsafe {
                            fp.as_mut()
                        };
                        let mut graphics_commands = unsafe { GraphicsCommands::new(
                            self,
                            CommandId(idx),
                            timeline_value,
                        ) }.context_with(|| format_compact!(
                            "failed to initialize graphics commands (command id {id})"
                        ))?;
                        f(&mut graphics_commands)
                            .context_from_tracked(|orig| format_compact!(
                                "failed to record graphics commands at {}",
                                orig.or_this()
                            ))?;
                        (graphics_commands.finish(alloc), size_of_val(f), align_of_val(f))
                    };
                    unsafe {
                        fp.drop_in_place();
                        self.inner.stack.free_raw(
                            fp.cast(),
                            Layout::from_size_align(size, align).unwrap()
                        );
                    }
                    cmd_result.context_with(|| format_compact!(
                        "failed to finish graphics commands (command id {id})"
                    ))?
                },
                _ => todo!()
            };
            timeline_value = command_result.timeline_value;
            for &id in &self.inner.flush_resources.flush_buffers {
                if let Ok(buffer) = self.buffers.get_mut(id) {
                    buffer.flush_state();
                }
            }
            for &id in &self.inner.flush_resources.flush_images {
                if let Ok(image) = self.images.get_mut(id) {
                    image.flush_subresources();
                }
            }
            self.inner.flush_resources.reset();
            let inner = &mut *self.inner;
            let command_resources = unsafe {
                inner.command_resources.get_unchecked_mut(index)
            };
            let signal = unsafe { inner.gpu
                .get_timeline_semaphore(command_resources.semaphore_id)
                .unwrap_unchecked()
            };
            let mut submit_info = SubmitInfo {
                queue: command_result.queue,
                wait_semaphore_infos: NonNullVec32::with_capacity(
                    command_resources.touches_swapchain_images.len() as u32 + command_resources.wait_semaphore_cache.len(),
                    alloc
                )?,
                command_buffer_infos: NonNullVec32::with_capacity(
                    command_result.primary_command_buffers.len(),
                    alloc
                )?,
                signal_semaphore_infos: NonNullVec32::with_capacity(
                    1 + command_resources.signal_semaphores.len(), alloc
                )?,
                alloc,
            };
            submit_info.wait_semaphore_infos.extend(command_resources.touches_swapchain_images
                .iter()
                .map(|&semaphore| {
                    vk::SemaphoreSubmitInfo {
                        semaphore,
                        stage_mask: command_result.wait_scope,
                        ..Default::default()
                    }
                })
            );
            submit_info.wait_semaphore_infos.try_extend(command_resources.wait_semaphore_cache
                .iter()
                .map(|&id| unsafe {
                    let &(value, dependency_hint) = command_resources.wait_semaphores.get(&id).unwrap_unchecked();
                    vk::SemaphoreSubmitInfo {
                        semaphore: self.gpu().get_timeline_semaphore(id)?,
                        value,
                        stage_mask: if dependency_hint.is_empty() {
                            command_result.wait_scope
                        } else {
                            dependency_hint.into()
                        },
                        ..Default::default()
                    }
                })
            )?;
            submit_info.command_buffer_infos.extend(command_result.primary_command_buffers
                .iter().map(|&command_buffer| {
                    vk::CommandBufferSubmitInfo {
                        command_buffer,
                        ..Default::default()
                    }
                })
            );
            submit_info.signal_semaphore_infos.push(vk::SemaphoreSubmitInfo {
                semaphore: signal,
                value: timeline_value,
                stage_mask: command_result.signal_scope,
                ..Default::default()
            });
            submits.push(submit_info);
            command_resources.finish(timeline_value);
            global_wait_semaphore_infos.push(vk::SemaphoreSubmitInfo {
                s_type: vk::StructureType::SEMAPHORE_SUBMIT_INFO,
                semaphore: signal,
                value: timeline_value,
                stage_mask: vk::PipelineStageFlags2::NONE,
                ..Default::default()
            });
        }
        debug_assert!(self.inner.commands.is_empty());
        let worker = &self.inner.workers[self.inner.free_worker as usize];
        let mut global_signal_semaphore_infos = NonNullVec32::with_capacity(2, alloc)?;
        global_signal_semaphore_infos.append(&[
            vk::SemaphoreSubmitInfo {
                s_type: vk::StructureType::SEMAPHORE_SUBMIT_INFO,
                semaphore: unsafe { self.inner.gpu
                    .get_timeline_semaphore(worker.semaphore_id) 
                    .unwrap_unchecked()
                },
                value: worker.timeline_value,
                stage_mask: vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
                ..Default::default()
            },
            vk::SemaphoreSubmitInfo {
                s_type: vk::StructureType::SEMAPHORE_SUBMIT_INFO,
                semaphore: unsafe { self.inner.gpu
                    .get_timeline_semaphore(self.inner.frame_semaphore) 
                    .unwrap_unchecked()
                },
                value: current_frame,
                stage_mask: vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
                ..Default::default()
            },
        ]);
        submits.push(SubmitInfo {
            queue: self.inner.gpu.vk().graphics_queue(),
            wait_semaphore_infos: global_wait_semaphore_infos,
            command_buffer_infos: Default::default(),
            signal_semaphore_infos: global_signal_semaphore_infos,
            alloc
        });
        for &id in &self.inner.touched_buffer_id_cache {
            let buffer = self.buffers.get_mut(id).unwrap();
            unsafe {
                buffer.set_last_used_frame(current_frame);
            }
        }
        for &id in &self.inner.touched_image_id_cache {
            let image = self.images.get_mut(id).unwrap();
            unsafe {
                image.set_last_used_frame(current_frame);
            }
        }
        self.inner.touched_buffers.clear();
        self.inner.touched_buffer_id_cache.clear();
        self.inner.touched_images.clear();
        self.inner.touched_image_id_cache.clear();
        unsafe {
            self.inner.stack.clear();
        }
        Ok(submits)
    }
}

pub unsafe fn cmd_pipeline_barrier<Alloc>(
    vk: &Vulkan,
    command_buffer: vk::CommandBuffer,
    buffer_barriers: &[(vk::Buffer, &[BufferMemoryBarrier])],
    image_barriers: &[(vk::Image, &[ImageMemoryBarrier])],
    command_index: u32,
    command_timeline_value: u64,
    alloc: &Alloc,
) -> Result<()>
    where Alloc: LocalAlloc<Error = Error>
{
    let mut vk_buffer_barriers = NonNullVec32::with_capacity(
        buffer_barriers.iter().map(|(_, b)| b.len()).sum::<usize>() as u32,
        alloc,
    )?;
    let mut vk_image_barriers = NonNullVec32::with_capacity(
        image_barriers.iter().map(|(_, b)| b.len()).sum::<usize>() as u32,
        alloc,
    )?;
    for &(handle, barriers) in buffer_barriers {
        for barrier in barriers {
            if barrier.src_command_index != COMMAND_INDEX_IGNORED &&
                barrier.src_command_index == command_index &&
                barrier.src_command_timeline_value == command_timeline_value 
            {
                return Err(Error::just_context(format_compact!(
                    "concurrent usage of buffer subresource where it is not allowed",
                )))
            }
            vk_buffer_barriers.push(barrier.into_vk(handle));
        }
    }
    for &(handle, barriers) in image_barriers {
        for barrier in barriers {
            if barrier.src_command_index != COMMAND_INDEX_IGNORED &&
                barrier.src_command_index == command_index &&
                barrier.src_command_timeline_value == command_timeline_value
            {
                return Err(Error::just_context(format_compact!(
                    "concurrent usage of buffer subresource where it is not allowed",
                )))
            }
            vk_image_barriers.push(barrier.into_vk(handle));
        }
    }
    let dependency_info = vk::DependencyInfo {
        s_type: vk::StructureType::DEPENDENCY_INFO,
        buffer_memory_barrier_count: vk_buffer_barriers.len(),
        p_buffer_memory_barriers: vk_buffer_barriers.as_ptr(),
        image_memory_barrier_count: vk_image_barriers.len(),
        p_image_memory_barriers: vk_image_barriers.as_ptr(),
        ..Default::default()
    };
    unsafe {
        vk.device().cmd_pipeline_barrier2(command_buffer, &dependency_info);
        vk_buffer_barriers.drop_and_free(alloc);
        vk_image_barriers.drop_and_free(alloc);
    }
    Ok(())
}
