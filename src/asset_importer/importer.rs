use std::path::Path;

use core::{
    cell::RefCell,
    marker::PhantomData,
};

use crate::{
    global_alloc::GlobalAlloc,
    vec_types::{Vector, DynVec, CapacityError},
};

use super::{
    LoadError,
    AssetType,
    BinGen,
    Import,
    MetaFile,
};

pub struct AssetImporter<A: AssetType, B: BinGen<A>> {
    meta: DynVec<'static, MetaFile<A>, GlobalAlloc>,
    _gen: PhantomData<B>,
}

impl<A: AssetType, B: BinGen<A>> AssetImporter<A, B> {

    pub fn new(initial_capacity: Option<usize>, allocator: &'static RefCell<GlobalAlloc>) -> Result<Self, CapacityError> {
        let capacity = initial_capacity.unwrap_or(128);
        Ok(Self {
            meta: DynVec::with_capacity(capacity, allocator)?,
            _gen: PhantomData,
        })
    }

    pub fn import(
        &mut self,
        path: &Path,
        asset_type: Option<A>,
        bin_gen: &mut B,
    ) -> Result<Import<A, B>, LoadError> {
        let mut import = Import::new(path);
        let meta = import.import(asset_type, || 0, bin_gen)?;
        self.meta.push_if_unique(meta)?;
        Ok(import)
    }
}
