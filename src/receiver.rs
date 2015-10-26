//! The default implementation of a WebSocket Receiver.
#[cfg(feature = "evented")]
extern crate mio;

use std::io::Read;
use std::io::Result as IoResult;
use hyper::buffer::BufReader;

use dataframe::{DataFrame, Opcode};
use result::{WebSocketResult, WebSocketError};
use stream::WebSocketStream;
use stream::Shutdown;
use ws;

#[cfg(feature = "evented")]
use self::mio::tcp::TcpStream as EventedTcpStream;

#[cfg(feature = "evented")]
use self::mio::{Evented, Selector, Token, EventSet, PollOpt};

/// A Receiver that wraps a Reader and provides a default implementation using
/// DataFrames and Messages.
pub struct Receiver<R> {
	inner: BufReader<R>,
	buffer: Vec<DataFrame>
}

impl<R> Receiver<R> {
	/// Create a new Receiver using the specified Reader.
	pub fn new(reader: BufReader<R>) -> Receiver<R> {
		Receiver {
			inner: reader,
			buffer: Vec::new()
		}
	}
	/// Returns a reference to the underlying Reader.
	pub fn get_ref(&self) -> &BufReader<R> {
		&self.inner
	}
	/// Returns a mutable reference to the underlying Reader.
	pub fn get_mut(&mut self) -> &mut BufReader<R> {
		&mut self.inner
	}
}

impl Receiver<WebSocketStream> {
    /// Closes the receiver side of the connection, will cause all pending and future IO to
    /// return immediately with an appropriate value.
    pub fn shutdown(&mut self) -> IoResult<()> {
        self.inner.get_mut().shutdown(Shutdown::Read)
    }

    /// Shuts down both Sender and Receiver, will cause all pending and future IO to
    /// return immediately with an appropriate value.
    pub fn shutdown_all(&mut self) -> IoResult<()> {
        self.inner.get_mut().shutdown(Shutdown::Both)
    }
}

#[cfg(feature = "evented")]
impl Receiver<EventedTcpStream> {
    /// Gets a reference to the underlying TCP Stream
    pub fn stream(&self) -> &EventedTcpStream {
        self.inner.get_ref()
    }

    /// Gets a mutable reference to the underlying TCP Stream
    pub fn stream_mut(&mut self) -> &mut EventedTcpStream {
        self.inner.get_mut()
    }
}

#[cfg(feature = "evented")]
impl Evented for Receiver<EventedTcpStream> {
    fn register(&self, selector: &mut Selector, token: Token, interest: EventSet, opts: PollOpt) -> IoResult<()> {
        self.inner.get_ref().register(selector, token, interest, opts)
    }

    fn reregister(&self, selector: &mut Selector, token: Token, interest: EventSet, opts: PollOpt) -> IoResult<()> {
        self.inner.get_ref().reregister(selector, token, interest, opts)
    }

    fn deregister(&self, selector: &mut Selector) -> IoResult<()> {
        self.inner.get_ref().deregister(selector)
    }
}

impl<R: Read> ws::Receiver<DataFrame> for Receiver<R> {
	/// Reads a single data frame from the remote endpoint.
	fn recv_dataframe(&mut self) -> WebSocketResult<DataFrame> {
		DataFrame::read_dataframe(&mut self.inner, false)
	}
	/// Returns the data frames that constitute one message.
	fn recv_message_dataframes(&mut self) -> WebSocketResult<Vec<DataFrame>> {
		let mut finished = if self.buffer.is_empty() {
			let first = try!(self.recv_dataframe());

			if first.opcode == Opcode::Continuation {
				return Err(WebSocketError::ProtocolError(
					"Unexpected continuation data frame opcode"
				));
			}

			let finished = first.finished;
			self.buffer.push(first);
			finished
		}
		else {
			false
		};

		while !finished {
			let next = try!(self.recv_dataframe());
			finished = next.finished;

			match next.opcode as u8 {
				// Continuation opcode
				0 => self.buffer.push(next),
				// Control frame
				8...15 => {
					return Ok(vec![next]);
				}
				// Others
				_ => return Err(WebSocketError::ProtocolError(
					"Unexpected data frame opcode"
				)),
			}
		}

		let buffer = self.buffer.clone();
		self.buffer.clear();

		Ok(buffer)
	}
}
