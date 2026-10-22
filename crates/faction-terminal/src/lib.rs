//! Bounded terminal presentation; actions carry no authorization.
#![forbid(unsafe_code)]
mod art;
mod editor;
mod keys;
pub use editor::{LINE_BYTES, LineEditor, LineEvent};
mod theme;
pub use art::Art;
pub use keys::{Decoder, Key};
pub use theme::{Action, Entry, Menu, Style, Theme};
pub type Result<T> = std::result::Result<T, &'static str>;

mod reader;
pub use reader::{MESSAGE_BYTES, Message, MessageDetails, MessagePage, section};
