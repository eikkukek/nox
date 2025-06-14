use std::path::{Path, PathBuf};

use super::{LoadError, AssetType};

pub trait BinGen<A: AssetType> {

    fn generate(&mut self, asset_type: A, in_path: &Path) -> Result<PathBuf, LoadError>;
}
