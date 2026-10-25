//! Read-only, historical operator-selected bundle check; never recovery authority.
use std::{io, path::Path};
pub const BYTES: usize = 512;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub attempted: u64,
    pub verified: u64,
    pub successful: bool,
    pub catalog_bytes: u64,
    pub object_bytes: u64,
    pub objects: u16,
}
impl Receipt {
    pub fn parse(text: &str) -> Option<Self> {
        if text.len() > BYTES || !text.is_ascii() {
            return None;
        }
        let mut lines = text.lines();
        if lines.next()? != "version=1" {
            return None;
        }
        let mut value = |prefix: &str| {
            let raw = lines.next()?.strip_prefix(prefix)?;
            if raw.is_empty() || !raw.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            raw.parse::<u64>().ok()
        };
        let attempted = value("attempted=")?;
        let verified = value("verified=")?;
        let successful = match value("result=")? {
            0 => false,
            1 => true,
            _ => return None,
        };
        let catalog_bytes = value("catalog_bytes=")?;
        let object_bytes = value("object_bytes=")?;
        let objects = u16::try_from(value("objects=")?).ok()?;
        if lines.next().is_some()
            || attempted == 0
            || catalog_bytes > 512 * 1024 * 1024
            || object_bytes > 256 * 1024 * 1024
            || objects > 2000
            || (successful && (attempted != verified || catalog_bytes == 0))
            || (verified == 0 && (catalog_bytes != 0 || object_bytes != 0 || objects != 0))
            || (verified != 0 && catalog_bytes == 0)
        {
            return None;
        }
        Some(Self {
            attempted,
            verified,
            successful,
            catalog_bytes,
            object_bytes,
            objects,
        })
    }
}
pub fn read(directory: &Path) -> io::Result<Receipt> {
    Receipt::parse(&super::read_text(directory, "backup.status", BYTES)?)
        .ok_or_else(|| io::ErrorKind::InvalidData.into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn records_attempt_and_last_success_separately_without_content() {
        let raw = "version=1\nattempted=20\nverified=10\nresult=0\ncatalog_bytes=4096\nobject_bytes=80\nobjects=1\n";
        let v = Receipt::parse(raw).unwrap();
        assert!(!v.successful);
        assert_eq!(v.verified, 10);
        assert!(Receipt::parse(&raw.replace("result=0", "result=1")).is_none());
        assert!(Receipt::parse(&raw.replace("objects=1", "objects=2001")).is_none());
        assert!(Receipt::parse(&(raw.to_owned() + "path=secret\n")).is_none());
        assert!(Receipt::parse(&raw.replace("verified=10", "verified=0")).is_none());
        // Backward wall-clock steps preserve the prior check, shown as unknown by UI.
        assert!(Receipt::parse(&raw.replace("verified=10", "verified=30")).is_some());
    }
}
