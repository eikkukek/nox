use core::cell::RefCell;

use ash::vk;

use crate::{
    allocator_traits::{Allocate, Free}, map_types::FixedMap, stack_alloc::StackGuard, vec_types::CapacityError
};

use super::{
    handle::Handle,
    frame_graph::{Execute, FrameGraph, ImageResource, ResourcePool, UID},
    physical_device::QueueFamilyIndices,
};

pub trait Construct<'mem, 'r> {

    fn new(
        device: Handle<'r, ash::Device>,
        command_buffer: Handle<'r, vk::CommandBuffer>,
        swapchain_image_resource: RefCell::<ImageResource<'r>>,
        allocator: StackGuard<'mem>,
        temp_allocator: StackGuard<'mem>,
        queue_family_indices: QueueFamilyIndices,
        image_index: u32,
    ) -> Self;
}

pub struct Frame<'mem, 'r> {
    device: Handle<'r, ash::Device>,
    command_buffer: Handle<'r, vk::CommandBuffer>,
    swapchain_image_resource: RefCell<ImageResource<'r>>,
    allocator: RefCell<StackGuard<'mem>>,
    temp_allocator: RefCell<StackGuard<'mem>>,
    queue_family_indices: QueueFamilyIndices,
    image_index: u32,
}

impl<'mem, 'r> Frame<'mem, 'r> {

    pub fn swapchain_image_resource(&self) -> ImageResource<'r> {
        self.swapchain_image_resource.borrow().clone()
    }

    pub fn graphics_queue_family_index(&self) -> u32 {
        self.queue_family_indices.get_graphics_index()
    }

    pub fn transfer_queue_family_index(&self) -> u32 {
        self.queue_family_indices.get_transfer_index()
    }

    pub fn compute_queue_family_index(&self) -> u32 {
        self.queue_family_indices.get_compute_index()
    }

    pub fn get_allocator(&self) -> &RefCell<StackGuard<'mem>> {
        &self.allocator
    }

    pub fn get_temp_allocator(&self) -> &RefCell<StackGuard<'mem>> {
        &self.temp_allocator
    }

    pub fn image_index(&self) -> u32 {
        self.image_index
    }

    pub fn render<'s, 'a, A>(
        &self,
        frame_graph: &'s FrameGraph<'a, A>,
        resource_pool: &'s mut ResourcePool<'a, 'r, A>,
        callbacks: Option<&'s FixedMap<'a, UID, fn(UID), A>>,
    ) -> Result<(), CapacityError>
        where
            A: Allocate + Free,
            'mem: 'r,
            'r: 'a,
    {
        frame_graph.execute(
            &self.device,
            *self.command_buffer,
            resource_pool,
            &self.swapchain_image_resource,
            callbacks,
            &self.temp_allocator,
        )
    }
}

impl<'mem, 'r> Construct<'mem, 'r> for Frame<'mem, 'r> {

    fn new(
        device: Handle<'r, ash::Device>,
        command_buffer: Handle<'r, vk::CommandBuffer>,
        swapchain_image_resource: RefCell::<ImageResource<'r>>,
        allocator: StackGuard<'mem>,
        temp_allocator: StackGuard<'mem>,
        queue_family_indices: QueueFamilyIndices,
        image_index: u32,
    ) -> Self
    {
        Self {
            device,
            command_buffer,
            swapchain_image_resource,
            allocator: RefCell::new(allocator),
            temp_allocator: RefCell::new(temp_allocator),
            queue_family_indices,
            image_index,
        }
    }
}
