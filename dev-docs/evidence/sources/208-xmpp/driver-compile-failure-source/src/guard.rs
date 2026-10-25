//! Incremental rxml guard below the XMPP DOM parser. No XML framing is implemented here.
use rxml::{error::EndOrError, Event, Parse, Parser};
use std::{cell::RefCell, io, pin::Pin, rc::Rc, task::{Context, Poll}};
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite, ReadBuf};

pub const MAX_BYTES: usize = 65536;
pub const MAX_DEPTH: usize = 32;
pub const MAX_EVENTS: usize = 256;
pub const MAX_STANZA_BYTES: usize = 16384;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection { Limit, Malformed }

pub struct Guard {
    parser: Parser,
    bytes: usize,
    depth: usize,
    events: usize,
    stanza_bytes: usize,
    restarted: bool,
    rejection: Option<Rejection>,
}
impl Default for Guard { fn default() -> Self { Self::new() } }
impl Guard {
    pub fn new() -> Self { Self { parser: Parser::new(), bytes: 0, depth: 0, events: 0, stanza_bytes: 0, restarted: false, rejection: None } }
    pub fn bytes(&self) -> usize { self.bytes }
    pub fn depth(&self) -> usize { self.depth }
    pub fn events(&self) -> usize { self.events }
    pub fn rejection(&self) -> Option<Rejection> { self.rejection }
    fn reject(&mut self, why: Rejection) -> Result<(), Rejection> { self.rejection = Some(why); Err(why) }
    pub fn feed(&mut self, byte: u8) -> Result<(), Rejection> {
        if let Some(why) = self.rejection { return Err(why); }
        self.bytes += 1;
        self.stanza_bytes += 1;
        if self.bytes > MAX_BYTES || self.stanza_bytes > MAX_STANZA_BYTES { return self.reject(Rejection::Limit); }
        let bytes = [byte];
        let mut input = bytes.as_slice();
        loop {
            match self.parser.parse(&mut input, false) {
                Ok(Some(event)) => {
                    self.events += 1;
                    if self.events > MAX_EVENTS { return self.reject(Rejection::Limit); }
                    match event {
                        Event::StartElement(..) => {
                            self.depth += 1;
                            if self.depth > MAX_DEPTH { return self.reject(Rejection::Limit); }
                        }
                        Event::EndElement(..) => { self.depth -= 1; }
                        _ => {}
                    }
                    // Includes root/header and inter-stanza whitespace in the bound.
                    if self.depth <= 1 { self.stanza_bytes = 0; }
                }
                Err(EndOrError::NeedMoreData) => return Ok(()),
                Ok(None) => return Ok(()),
                Err(_) => return self.reject(Rejection::Malformed),
            }
        }
    }
    /// Called by the trusted driver only after the library accepted SASL success.
    /// The guard's lifetime counters survive the single permitted XML restart.
    pub fn restart(&mut self) -> Result<(), Rejection> {
        if self.restarted || self.depth != 1 || self.rejection.is_some() { return self.reject(Rejection::Malformed); }
        self.parser = Parser::new();
        self.depth = 0;
        self.stanza_bytes = 0;
        self.restarted = true;
        Ok(())
    }
}

pub type SharedGuard = Rc<RefCell<Guard>>;
pub struct GuardedIo<T> { inner: T, state: SharedGuard, byte: [u8; 1], available: bool }
impl<T> GuardedIo<T> {
    pub(crate) fn replace_state(&mut self, state: SharedGuard) { self.state = state; }
    pub fn new(inner: T) -> (Self, SharedGuard) {
        let state = Rc::new(RefCell::new(Guard::new()));
        (Self { inner, state: state.clone(), byte: [0], available: false }, state)
    }
}
fn rejected() -> io::Error { io::Error::new(io::ErrorKind::InvalidData, "XML guard rejected input") }
impl<T: AsyncRead + Unpin> AsyncBufRead for GuardedIo<T> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        let this = self.get_mut();
        if this.state.borrow().rejection().is_some() { return Poll::Ready(Err(rejected())); }
        if !this.available {
            let mut buf = ReadBuf::new(&mut this.byte);
            match Pin::new(&mut this.inner).poll_read(cx, &mut buf) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Ready(Ok(())) if buf.filled().is_empty() => return Poll::Ready(Ok(&[])),
                Poll::Ready(Ok(())) => {}
            }
            if this.state.borrow_mut().feed(this.byte[0]).is_err() { return Poll::Ready(Err(rejected())); }
            this.available = true;
        }
        Poll::Ready(Ok(&this.byte))
    }
    fn consume(self: Pin<&mut Self>, amt: usize) { let this = self.get_mut(); assert!(amt <= usize::from(this.available)); if amt == 1 { this.available = false; } }
}
impl<T: AsyncRead + Unpin> AsyncRead for GuardedIo<T> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        if buf.remaining() == 0 { return Poll::Ready(Ok(())); }
        match self.as_mut().poll_fill_buf(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Ready(Ok(bytes)) => { let n = bytes.len(); buf.put_slice(bytes); self.consume(n); Poll::Ready(Ok(())) }
        }
    }
}
impl<T: AsyncWrite + Unpin> AsyncWrite for GuardedIo<T> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, bytes: &[u8]) -> Poll<io::Result<usize>> { Pin::new(&mut self.get_mut().inner).poll_write(cx, bytes) }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> { Pin::new(&mut self.get_mut().inner).poll_flush(cx) }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> { Pin::new(&mut self.get_mut().inner).poll_shutdown(cx) }
}
