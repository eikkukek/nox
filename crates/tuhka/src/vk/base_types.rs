#![allow(non_camel_case_types)]

use super::*;

pub type ANativeWindow = ffi::c_void;
pub type AHardwareBuffer = ffi::c_void;
pub type CAMetalLayer = ffi::c_void;
pub type MTLDevice_id = *mut ffi::c_void;
pub type MTLCommandQueue_id = *mut ffi::c_void;
pub type MTLBuffer_id = *mut ffi::c_void;
pub type MTLTexture_id = *mut ffi::c_void;
pub type MTLSharedEvent_id = *mut ffi::c_void;
pub type IOSurfaceRef = *mut __IOSurface;
pub type SampleMask = u32;
pub type Bool32 = u32;
pub type Flags = u32;
pub type Flags64 = u64;
pub type DeviceSize = u64;
pub type DeviceAddress = u64;
pub type OHNativeWindow = ffi::c_void;
pub type OHBufferHandle = ffi::c_void;
pub type OH_NativeBuffer = ffi::c_void;
pub type NvSciSyncAttrList = *mut ffi::c_void;
pub type NvSciSyncObj = *mut ffi::c_void;
pub type NvSciBufAttrList = *mut ffi::c_void;
pub type NvSciBufObj = *mut ffi::c_void;
pub type NvSciSyncFence = [u64; 6];
pub type RemoteAddressNV = *const ffi::c_void;
