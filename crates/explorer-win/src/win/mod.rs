//! Windows COM `IExplorerCommand` DLL implementation.

mod command;
mod dll;
mod pipe;

// Ensure DLL exports are linked into the cdylib.
#[allow(unused_imports)]
pub use dll::{DllCanUnloadNow, DllGetClassObject, DllMain};
