use std::ffi::CString;
use core::{
    marker::PhantomData,
    slice,
    hash::{Hasher, Hash},
};

use parking_lot::RwLockWriteGuard;

use ahash::AHashMap;

use compact_str::format_compact;

use nox_ash::vk;
use nox_proc::Display;
use nox_mem::{
    alloc::{StdAlloc, LocalAlloc, Layout},
    slot_map::{SlotMap, SlotIndex},
    option::OptionExt,
    align_up,
    vec::{Vec32, FixedVec32, ArrayVec, Vector, Pointer},
    vec32,
};
use nox_threads::{
    futures::{
        future::RemoteHandle,
    },
    executor::{ThreadPool, SpawnExt},
};
use nox_alloc::arena::Arena;

use crate::{
    log,
    gpu::{
        prelude::*,
        TmpAllocs,
        ext,
    },
    sync::{Arc, FutureLock, RwLock},
    dev::error::*,
};

nox_ash::ash_style_enum!(
    /// Specifies how a descriptor set *can* be used.
    #[flags(Flags32)]
    #[default = Self::empty()]
    pub enum DescriptorSetFlags {
        /// *This requires enabling the [`push_descriptor`] device extension.*
        ///
        /// Specifies that the descriptor set *must* not be allocated from a
        /// [`ShaderResourcePool`], but instead should be pushed by `cmd_push_descriptor_set` or
        /// `cmd_push_descriptor_set2` provided by [`GraphicsCommands`] and [`ComputeCommands`].
        #[display("Push Descriptor")]
        PUSH_DESCRIPTOR = 0x1,
    }
);

#[derive(Clone)]
pub(crate) struct DescriptorSetLayout {
    pub handle: vk::DescriptorSetLayout,
    pub bindings: Vec32<DescriptorSetLayoutBinding>,
    pub shader_stage_mask: vk::ShaderStageFlags,
    pub flags: DescriptorSetFlags,
}

impl DescriptorSetLayout {

    #[inline(always)]
    pub fn is_push_descriptor(&self) -> bool {
        self.flags.contains(DescriptorSetFlags::PUSH_DESCRIPTOR)
    }
}

struct ShaderSet {
    inner: FutureLock<Arc<ShaderSetInner>, RemoteHandle<Result<Arc<ShaderSetInner>>>>,
}

unsafe impl Sync for ShaderSet {}

impl ShaderSet {

    #[inline(always)]
    pub fn default_attributes() -> ShaderSetAttributes {
        ShaderSetAttributes::new()
    }

    fn new(f: RemoteHandle<Result<Arc<ShaderSetInner>>>) -> Self {
        Self {
            inner: FutureLock::new(f)
        }
    }

    #[inline(always)]
    pub(crate) fn load(&self) -> Result<&Arc<ShaderSetInner>> {
        self.inner.load()
    }
}

pub(crate) struct ShaderSetInner {
    vk: Arc<Vulkan>,
    n_descriptor_set_layouts: u32,
    descriptor_set_layouts: Pointer<DescriptorSetLayout>,
    n_push_constant_ranges: u32,
    push_constant_ranges: Pointer<PushConstantRange>,
    n_shaders: u32,
    shaders: Pointer<(ShaderSourceCompiled, CString, vk::ShaderModule)>,
    pipeline_layout: vk::PipelineLayout,
}

unsafe impl Send for ShaderSetInner {}
unsafe impl Sync for ShaderSetInner {}

impl ShaderSetInner {

