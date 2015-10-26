//! The default implementation of a WebSocket Sender.
#[cfg(feature = "evented")]
extern crate mio;

use std::io::Write;
use std::io::Result as IoResult;
use result::WebSocketResult;
use ws::dataframe::DataFrame;
use stream::WebSocketStream;
use stream::Shutdown;
use ws;

#[cfg(feature = "evented")]
use self::mio::tcp::TcpStream as EventedTcpStream;

#[cfg(feature = "evented")]
use self::mio::{Evented, Selector, Token, EventSet, PollOpt};

/// A Sender that wraps a Writer and provides a default implementation using
/// DataFrames and Messages.
pub struct Sender<W> {
	inner: W
}

impl<W> Sender<W> {
	/// Create a new WebSocketSender using the specified Writer.
	pub fn new(writer: W) -> Sender<W> {
		Sender {
			inner: writer
		}
	}
	/// Returns a reference to the underlying Writer.
	pub fn get_ref(&self) -> &W {
		&self.inner
	}
	/// Returns a mutable reference to the underlying Writer.
	pub fn get_mut(&mut self) -> &mut W {
		&mut self.inner
	}
}

impl Sender<WebSocketStream> {
    /// Closes the sender side of the connection, will cause all pending and future IO to
    /// return immediately with an appropriate value.
    pub fn shutdown(&mut self) -> IoResult<()> {
        self.inner.shutdown(Shutdown::Write)
    }

    /// Shuts down both Sender and Receiver, will cause all pending and future IO to
    /// return immediately with an appropriate value.
    pub fn shutdown_all(&mut self) -> IoResult<()> {
        self.inner.shutdown(Shutdown::Both)
    }
}

#[cfg(feature = "evented")]
impl Sender<EventedTcpStream> {
    /// Gets a reference to the underlying TCP Stream
    pub fn stream(&self) -> &EventedTcpStream {
        &self.inner
    }

    /// Gets a mutable reference to the underlying TCP Stream
    pub fn stream_mut(&mut self) -> &mut EventedTcpStream {
        &mut self.inner
    }
}

#[cfg(feature = "evented")]
impl Evented for Sender<EventedTcpStream> {
    fn register(&self, selector: &mut Selector, token: Token, interest: EventSet, opts: PollOpt) -> IoResult<()> {
        self.inner.register(selector, token, interest, opts)
    }

    fn reregister(&self, selector: &mut Selector, token: Token, interest: EventSet, opts: PollOpt) -> IoResult<()> {
        self.inner.reregister(selector, token, interest, opts)
    }

    fn deregister(&self, selector: &mut Selector) -> IoResult<()> {
        self.inner.deregister(selector)
    }
}

impl<W: Write> ws::Sender for Sender<W> {
	/// Sends a single data frame to the remote endpoint.
	fn send_dataframe<D>(&mut self, dataframe: &D) -> WebSocketResult<()>
	where D: DataFrame {
		dataframe.write_to(&mut self.inner, true)
	}
}
