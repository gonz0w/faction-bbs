//! Connection-bound file control for the local terminal client, no extra listener.
use super::*;
pub(super) struct Native {
    key: String,
    nonce: String,
    sequence: u32,
}
impl Client {
    pub(super) fn native_frame(&mut self, fields: &[&str]) -> faction_core::Result<()> {
        self.queue(&faction_files::native::frame(fields).map_err(Error::Io)?);
        Ok(())
    }
    pub(super) fn native_enter(
        &mut self,
        b: &mut Board,
        key: &str,
        nonce: &str,
    ) -> faction_core::Result<()> {
        self.approved(b)?;
        if !self.files_enabled || !faction_files::id(nonce) {
            return Err(Error::Denied);
        }
        let key = faction_core::file_key(&format!("ssh-ed25519 {key}"))?;
        let board = b.file_namespace(self.session()?)?;
        self.native = Some(Native {
            key,
            nonce: nonce.into(),
            sequence: 0,
        });
        self.state = State::NativeFiles;
        self.native_frame(&["hello", nonce, &board])
    }
    pub(super) fn native_line(
        &mut self,
        line: &str,
        b: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(b)?;
        let (nonce, seq, fields) = faction_files::native::request(line).map_err(Error::Io)?;
        let session = self.native.as_mut().ok_or(Error::Denied)?;
        if nonce != session.nonce || session.sequence.checked_add(1) != Some(seq) {
            return Err(Error::Denied);
        }
        session.sequence = seq;
        let key = session.key.clone();
        let f = fields.iter().map(String::as_str).collect::<Vec<_>>();
        if f.as_slice() == ["back"] {
            self.native_frame(&["bye", nonce, &seq.to_string()])?;
            self.native = None;
            return self.menu(theme, b);
        }
        let result: faction_core::Result<Vec<Vec<String>>> = (|| {
            let num = |s: &str| {
                s.parse()
                    .map_err(|_| Error::Invalid("invalid numeric field"))
            };
            let mut rows = Vec::new();
            match f.as_slice() {
                ["areas", after] => {
                    for a in b.file_areas(self.session()?, num(after)?)? {
                        rows.push(vec![
                            "area".into(),
                            a.id.to_string(),
                            a.slug,
                            a.title,
                            a.private.to_string(),
                            a.quota.to_string(),
                        ]);
                    }
                }
                ["list", area, after] => {
                    for a in b.file_list(self.session()?, num(area)?, after, "", false)? {
                        rows.push(vec![
                            "file".into(),
                            a.id,
                            a.filename,
                            a.bytes.to_string(),
                            a.sha256,
                            a.uploader,
                            a.description.chars().take(80).collect(),
                        ]);
                    }
                }
                ["cancel", id] => {
                    b.file_abandon(self.session()?, id)?;
                    rows.push(vec!["cancelled".into()]);
                }
                ["info", reference] => {
                    let a = b.file_resolve(self.session()?, reference)?;
                    rows.push(vec![
                        "info".into(),
                        a.id,
                        a.filename,
                        a.bytes.to_string(),
                        a.sha256,
                    ]);
                }
                ["upload", area, name, bytes, description] => {
                    if description.len() > 256 {
                        return Err(Error::Denied);
                    }
                    let bytes: u32 = bytes.parse().map_err(|_| Error::Denied)?;
                    let grant = b.file_upload(
                        self.session()?,
                        num(area)?,
                        name,
                        description,
                        bytes,
                        &format!("ssh-ed25519 {key}"),
                    )?;
                    rows.push(vec![
                        "grant".into(),
                        "upload".into(),
                        (*name).into(),
                        bytes.to_string(),
                        String::new(),
                        grant,
                    ]);
                }
                ["download", reference] => {
                    let a = b.file_resolve(self.session()?, reference)?;
                    let grant =
                        b.file_download(self.session()?, &a.id, &format!("ssh-ed25519 {key}"))?;
                    rows.push(vec![
                        "grant".into(),
                        "download".into(),
                        a.filename,
                        a.bytes.to_string(),
                        a.sha256,
                        grant,
                    ]);
                }
                _ => return Err(Error::Denied),
            }
            Ok(rows)
        })();
        match result {
            Ok(rows) => {
                for mut row in rows {
                    row.insert(0, seq.to_string());
                    row.insert(0, nonce.into());
                    row.insert(0, "row".into());
                    self.native_frame(&row.iter().map(String::as_str).collect::<Vec<_>>())?;
                }
            }
            Err(_) => {
                self.native_frame(&["error", nonce, &seq.to_string(), "file operation denied"])?
            }
        }
        self.native_frame(&["end", nonce, &seq.to_string()])?;
        self.state = State::NativeFiles;
        Ok(())
    }
}
