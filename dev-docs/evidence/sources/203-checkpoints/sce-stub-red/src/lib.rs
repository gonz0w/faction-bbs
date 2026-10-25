//! Default-off experimental member-device OMEMO adapter.
#[derive(Debug)]
pub struct Error;
pub type Result<T> = std::result::Result<T, Error>;
#[cfg(feature = "native")]
pub mod sce;
