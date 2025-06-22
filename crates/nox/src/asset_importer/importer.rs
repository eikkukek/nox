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

pub struct AssetImporter<'alloc, A: AssetType, B: BinGen<A>> {
    meta: DynVec<'alloc, MetaFile<A>, GlobalAlloc>,
    _gen: PhantomData<B>,
}

impl<'alloc, A: AssetType, B: BinGen<A>> AssetImporter<'alloc, A, B> {

    pub fn new(initial_capacity: Option<usize>, allocator: &'alloc RefCell<GlobalAlloc>) -> Result<Self, CapacityError> {
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
