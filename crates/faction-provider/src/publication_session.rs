use super::*;
impl Session {
    #[cfg(test)]
    pub(crate) fn capture_publication_before_authority(
        &mut self,
        signal: std::sync::mpsc::Sender<Instant>,
    ) {
        self.publication_capture = Some(signal);
    }
    /// Capture one exact sender operation and fully release its incoming carrier.
    /// Attempt charging precedes caller data validation; the original cycle remains.
    pub fn prepare_publication(
        mut self,
        binding: faction_files::private_blobs::Binding,
        request: Request,
    ) -> Result<crate::services::PendingPublication, Error> {
        let state = self.attempt()?;
        let initial = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Store(faction_files::private_blobs::Error::Clock))?;
        #[cfg(test)]
        if let Some(signal) = &self.publication_capture {
            signal
                .send(self.deadline)
                .map_err(|_| Error::Transport(Stage::Denied))?;
        }
        let mut a = state
            .authority
            .lock()
            .map_err(|_| Error::Transport(Stage::Denied))?;
        let pending = a.prepare_publication_source(
            &state,
            crate::services::publication::PublicationSource {
                session: crate::retrieval::Source {
                    connection: &self.connection,
                    handle: &self.handle,
                    leaf: &self.leaf,
                    deadline: self.deadline,
                    elapsed_ms: self.start.elapsed().as_millis() as u64,
                },
                attempted: self.attempted,
                cancel: self.cancel.clone(),
                initial,
            },
            binding,
            request,
        )?;
        drop(a);
        drop(self);
        Ok(pending)
    }
}
