//! Typed in-process IQ embedding after independently verified device mTLS.
//! This is not an XMPP stream/server or a carrier for ordinary PLAIN sessions.
use crate::{Error, Session, tls::Stage};
use faction_files::private_blobs::{self as blobs, Request, Store};
use faction_xmpp::{Element, blob_iq, validate_output};
/// Trusted configured local service. It cannot be selected by IQ `to`.
pub struct IqService {
    home: String,
}
impl IqService {
    pub fn new(home: &str) -> Result<Self, Error> {
        if home.len() > 189 {
            return Err(Error::Transport(Stage::Configuration));
        }
        // Reuse the exact native account/service syntax, not another JID profile.
        blob_iq::request(
            "service-check",
            &format!("policy@{home}"),
            home,
            &Request::Status { id: [1; 32] },
        )
        .map_err(|_| Error::Transport(Stage::Configuration))?;
        Ok(Self { home: home.into() })
    }
}
impl Session {
    /// Charge one shared session attempt before touching peer DOM content. The
    /// current peer certificate/registry owns the actor; `from` asserts routing.
    /// Safe valid requests receive the existing uniform failure encoding.
    pub fn dispatch_iq(
        &mut self,
        store: &mut Store,
        service: &IqService,
        iq: &Element,
    ) -> Result<Element, Error> {
        let state = self.attempt()?;
        // Bounds caller-created DOM depth/events/raw/escaped bytes before .text(),
        // base64 decoding or any unbounded serializer temporary.
        validate_output(iq).map_err(|_| Error::Transport(Stage::Frame))?;
        let route = iq.attr("from").ok_or(Error::Transport(Stage::Frame))?;
        let request = blob_iq::decode_request(iq, route, &service.home)
            .map_err(|_| Error::Transport(Stage::Frame))?;
        let result = if route.split_once('@').map(|(_, home)| home) != Some(service.home.as_str()) {
            Err(blobs::Error::Denied)
        } else {
            match self.dispatch_charged(&state, store, route, request) {
                Ok(response) => Ok(response),
                Err(Error::Store(error)) => Err(error),
                Err(error) => return Err(error),
            }
        };
        blob_iq::response(iq, &service.home, result).map_err(|_| Error::Transport(Stage::Frame))
    }
}
