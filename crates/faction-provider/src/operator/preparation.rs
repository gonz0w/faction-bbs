//! Check a complete independently approved proposal. No retained authority,
//! private-key loading, listener or membership mutation is available here.
mod output;
use super::{
    input::{self, Config},
    mapped, same_home,
};
use faction_provider::preparation::{Card, MAX_CARD, MAX_PEM, certificate_sha256};
use std::{
    ffi::OsString,
    io::{self, Write},
    path::Path,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Error {
    Invalid,
    BeforePublication,
    PublicationUncertain,
}
pub(super) fn execute(args: &[OsString]) -> std::result::Result<(), Error> {
    if args.len() != 5 {
        return Err(Error::Invalid);
    }
    let checked = (|| {
        let config = Config::load(Path::new(&args[0]))?;
        let card_path = Path::new(&args[1]);
        let cert_path = Path::new(&args[2]);
        let proposal_path = Path::new(&args[3]);
        let output_path = Path::new(&args[4]);
        // A selected path cannot turn this command into a retained-state reader
        // or write an unrelated entry into either live storage namespace.
        for path in [card_path, cert_path, proposal_path, output_path] {
            if path.starts_with(&config.policy)
                || path.starts_with(&config.blobs)
                || path == config.key
            {
                return Err(());
            }
        }
        let card = mapped(Card::decode(&input::private_file(
            card_path,
            &config.root,
            MAX_CARD as u64,
        )?))?;
        let digest = mapped(certificate_sha256(&input::private_file(
            cert_path,
            &config.root,
            MAX_PEM as u64,
        )?))?;
        let bytes = input::private_file(proposal_path, &config.root, 65_536)?;
        let registry = input::proposal_bytes(&bytes)?;
        if !same_home(&registry, &config.home) || digest != card.certificate_sha256 {
            return Err(());
        }
        let mut target = registry.bindings().iter().filter(|binding| {
            binding.principal.account == card.principal.account
                && binding.principal.device == card.principal.device
        });
        let binding = target.next().ok_or(())?;
        if target.next().is_some()
            || binding.principal != card.principal
            || binding.certificate_sha256 != digest
        {
            return Err(());
        }
        // Keep all reviewed fields and binding order. Table ordering is merely
        // canonical presentation; no authority field is inferred or defaulted.
        let table = mapped(mapped(std::str::from_utf8(&bytes))?.parse::<toml::Table>())?;
        let encoded = table.to_string().into_bytes();
        input::proposal_bytes(&encoded)?;
        let roundtrip = mapped(mapped(std::str::from_utf8(&encoded))?.parse::<toml::Table>())?;
        if roundtrip != table {
            return Err(());
        }
        Ok((config, output_path.to_path_buf(), encoded))
    })()
    .map_err(|_| Error::Invalid)?;
    output::publish(&checked.2, &checked.1, &checked.0.root)?;
    writeln!(io::stdout(), "prepared candidate; not applied or approved")
        .map_err(|_| Error::PublicationUncertain)?;
    io::stdout()
        .flush()
        .map_err(|_| Error::PublicationUncertain)
}
