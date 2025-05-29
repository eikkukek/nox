use super::{
    ImageState,
    helpers::Handle,
};

use crate::{
    allocator_traits::AllocateExt,
    map_types::FixedMap,
    string::{String, SmallError},
    vec_types::{FixedVec, VecOperations}
};

use ash::vk;

use std::slice;

type UID = u64;

pub struct ImageResource<'r> {
    handle: Handle<'r, vk::Image>,
    view: Handle<'r, vk::ImageView>,
    subresource_range: vk::ImageSubresourceRange,
    state: ImageState,
    format: vk::Format,
    extent: vk::Extent2D,
}

impl<'r> ImageResource<'r> {

    pub fn new(
        handle: Handle<'r, vk::Image>,
        view: Handle<'r, vk::ImageView>,
        subresource_range: vk::ImageSubresourceRange,
        state: ImageState,
        format: vk::Format,
        extent: vk::Extent2D,
    ) -> Self {
        Self {
            handle,
            view,
            subresource_range,
            state,
            format,
            extent,
        }
    }

    pub fn handle(&self) -> Handle<'r, vk::Image> {
        Handle::new(self.handle.clone())
    }

    pub fn view(&self) -> Handle<'r, vk::ImageView> {
        Handle::new(self.view.clone())
    }

    pub fn subresource_range(&self) -> &vk::ImageSubresourceRange {
        &self.subresource_range
    }

    pub fn state(&self) -> ImageState {
        self.state
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }
}

pub struct ResourcePool<'a, 'r> {
    next_uid: UID,
    image_resources: FixedMap<'a, UID, ImageResource<'r>>,
}

impl<'a, 'r> ResourcePool<'a, 'r> {

    pub fn new<A: AllocateExt<'a>>(
        max_images: usize,
        allocator: &mut A,
    ) -> Option<Self> {
        Some(Self{
            image_resources: FixedMap::new(max_images, allocator)?,
            next_uid: 0,
        })
    }

    pub fn add_image_resource(&mut self, resource: ImageResource<'r>) -> Option<&mut Self> {
        self.image_resources.insert(self.next_uid, resource)?;
        self.next_uid += 1;
        Some(self)
    }
}

pub struct Pass<'a> {
    uid: UID,
    reads: Option<FixedVec<'a, (UID, ImageState)>>,
    writes: Option<FixedVec<'a, (UID, ImageState)>>,
    dependencies: Option<FixedVec<'a, UID>>,
    pub name: String::<64>,
}

impl<'a> Pass<'a> {
    
    pub fn new<A: AllocateExt<'a>>(
        name: String::<64>,
        uid: &mut UID,
        max_reads: usize,
        max_writes: usize,
        max_dependencies: usize,
        allocator: &mut A,
    ) -> Option<Self> {
        let cpy = *uid;
        *uid += 1;
        Some(Self {
            uid: cpy,
            reads: if max_reads != 0 { Some(FixedVec::new(max_reads, allocator)?) } else { None },
            writes: if max_writes != 0 { Some(FixedVec::new(max_writes, allocator)?) } else { None },
            dependencies: if max_dependencies != 0 { Some(FixedVec::new(max_dependencies, allocator)?) } else { None },
            name,
        })
    }

    pub fn name(&mut self, name: &str) {
        self.name = String::from_str(name);
    }

    pub fn add_read(&mut self, uid: UID, dst_image_state: ImageState) -> Option<&mut Self> {
        if let Some(reads) = &mut self.reads {
            reads.push_back((uid, dst_image_state))?;
        }
        return Some(self)
    }

    pub fn add_write(&mut self, uid: UID, dst_image_state: ImageState) -> Option<&mut Self> {
        if let Some(writes) = &mut self.writes {
            writes.push_back((uid, dst_image_state))?;
        }
        return Some(self)
    }

    pub fn add_dependency(&mut self, uid: UID) -> Option<&mut Self> {
        if let Some(dependencies) = &mut self.dependencies {
            dependencies.push_back(uid)?;
        }
        return Some(self)
    }
}

