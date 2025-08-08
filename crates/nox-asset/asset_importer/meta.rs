use std::{
    io::{Write, Read},
};

use crate::{
    version::Version,
};

use super::{
    LoadError,
    asset_type::AssetType,
    UUID,
};

pub struct MetaFile<A: AssetType> {
    pub version: Version,
    pub uuid: u128,
    pub src_path: String,
    pub dst_path: String,
    pub asset_type: A,
    pub dependencies: Vec<String>,
    pub hash: blake3::Hash,
    pub timestamp: u64,
}

impl<A: AssetType> MetaFile<A> {

    pub fn new(
        version: Version,
        uuid: UUID,
        src_path: &str,
        dst_path: &str,
        asset_type: A,
        dependencies: Vec<String>,
        hash: blake3::Hash,
        timestamp: u64
    ) -> Self
    {
        Self {
            version,
            uuid,
            src_path: src_path.to_string(),
            dst_path: dst_path.to_string(),
            asset_type: asset_type,
            dependencies,
            hash,
            timestamp
        }
    }

    pub fn old_version(&self) -> bool {
        self.version != self.asset_type.version()
    }
}

impl<A: AssetType> WriteLe for MetaFile<A> {

    fn write_le<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.version.write_le(writer)?;
        self.uuid.write_le(writer)?;
        self.src_path.write_le(writer)?;
        self.dst_path.write_le(writer)?;
        self.asset_type.write_le(writer)?;
        self.dependencies.write_le(writer)?;
        self.hash.write_le(writer)?;
        self.timestamp.write_le(writer)?;
        Ok(())
    }
}

impl<A: AssetType> ReadLe for MetaFile<A> {

    type Error = LoadError;

    fn read_le<R: Read>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self {
            version: Version::read_le(reader)?,
            uuid: u128::read_le(reader)?,
            src_path: String::read_le(reader)?,
            dst_path: String::read_le(reader)?,
            asset_type: A::read_le(reader)?,
            dependencies: Default::default(),
            hash: blake3::Hash::read_le(reader)?,
            timestamp: u64::read_le(reader)?,
        })
    }
}

impl<A: AssetType> PartialEq for MetaFile<A> {

    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}