    #[inline(always)]
    pub(crate) fn new(
        vk: Arc<Vulkan>,
        descriptor_set_layouts: &[DescriptorSetLayout],
        push_constant_ranges: &[PushConstantRange],
        shaders: impl ExactSizeIterator<Item = (ShaderSourceCompiled, CString, vk::ShaderModule)>,
        pipeline_layout: vk::PipelineLayout,
    ) -> Self {
        let pc_align = align_of_val(push_constant_ranges);
        let shaders_align = align_of::<(ShaderSourceCompiled, vk::ShaderModule)>();
        let align = align_of_val(descriptor_set_layouts)
            .max(pc_align)
            .max(shaders_align);
        let mut size = 0;
        let desc_off = size;
        size += size_of_val(descriptor_set_layouts);
        size = align_up(size, pc_align);
        let pc_off = size;
        size += size_of_val(push_constant_ranges);
        size = align_up(size, shaders_align);
        let shader_off = size;
        size += size_of::<(ShaderSourceCompiled, vk::ShaderModule)>() * shaders.len();
        let data = unsafe {
            StdAlloc
                .allocate_raw(Layout::from_size_align(size, align).unwrap())
                .expect("global alloc failed")
        };
        let (descriptor_set_layouts, n_descriptor_set_layouts) = unsafe {
            let ptr = Pointer::from(data.add(desc_off)).cast();
            Pointer
                ::new(descriptor_set_layouts.as_ptr().cast_mut())
                .unwrap()
                .clone_elements(ptr, descriptor_set_layouts.len());
            assert!(descriptor_set_layouts.len() <= u32::MAX as usize);
            (ptr, descriptor_set_layouts.len() as u32)
        };
        let (push_constant_ranges, n_push_constant_ranges) = unsafe {
            let ptr = Pointer::from(data.add(pc_off)).cast();
            Pointer
                ::new(push_constant_ranges.as_ptr().cast_mut())
                .unwrap()
                .clone_elements(ptr, push_constant_ranges.len());
            assert!(push_constant_ranges.len() <= u32::MAX as usize);
            (ptr, push_constant_ranges.len() as u32)
        };
        let (shaders, n_shaders) = unsafe {
            let ptr = Pointer::from(data.add(shader_off)).cast();
            let len = shaders.len();
            assert!(len <= u32::MAX as usize);
            let n_shaders = len as u32;
            for (i, shader) in shaders.enumerate() {
                ptr.add(i).write(shader);
            }
            (ptr, n_shaders)
        };
        Self {
            vk,
            n_descriptor_set_layouts,
            descriptor_set_layouts,
            n_push_constant_ranges,
            push_constant_ranges,
            n_shaders,
            shaders,
            pipeline_layout,
        }
    }

    #[inline(always)]
    pub(crate) fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }

    #[inline(always)]
    pub(crate) fn descriptor_set_layouts(
        &self,
    ) -> &[DescriptorSetLayout]
    {
        unsafe {
            slice::from_raw_parts(
                self.descriptor_set_layouts.as_ptr(),
                self.n_descriptor_set_layouts as usize,
            )
        }
    }

    #[inline(always)]
    pub(crate) fn push_constant_ranges(&self) -> &[PushConstantRange] {
        unsafe {
            slice::from_raw_parts(
                self.push_constant_ranges.as_ptr(),
                self.n_push_constant_ranges as usize,
            )
        }
    }

    #[inline(always)]
    pub(crate) fn shaders(&self) -> &[(ShaderSourceCompiled, CString, vk::ShaderModule)] {
        unsafe {
            slice::from_raw_parts(
                self.shaders.as_ptr(),
                self.n_shaders as usize
            )
        }
    }
}

impl Drop for ShaderSetInner {

    fn drop(&mut self) {
        unsafe {
            self.descriptor_set_layouts.drop_in_place(self.n_descriptor_set_layouts as usize);
            self.push_constant_ranges.drop_in_place(self.n_push_constant_ranges as usize);
            for (_, _, module) in self.shaders() {
                self.vk.device().destroy_shader_module(*module, None);
            }
            self.shaders.drop_in_place(self.n_shaders as usize);
            self.vk.device().destroy_pipeline_layout(self.pipeline_layout, None);
        }
        let descriptor_set_layouts = self.descriptor_set_layouts();
        let push_constant_ranges = self.push_constant_ranges();
        let shaders = self.shaders();
        let pc_align = align_of_val(push_constant_ranges);
        let shaders_align = align_of_val(shaders);
        let align = align_of_val(descriptor_set_layouts)
            .max(pc_align)
            .max(shaders_align);
        let mut size = 0;
        size += size_of_val(descriptor_set_layouts);
        size = align_up(size, pc_align);
        size += size_of_val(push_constant_ranges);
        size = align_up(size, shaders_align);
        size += size_of_val(shaders);
        let data = *self.descriptor_set_layouts.cast();
        unsafe {
            StdAlloc
                .free_raw(data, Layout::from_size_align(size, align).unwrap());
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)] #[display("{0}")]
pub struct ShaderSetId(pub(super) SlotIndex<ShaderSet>);

#[derive(Clone)]
pub struct ShaderSetAttributes {
    count_spec: AHashMap<(u32, u32), Vec32<SpecializationConstant<u32>>>,
    flags: AHashMap<u32, DescriptorSetFlags>,
    push_descriptor_required: bool,
}

impl ShaderSetAttributes {

    #[inline(always)]
    fn new() -> Self {
        Self {
            count_spec: AHashMap::default(),
            flags: AHashMap::default(),
            push_descriptor_required: false,
        }
    }

    #[inline(always)]
    pub fn with_count_specialization_constant(
        mut self,
        descriptor_set: u32,
        binding: u32,
        constant: SpecializationConstant<u32>,
    ) -> Self {
        self.count_spec
            .entry((descriptor_set, binding))
            .and_modify(|c| c.push(constant))
            .or_insert_with(|| vec32![constant]);
        self
    }

