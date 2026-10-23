//! Bounded terminal presentation; actions carry no authorization.
#![forbid(unsafe_code)]
mod art;
mod editor;
pub mod graphics;
mod keys;
pub use editor::{LINE_BYTES, LineEditor, LineEvent};
mod theme;
pub use art::Art;
pub use keys::{Decoder, Key};
pub use theme::{Action, Entry, Menu, Style, Theme};
pub type Result<T> = std::result::Result<T, &'static str>;

mod reader;
pub mod subject;
pub use reader::{MESSAGE_BYTES, Message, MessageDetails, MessagePage, section};

pub mod capabilities;
pub mod text;

mod login;
pub mod seasons;

pub mod directory;

pub mod rooms;

pub mod composer;
