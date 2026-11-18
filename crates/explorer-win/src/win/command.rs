//! `IExplorerCommand` + `IClassFactory` implementations.

use std::os::windows::ffi::OsStrExt;

use windows::core::{implement, Interface, GUID, PCWSTR, PWSTR};
use windows::Win32::Foundation::{CLASS_E_CLASSNOTAVAILABLE, E_INVALIDARG, E_NOTIMPL, E_POINTER};
use windows::Win32::System::Com::{IBindCtx, IClassFactory, IClassFactory_Impl};
use windows::Win32::UI::Shell::{
    ECF_DEFAULT, ECS_ENABLED, IEnumExplorerCommand, IExplorerCommand, IExplorerCommand_Impl,
    IShellItemArray, SHStrDupW, SIGDN_FILESYSPATH,
};
use windows_core::{BOOL, Ref};

use crate::ipc::{ShellAction, ShellOpenMessage};
use crate::win::pipe;

/// CLSID `{8F3A2B1C-6D4E-4F9A-9B2C-1E5D7A8F3C4E}`
pub const CLSID_FILEPRISM_COMMAND: GUID = GUID::from_u128(0x8F3A_2B1C_6D4E_4F9A_9B2C_1E5D_7A8F_3C4E);

#[implement(IExplorerCommand)]
pub struct FilePrismCommand {
    action: ShellAction,
}

impl FilePrismCommand {
    pub fn new(action: ShellAction) -> Self {
        Self { action }
    }

    fn title(&self) -> &'static str {
        match self.action {
            ShellAction::Open => "Open with FilePrism",
            ShellAction::Convert => "Convert with FilePrism",
            ShellAction::Compress => "Compress with FilePrism",
            ShellAction::Extract => "Extract with FilePrism",
        }
    }
}

impl IExplorerCommand_Impl for FilePrismCommand_Impl {
    fn GetTitle(&self, _psiitemarray: Ref<'_, IShellItemArray>) -> windows::core::Result<PWSTR> {
        let title: Vec<u16> = self
            .title()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        unsafe { SHStrDupW(PCWSTR(title.as_ptr())) }
    }

    fn GetIcon(&self, _psiitemarray: Ref<'_, IShellItemArray>) -> windows::core::Result<PWSTR> {
        if let Some(exe) = pipe::resolve_exe_path() {
            let wide: Vec<u16> = exe
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            unsafe { SHStrDupW(PCWSTR(wide.as_ptr())) }
        } else {
            Err(E_NOTIMPL.into())
        }
    }

    fn GetToolTip(&self, _psiitemarray: Ref<'_, IShellItemArray>) -> windows::core::Result<PWSTR> {
        Err(E_NOTIMPL.into())
    }

    fn GetCanonicalName(&self) -> windows::core::Result<GUID> {
        Ok(GUID::zeroed())
    }

    fn GetState(
        &self,
        _psiitemarray: Ref<'_, IShellItemArray>,
        _foktobeslow: BOOL,
    ) -> windows::core::Result<u32> {
        Ok(ECS_ENABLED.0 as u32)
    }

    fn Invoke(
        &self,
        psiitemarray: Ref<'_, IShellItemArray>,
        _pbc: Ref<'_, IBindCtx>,
    ) -> windows::core::Result<()> {
        let Some(items) = psiitemarray.as_ref() else {
            return Ok(());
        };
        let count = unsafe { items.GetCount()? };
        let mut paths = Vec::with_capacity(count as usize);
        for idx in 0..count {
            let item = unsafe { items.GetItemAt(idx)? };
            let name = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH)? };
            let path = unsafe { name.to_string().map_err(|_| E_INVALIDARG)? };
            paths.push(path);
        }
        if paths.is_empty() {
            return Ok(());
        }
        let msg = ShellOpenMessage::new(self.action, paths);
        pipe::handoff(&msg);
        Ok(())
    }

    fn GetFlags(&self) -> windows::core::Result<u32> {
        Ok(ECF_DEFAULT.0 as u32)
    }

    fn EnumSubCommands(&self) -> windows::core::Result<IEnumExplorerCommand> {
        Err(E_NOTIMPL.into())
    }
}

#[implement(IClassFactory)]
pub struct FilePrismClassFactory;

impl IClassFactory_Impl for FilePrismClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<'_, windows_core::IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> windows::core::Result<()> {
        if ppvobject.is_null() || riid.is_null() {
            return Err(E_POINTER.into());
        }
        unsafe {
            *ppvobject = std::ptr::null_mut();
        }
        if punkouter.is_null() {
            let command: IExplorerCommand = FilePrismCommand::new(ShellAction::Open).into();
            unsafe { command.query(riid, ppvobject).ok() }
        } else {
            Err(E_INVALIDARG.into())
        }
    }

    fn LockServer(&self, _flock: BOOL) -> windows::core::Result<()> {
        Ok(())
    }
}

pub fn class_factory_for(rclsid: &GUID) -> windows::core::Result<IClassFactory> {
    if *rclsid == CLSID_FILEPRISM_COMMAND {
        Ok(FilePrismClassFactory.into())
    } else {
        Err(CLASS_E_CLASSNOTAVAILABLE.into())
    }
}