pub struct FrameGraph<'a> {
    passes: FixedMap<'a, UID, Pass<'a>>, // ordered map
    sorted: FixedVec<'a, UID>,
}

impl<'a> FrameGraph<'a> {

    pub fn new<'b, A: AllocateExt<'a>, B: AllocateExt<'b>>(
        passes: FixedMap<'a, UID, Pass<'a>>,
        allocator: &mut A,
        temp_allocator: &mut B,
    ) -> Result<Self, SmallError> {
        let Some(mut in_degree) = FixedMap::<UID, usize>::new(passes.size(), temp_allocator) else {
            return Err(String::from_str("allocation failed"))
        };
        let Some(mut dependents) = FixedMap::<UID, FixedVec<UID>>::new(passes.size(), temp_allocator) else {
            return Err(String::from_str("allocation failed"))
        };
        for (uid, pass) in passes.iter() {
            if let Some(deps) = &pass.dependencies {
                for dep in deps {
                    in_degree.insert_or_modify(*uid, || 1, |v| *v += 1);
                    let Some(dependent_list) = &mut dependents
                        .insert(
                            *dep,
                            match FixedVec::new(passes.size(), temp_allocator) {
                                Some(r) => r,
                                None => return Err(String::from_str("allocation failed"))
                        })
                        else {
                            return Err(String::from_str("allocation failed"))
                        };
                    dependent_list.push_back(*uid);
                }
            }
            
        }
        let Some(mut pending) = FixedVec::<UID>::new(passes.size(), temp_allocator) else {
            return Err(String::from_str("allocation failed"))
        };
        for (uid, deg) in in_degree.iter() {
            if *deg == 0 {
                pending.push_back(*uid);
            }
        }
        let Some(mut sorted) = FixedVec::new(passes.size(), allocator) else {
            return Err(String::from_str("allocation failed"))
        };
        while let Some(uid) = pending.pop_back() {
            let pass = passes.get(uid).expect("UID not found");
            sorted.push_back(pass.uid);
            if let Some(dependents) = dependents.get(uid) {
                for dep_uid in dependents {
                    let count = in_degree.get_mut(*dep_uid).unwrap();
                    *count -= 1;
                    if *count == 0 {
                        pending.push_back(*dep_uid);
                    }
                }
            }
        }
        if sorted.len() != passes.len() {
            Err(String::from_str("cycle detected"))
        }
        else {
            Ok(Self {
                passes,
                sorted,
            })
        }
    }

    pub fn execute(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        resource_pool: &mut ResourcePool,
        funs: FixedMap<'a, UID, fn(UID)>,
    ) {
        for uid in &self.sorted {
            let pass = self.passes.get(*uid).expect("couldn't find pass");
            if let Some(reads) = &pass.reads {
                for read in reads {
                    let image_resource = resource_pool.image_resources.get_mut(read.0).expect("couldn't find resource");
                    let memory_barrier = image_resource.state
                        .to_memory_barrier(
                            *image_resource.handle,
                            &read.1,
                            image_resource.subresource_range,
                        );
                    unsafe {
                        device.cmd_pipeline_barrier(
                            command_buffer,
                            image_resource.state.pipeline_stage,
                            read.1.pipeline_stage,
                            Default::default(),
                            Default::default(),
                            Default::default(),
                            slice::from_ref(&memory_barrier),
                        );
                    }
                }
            }
            if let Some(writes) = &pass.writes {
                for write in writes {
                    let image_resource = resource_pool.image_resources.get_mut(write.0).expect("couldn't find resource");
                    let memory_barrier = image_resource.state
                        .to_memory_barrier(
                            *image_resource.handle,
                            &write.1,
                            image_resource.subresource_range,
                        );
                    unsafe {
                        device.cmd_pipeline_barrier(
                            command_buffer,
                            image_resource.state.pipeline_stage,
                            write.1.pipeline_stage,
                            Default::default(),
                            Default::default(),
                            Default::default(),
                            slice::from_ref(&memory_barrier),
                        );
                    }
                }
            }
            if let Some(fun) = funs.get(pass.uid) {
                fun(pass.uid);
            }
        }
    }
}
