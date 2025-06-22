use std::{
    io::BufWriter,
    path::{Path, PathBuf},
    fs::File,
    ffi::OsString,
};

use core::marker::PhantomData;

use blake3;
use memmap2::Mmap;

use crate::serialization::{ReadLe, WriteLe};

use super::{
    UUID,
    LoadError,
    AssetType,
    BinGen,
    MetaFile
};

pub struct Info<A: AssetType> {
    pub hash: blake3::Hash,
    pub uuid: UUID,
    pub asset_type: A,
}

pub struct Import<A: AssetType, B: BinGen<A>> {
    in_path: PathBuf,
    info: Option<Info<A>>,
    _gen: PhantomData<B>
}

impl<A: AssetType, B: BinGen<A>> Import<A, B> {

    pub fn new(path: &Path) -> Self {
        Self {
            in_path: PathBuf::from(path),
            info: None,
            _gen: PhantomData,
        }
    }

    pub fn info(&self) -> Option<&Info<A>> {
        self.info.as_ref()
    }

    pub fn import<F: FnMut() -> UUID>(
        &mut self,
        maybe_asset_type: Option<A>,
        mut gen_uuid: F,
        bin_gen: &mut B
    ) -> Result<MetaFile<A>, LoadError> {
        let asset_type = self.gen_asset_type(maybe_asset_type)?;
        let hash = self.gen_hash()?;
        let meta_path = path_with_extension(&self.in_path, "noxmeta")?;
        if meta_path 
            .try_exists()
            .map_err(LoadError::IoError)?
        {
            let file = &mut File::open(meta_path)?;
            let mut meta = MetaFile::<A>::read_le(file)?;
            let src_path = self.in_path
                .to_str()
                .ok_or_else(|| LoadError::InvalidUtf8)?;
            if meta.src_path != src_path || meta.old_version() || hash != meta.hash
            {
                let out_path = bin_gen.generate(asset_type, &self.in_path)?;
                meta = MetaFile::new(
                    asset_type.version(),
                    meta.uuid,
                    self.in_path.to_str().ok_or(LoadError::InvalidUtf8)?,
                    out_path.to_str().ok_or(LoadError::InvalidUtf8)?,
                    asset_type,
                    Default::default(),
                    hash,
                    0,
                );
                let writer = &mut BufWriter::new(file);
                meta.write_le(writer)?;
                self.set_info(asset_type, hash, meta.uuid);
            }
            Ok(meta)
        }
        else
        {
            let out_path = bin_gen.generate(asset_type, &self.in_path)?;
            let meta = MetaFile::new(
                asset_type.version(),
                gen_uuid(),
                self.in_path.to_str().ok_or(LoadError::InvalidUtf8)?,
                out_path.to_str().ok_or(LoadError::InvalidUtf8)?,
                asset_type,
                Default::default(),
                hash,
                0,
            );
            let mut writer = BufWriter::new(File::open(meta_path)?);
            meta.write_le(&mut writer)?;
            self.set_info(asset_type, hash, meta.uuid);
            Ok(meta)
        }
    }

    fn set_info(&mut self, asset_type: A, hash: blake3::Hash, uuid: UUID) {
        self.info = Some(Info {
            asset_type,
            hash,
            uuid,
        });
    }

    fn gen_asset_type(&self, maybe_asset_type: Option<A>) -> Result<A, LoadError> {
        let asset_type =
            if maybe_asset_type.is_some() {
                let r = unsafe { maybe_asset_type.unwrap_unchecked() };
                if !r.valid_extension(&self.in_path)?
                {
                    return Err(LoadError::InvalidPath)
                }
                r
            }
            else {
                A::infer(&self.in_path)?
            };
        Ok(asset_type)
    }

    fn gen_hash(&self) -> Result<blake3::Hash, LoadError> {
        let in_map = unsafe { Mmap::map(&File::open(&self.in_path)?)? };
        Ok(blake3::hash(&in_map))
    }
}

pub fn path_with_extension(path: &Path, ext: &str) -> Result<PathBuf, LoadError> {
    let ext_byte_len = ext.as_bytes().len() + 1;
    let mut res = PathBuf::with_capacity(path.as_os_str().len() + ext_byte_len);
    res.push(path);
    res.set_file_name({
        let file_name = path
            .file_name()
            .ok_or(LoadError::InvalidPath)?;
        let mut name = OsString::with_capacity(
            file_name.len() + ext_byte_len
        );
        name.push(file_name);
        name.push(".");
        name.push(ext);
        name
    });
    Ok(res)
}
