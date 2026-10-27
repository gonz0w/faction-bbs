//! Default-off experimental member-device OMEMO adapter.
//! Native calls and the bounded SCE codec exist only with `native` enabled.
#![cfg_attr(not(feature = "native"), forbid(unsafe_code))]
#[cfg(feature = "native")]
mod native;
#[cfg(feature = "native")]
pub use native::*;
#[cfg(feature = "native")]
pub mod sce;
