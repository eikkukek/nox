#[inline]
pub const fn make_api_version(
    variant: u32,
    major: u32,
    minor: u32,
    patch: u32
) -> u32 {
    (variant << 29) | (major << 22) | (minor << 12) | patch
}

#[inline]
pub const fn api_version_variant(version: u32) -> u32 {
    version >> 29
}

#[inline]
pub const fn api_version_major(version: u32) -> u32 {
    (version >> 22) & 0x7f
}

#[inline]
pub const fn api_version_minor(version: u32) -> u32 {
    (version >> 12) & 0x3ff
}

#[inline]
pub const fn api_version_patch(version: u32) -> u32 {
    version & 0xfff
}

pub const API_VERSION_1_0: u32 = make_api_version(
    0, 1, 0, 0,
);

pub const API_VERSION_1_1: u32 = make_api_version(
    0, 1, 1, 0,
);

pub const API_VERSION_1_2: u32 = make_api_version(
    0, 1, 2, 0,
);

pub const API_VERSION_1_3: u32 = make_api_version(
    0, 1, 3, 0,
);

pub const API_VERSION_1_4: u32 = make_api_version(
    0, 1, 4, 0,
);
