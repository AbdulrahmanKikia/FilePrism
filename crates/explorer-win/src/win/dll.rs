//! COM DLL entry points exported to Explorer / the sparse package.

use std::sync::atomic::{AtomicU32, Ordering};

use windows::core::{Interface, GUID, HRESULT};
use windows::Win32::Foundation::{E_POINTER, HINSTANCE, S_FALSE, S_OK};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows_core::BOOL;

use super::command::class_factory_for;

static LOCK_COUNT: AtomicU32 = AtomicU32::new(0);

#[no_mangle]
pub extern "system" fn DllMain(
    _hinst: HINSTANCE,
    reason: u32,
    _reserved: *mut core::ffi::c_void,
) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH | DLL_PROCESS_DETACH => {}
        _ => {}
    }
    BOOL::from(true)
}

/// Standard COM class-object export.
#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }
    *ppv = std::ptr::null_mut();
    match class_factory_for(&*rclsid) {
        Ok(factory) => factory.query(riid, ppv),
        Err(err) => err.code(),
    }
}

#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if LOCK_COUNT.load(Ordering::SeqCst) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}
