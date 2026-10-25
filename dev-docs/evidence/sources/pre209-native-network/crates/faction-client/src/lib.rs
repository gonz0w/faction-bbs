//! Member-device storage. Remote terminal streams cannot unlock or mutate it.
#[cfg(all(test, feature = "native-mail"))]
mod checkpoint_tests;
#[cfg(feature = "native-mail")]
pub mod mail_checkpoint;
pub mod mail_data;
pub mod mail_vault;