    #[inline(always)]
    pub fn with_descriptor_set_flags(
        mut self,
        descriptor_set: u32,
        flags: DescriptorSetFlags,
    ) -> Self {
        self.flags
            .entry(descriptor_set)
            .and_modify(|f| *f |= flags)
            .or_insert(flags);
        if flags.contains(DescriptorSetFlags::PUSH_DESCRIPTOR) {
            self.push_descriptor_required = true;
        }
        self
    }
}

#[derive(Clone)]
struct DescriptorSetLayoutKey {
    flags: DescriptorSetFlags,
    bindings: Vec32<DescriptorSetLayoutBinding>,
    hash: u64,
}

impl DescriptorSetLayoutKey {

    fn new(
        flags: DescriptorSetFlags,
        bindings: Vec32<DescriptorSetLayoutBinding>
    ) -> Self {
        let mut hasher = ahash::AHasher::default();
        flags.hash(&mut hasher);
        bindings.hash(&mut hasher);
        let hash = hasher.finish();
        Self {
            flags,
            bindings,
            hash,
        }
    }
}

impl PartialEq for DescriptorSetLayoutKey {

    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.bindings == other.bindings
    }
}

impl Eq for DescriptorSetLayoutKey {}

impl Hash for DescriptorSetLayoutKey {
    
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

pub(crate) struct ShaderCache {
    vk: Arc<Vulkan>,
    shader_sets: SlotMap<ShaderSet>,
    descriptor_set_layouts: Arc<RwLock<AHashMap<
        DescriptorSetLayoutKey,
        vk::DescriptorSetLayout
    >>>,
}

impl ShaderCache {

    #[inline(always)]
    pub(crate) fn new(vk: Arc<Vulkan>) -> Self {
        Self {
            vk,
            shader_sets: Default::default(),
            descriptor_set_layouts: Arc::new(RwLock::new(
                AHashMap::default()
            )),
        }
    } 

