use std::path::Path;

use core::marker::PhantomData;

use nox_mem::{Vector, GlobalVec, CapacityError};

use super::{
    LoadError,
    AssetType,
    BinGen,
    Import,
    MetaFile,
};

pub struct AssetImporter<A: AssetType, B: BinGen<A>> {
    meta: GlobalVec<MetaFile<A>>,
    _gen: PhantomData<B>,
}

impl<A: AssetType, B: BinGen<A>> AssetImporter<A, B> {

    pub fn new(initial_capacity: Option<usize>) -> Result<Self, CapacityError> {
        let capacity = initial_capacity.unwrap_or(128);
        Ok(Self {
            meta: GlobalVec::with_capacity(capacity)?,
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
