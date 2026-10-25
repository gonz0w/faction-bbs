use crate::{Error, Result};
use zeroize::Zeroizing;
pub struct Content { pub subject: Zeroizing<String>, pub body: Zeroizing<String> }
pub fn encode(_: &str, _: &str, _: u64, _: &str, _: &str) -> Result<Zeroizing<Vec<u8>>> { Err(Error) }
pub fn decode(_: &[u8], _: &str, _: &str, _: u64) -> Result<Content> { Err(Error) }
