use std::path::Path;

use crate::{serialization::{ReadLe, WriteLe}, Version};

use super::LoadError;

/// Trait for representing an asset type enum.
pub trait AssetType: Copy + WriteLe + ReadLe<Error = LoadError> {

    fn infer(path: &Path) -> Result<Self, LoadError>;

    fn version(self) -> Version;

    fn extensions(self) -> &'static [&'static str];

    fn valid_extension(self, path: &Path) -> Result<bool, LoadError> {
        let extension = path
            .extension()
            .ok_or_else(|| LoadError::InvalidPath)?
            .to_str()
            .ok_or_else(|| LoadError::InvalidUtf8)?;
        Ok(self
            .extensions()
            .contains(&extension)
        )
    }
}

#[macro_export]
macro_rules! impl_asset_type {
    ($nox:path, $type:ident, $($name:ident => $version:expr, $extensions:expr),+ $(,)?) => {

        use $nox as nox_path;

        use nox_path::asset_importer::LoadError;

        #[repr(u16)]
        #[derive(Clone, Copy)]
        pub enum $type {
            $(
                $name,
            )+
        }

        $(
            #[allow(non_upper_case_globals)]
            const $name: u16 = $type::$name as u16;
        )*

        impl nox_path::asset_importer::AssetType for $type {

            fn infer(path: &std::path::Path) -> Result<Self, LoadError> {
                let ext = path
                    .extension()
                    .ok_or_else(|| LoadError::InvalidUtf8)?
                    .to_str()
                    .ok_or_else(|| LoadError::InvalidUtf8)?;
                $ ( 
                    if Self::$name.extensions().contains(&ext) {
                        return Ok(Self::$name)
                    }
                )+
                Err(LoadError::InvalidPath)
            }

            #[inline(always)]
            #[allow(non_upper_case_globals)]
            fn version(self) -> nox_path::Version {
                match self {
                    $( Self::$name => $version ),+
                }
            }

            #[inline(always)]
            #[allow(non_upper_case_globals)]
            fn extensions(self) -> &'static [&'static str] {
                match self {
                    $( Self::$name => &$extensions ),+
                }
            }
        }

        impl nox_path::serialization::WriteLe for $type {

            #[inline(always)]
            fn write_le<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                (*self as u16).write_le(writer)
            }
        }

        impl nox_path::serialization::ReadLe for $type {

            type Error = nox_path::asset_importer::LoadError;

            #[inline(always)]
            #[allow(non_upper_case_globals)]
            fn read_le<R: std::io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
                let value = u16::read_le(reader)?;
                match value {
                    $(
                        $name => Ok(Self::$name),
                    )*
                    _ => Err(Self::Error::InvalidType),
                }
            }
        }
    };
}