    #[inline(always)]
    pub fn create_shader_set<const N_SHADERS: usize>(
        &mut self,
        shaders: [Shader; N_SHADERS],
        attributes: ShaderSetAttributes,
        thread_pool: ThreadPool,
        tmp_allocs: Arc<TmpAllocs>,
    ) -> Result<ShaderSetId>
    {
        let descriptor_set_layout_cache = self.descriptor_set_layouts.clone();
        let vk = self.vk.clone();
        let mut max_push_descriptors = vk.enabled_device_extensions()
            .get_attribute(ext::push_descriptor::MAX_PUSH_DESCRIPTORS_ATTRIBUTE_NAME)
            .u32().unwrap_or(0);
        let index = self.shader_sets.insert(ShaderSet::new(
            thread_pool.spawn_with_handle(async move {
                let mut shaders_inner = ArrayVec::<_, N_SHADERS>::new();
                for shader in shaders {
                    shaders_inner.push(shader
                        .into_inner()
                        .context("failed to create shader")?
                    );
                }
                let tmp_alloc = tmp_allocs.tmp_alloc();
                let tmp_alloc = tmp_alloc.guard();
                let mut sets = Vec32::<(
                    Vec32<DescriptorSetLayoutBinding>,
                    vk::ShaderStageFlags,
                    DescriptorSetFlags,
                )>::new();
                let mut push_constant_ranges = Vec32::new();
                for shader in &mut shaders_inner {
                    for uniform in shader.uniforms() {
                        let set = uniform.set;
                        if set >= sets.len() {
                            sets.resize(set + 1, (
                                vec32![],
                                vk::ShaderStageFlags::empty(),
                                DescriptorSetFlags::empty(),
                            ));
                            unsafe {
                                let last = sets.last_mut().unwrap_unchecked();
                                last.0.reserve(4);
                                last.2 = attributes.flags.get(&set).copied().unwrap_or_default();
                            }
                        }
                        let (bindings, stage_mask, _) = unsafe {
                            sets.get_unchecked_mut(set as usize)
                        };
                        *stage_mask |= shader.stage().into();
                        bindings.push(uniform.as_layout_binding(
                            attributes.count_spec
                                .get(&(uniform.set, uniform.binding))
                                .map(|spec| spec.as_ref())
                                .unwrap_or_default()
                        ));
                    }
                    for &pc in shader.push_constant_ranges() {
                        push_constant_ranges.push(pc);
                    }
                }
                let mut descriptor_set_layouts = Vec32::with_capacity(sets.len());
                for (mut bindings, stage_mask, flags) in sets {
                    bindings.sort_unstable_by(|a, b| a.binding.cmp(&b.binding));
                    let mut layout_flags = vk::DescriptorSetLayoutCreateFlags::empty();
                    if flags.contains(DescriptorSetFlags::PUSH_DESCRIPTOR)
                    {
                        layout_flags |= vk::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR_KHR;
                        let binding_count: u32 = bindings
                            .iter()
                            .map(|binding| binding.descriptor_count)
                            .sum();
                        if binding_count > max_push_descriptors {
                            return Err(Error::just_context(format_compact!(
                                "descriptor with push descriptor flag has {} descriptors when max push descriptors count is {}",
                                binding_count, max_push_descriptors,
                            )))
                        }
                    }
                    let key = DescriptorSetLayoutKey::new(flags, bindings);
                    let mut cache = descriptor_set_layout_cache.write();
                    let handle = cache
                        .get(&key)
                        .copied()
                        .unwrap_or_try_else(|| {
                            let binding_count = key.bindings.len() as u32;
                            let create_info = vk::DescriptorSetLayoutCreateInfo {
                                s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
                                flags: layout_flags,
                                binding_count,
                                p_bindings: key.bindings.as_ptr() as _,
                                ..Default::default()
                            };
                            let handle = unsafe {
                                vk.device()
                                    .create_descriptor_set_layout(&create_info, None)
                            }.context("failed to create descriptor set layout")?;
                            cache.insert(key.clone(), handle);
                            Ok(handle)
                        })?;
                    descriptor_set_layouts.push(DescriptorSetLayout {
                        handle,
                        bindings: key.bindings,
                        shader_stage_mask: stage_mask,
                        flags,
                    });
                }
                let mut vk_set_layouts = FixedVec32
                    ::with_capacity(descriptor_set_layouts.len(), &tmp_alloc)?;
                vk_set_layouts.extend(descriptor_set_layouts
                    .iter()
                    .map(|layout| layout.handle)
                );
                let mut vk_push_constants = FixedVec32
                    ::with_capacity(push_constant_ranges.len(), &tmp_alloc)?;
                vk_push_constants.extend(push_constant_ranges
                    .iter()
                    .map(|&r| r.into())
                );
                let create_info = vk::PipelineLayoutCreateInfo {
                    s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
                    set_layout_count: vk_set_layouts.len() as u32,
                    p_set_layouts: vk_set_layouts.as_ptr(),
                    push_constant_range_count: vk_push_constants.len() as u32,
                    p_push_constant_ranges: vk_push_constants.as_ptr(),
                    ..Default::default()
                };
                let pipeline_layout = unsafe {
                    vk.device()
                        .create_pipeline_layout(&create_info, None)
                }.context("failed to create pipeline layout")?;
                let mut shader_modules = ArrayVec::<_, N_SHADERS>::new();
                shader_modules.try_extend(
                    shaders_inner.iter_mut().map(|shader| {
                        let info = vk::ShaderModuleCreateInfo {
                            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
                            code_size: size_of_val(shader.source().spirv()),
                            p_code: shader.source().spirv().as_ptr(),
                            ..Default::default()
                        };
                        Ok(unsafe { RaiiHandle::new(
                            vk.device()
                                .create_shader_module(&info, None)
                                .context("failed to create shader module")?,
                            |module| {
                                vk.device().destroy_shader_module(module, None);
                            }
                        )})
                    }),
                )?;
                Ok(Arc::new(ShaderSetInner::new(
                    vk.clone(),
                    &descriptor_set_layouts,
                    &push_constant_ranges,
                    shader_modules
                        .into_iter()
                        .enumerate()
                        .map(|(i, s)| {
                            let shader = &shaders_inner[i];
                            (
                                shader.source().clone(),
                                shader.entry_point().into(), s.into_inner()
                            )
                        }),
                    pipeline_layout,
                )))
                }).context("failed to spawn")?
        ));
        Ok(ShaderSetId(index))
    }

    #[inline(always)]
    pub fn delete_shader_set(&mut self, id: ShaderSetId) {
        self.shader_sets.remove(id.0).ok();
    }

    #[inline(always)]
    pub fn get_shader_set(&self, id: ShaderSetId) -> Result<&Arc<ShaderSetInner>> {
        self.shader_sets
            .get(id.0)
            .context_with(|| format_compact!(
                "invalid shader set id {id}",
            ))?.load()
    }
}

impl Drop for ShaderCache {

    fn drop(&mut self) {
        let device = self.vk.device();
        self.shader_sets.clear(); 
        for &layout in self.descriptor_set_layouts.read().values() {
            unsafe {
                device.destroy_descriptor_set_layout(layout, None);
            }
        }
    }
}

#[must_use]
#[derive(Clone, Copy)]
pub struct ShaderDescriptorSetInfo {
    pub shader_set_id: ShaderSetId,
    pub descriptor_set_index: u32,
}

impl ShaderDescriptorSetInfo {
    
    #[inline(always)]
    pub fn new(
        shader_set_id: ShaderSetId,
        descriptor_set_index: u32
    ) -> Self {
        Self {
            shader_set_id,
            descriptor_set_index,
        }
    }
}
