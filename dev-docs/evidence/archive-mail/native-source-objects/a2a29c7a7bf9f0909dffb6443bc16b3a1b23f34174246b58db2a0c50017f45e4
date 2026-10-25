//! Local file catalog UI, independent of the selected byte-transfer adapter.
use super::*;
impl Client {
    pub(super) fn file_lobby(&mut self, b: &mut Board) -> faction_core::Result<()> {
        self.approved(b)?;
        if !self.files_enabled {
            return Err(Error::Denied);
        }
        self.queue("Local file areas. Transfers use the separate restricted SFTP service.\nCommands: areas [after-id], list AREA [after-object], search AREA AFTER|- TEXT, info OBJECT, link OBJECT, cancel TRANSFER_ID,\n upload AREA NAME BYTES DESCRIPTION, download OBJECT, back.\n");
        if self.sysop_display {
            self.queue("Sysop: create SLUG members|private QUOTA TITLE, allow|deny AREA HANDLE,\n review AREA [after-object], publish OBJECT, remove OBJECT.\n");
        }
        self.prompt(State::Files, "files> ");
        // This explicit presentation hint grants no file/member authority.
        // Ordinary dumb terminals still receive no protocol escape bytes.
        if self.ansi || self.native_text_hint {
            self.native_frame(&["ready"])?;
        }
        Ok(())
    }
    pub(super) fn file_line(
        &mut self,
        state: State,
        text: &str,
        b: &mut Board,
        theme: &Theme,
    ) -> faction_core::Result<()> {
        self.approved(b)?;
        if !self.files_enabled {
            return Err(Error::Denied);
        }
        match state {
            State::FileUploadKey {
                area,
                name,
                description,
                bytes,
            } => {
                let id = b.file_upload(self.session()?, area, &name, &description, bytes, text)?;
                self.queue(&format!("Upload grant {id}. Use SFTP with this key's private half within five minutes.\nUpload only {name}; incomplete files remain private and charged until cleanup.\n"));
            }
            State::FileDownloadKey(object) => {
                let id = b.file_download(self.session()?, &object, text)?;
                self.queue(&format!("Download grant {id}. Use SFTP with the matching private key within five minutes.\n"));
            }
            State::Files => {
                let mut words = text.split_whitespace();
                let cmd = words.next().unwrap_or("");
                let args = words.collect::<Vec<_>>();
                let number = |i: usize| -> faction_core::Result<i64> {
                    args.get(i)
                        .ok_or(Error::Invalid("missing argument"))?
                        .parse()
                        .map_err(|_| Error::Invalid("invalid numeric argument"))
                };
                match cmd {
                    "native-v1" if args.len() == 2 => {
                        return self.native_enter(b, args[0], args[1]);
                    }
                    "back" => return self.menu(theme, b),
                    "help" | "?" => return self.file_lobby(b),
                    "areas" if args.len() <= 1 => {
                        let rows = b.file_areas(
                            self.session()?,
                            if args.is_empty() { 0 } else { number(0)? },
                        )?;
                        for a in &rows {
                            self.queue(&format!(
                                "{}  {} - {} [{}; quota {} bytes]\n",
                                a.id,
                                Self::safe(&a.slug),
                                Self::safe(&a.title),
                                if a.private { "private" } else { "members" },
                                a.quota
                            ));
                        }
                        if let Some(a) = rows.last() {
                            self.queue(&format!("Next page: areas {}\n", a.id));
                        }
                    }
                    "list" | "search" | "review" if !args.is_empty() => {
                        let area = number(0)?;
                        let after = if cmd != "search" {
                            args.get(1).copied().unwrap_or("")
                        } else {
                            args.get(1).filter(|s| **s != "-").copied().unwrap_or("")
                        };
                        let search = if cmd == "search" {
                            args.get(2..).unwrap_or_default().join(" ")
                        } else {
                            String::new()
                        };
                        let rows =
                            b.file_list(self.session()?, area, after, &search, cmd == "review")?;
                        for f in &rows {
                            let brief = f.description.chars().take(80).collect::<String>();
                            self.queue(&format!(
                                "{} {} ({} bytes; {}; by {})\n {}\n",
                                f.id,
                                Self::safe(&f.filename),
                                f.bytes,
                                f.state,
                                Self::safe(&f.uploader),
                                Self::safe(&brief)
                            ));
                        }
                        if let Some(f) = rows.last() {
                            self.queue(&format!(
                                "Next page: {} {} {}\n",
                                cmd,
                                area,
                                if cmd == "search" {
                                    format!("{} {}", f.id, search)
                                } else {
                                    f.id.clone()
                                }
                            ));
                        }
                    }
                    "cancel" if args.len() == 1 => {
                        b.file_abandon(self.session()?, args[0])?;
                        self.queue("Transfer cancelled. Incomplete uploads remain private and charged for sysop cleanup.\n");
                    }
                    "link" if args.len() == 1 => {
                        let reference = b.file_reference(self.session()?, args[0])?;
                        self.queue(&format!("{reference}\nPaste into a room post or chat; each reader still needs file access.\n"));
                    }
                    "info" if args.len() == 1 => {
                        let f = if args[0].starts_with("faction-file:") {
                            b.file_resolve(self.session()?, args[0])?
                        } else {
                            b.file_info(self.session()?, args[0])?
                        };
                        self.queue(&format!(
                            "{} {}\n{} bytes; SHA256 {}\n{}\n",
                            f.id,
                            Self::safe(&f.filename),
                            f.bytes,
                            f.sha256,
                            Self::safe(&f.description)
                        ));
                    }
                    "upload" if args.len() >= 4 => {
                        let area = number(0)?;
                        let bytes = args[2]
                            .parse()
                            .map_err(|_| Error::Invalid("invalid byte count"))?;
                        self.prompt(
                            State::FileUploadKey {
                                area,
                                name: args[1].into(),
                                description: args[3..].join(" "),
                                bytes,
                            },
                            "Transfer Ed25519 public key (one line): ",
                        );
                        return Ok(());
                    }
                    "download" if args.len() == 1 => {
                        let object = if args[0].starts_with("faction-file:") {
                            b.file_resolve(self.session()?, args[0])?.id
                        } else {
                            faction_core::file_id(args[0])?;
                            args[0].into()
                        };
                        self.prompt(
                            State::FileDownloadKey(object),
                            "Transfer Ed25519 public key (one line): ",
                        );
                        return Ok(());
                    }
                    "create" if args.len() >= 4 => {
                        let private = match args[1] {
                            "members" => false,
                            "private" => true,
                            _ => {
                                return Err(Error::Invalid("area mode must be members or private"));
                            }
                        };
                        let quota = args[2]
                            .parse()
                            .map_err(|_| Error::Invalid("invalid quota"))?;
                        let area = b.file_create_area(
                            self.session()?,
                            args[0],
                            &args[3..].join(" "),
                            private,
                            quota,
                        )?;
                        self.queue(&format!("Created file area {area}.\n"));
                    }
                    "allow" | "deny" if args.len() == 2 => {
                        b.file_allow(self.session()?, number(0)?, args[1], cmd == "allow")?;
                        self.queue("File area grant updated.\n");
                    }
                    "publish" | "remove" if args.len() == 1 => {
                        b.file_review(self.session()?, args[0], cmd == "publish")?;
                        self.queue(if cmd == "publish" {
                            "File published.\n"
                        } else {
                            "File hidden; owner file GC reclaims its charged bytes.\n"
                        });
                    }
                    _ => self.queue("Unknown file command or arguments. Type help.\n"),
                }
            }
            _ => return Err(Error::Denied),
        }
        self.prompt(State::Files, "files> ");
        Ok(())
    }
}
