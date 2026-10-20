//! Bounded terminal presentation; actions carry no authorization.
#![forbid(unsafe_code)]
mod art;
mod keys;
mod theme;
pub use art::Art;
pub use keys::{Decoder, Key};
pub use theme::{Action, Entry, Menu, Style, Theme};
pub type Result<T> = std::result::Result<T, &'static str>;
