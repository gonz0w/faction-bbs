//! Member-device storage. Remote terminal streams cannot unlock or mutate it.
#[cfg(feature = "native-mail")]
pub mod attachment_save;
#[cfg(all(test, feature = "native-mail"))]
mod checkpoint_tests;
#[cfg(feature = "native-mail")]
pub mod mail_checkpoint;
pub mod mail_data;
pub mod mail_vault;

#[cfg(feature = "network-mail")]
pub mod mail_wire;

#[cfg(all(test, feature = "native-mail"))]
mod network_checkpoint_tests;

#[cfg(feature = "network-mail")]
pub mod mail_network;

#[cfg(feature = "network-mail")]
pub mod mail_archive;
