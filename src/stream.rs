//! Provides the default stream type for WebSocket connections.
use std::io::{self, Read, Write};
use openssl::ssl::SslStream;

pub use std::net::{SocketAddr, Shutdown, TcpStream};

/// A useful stream type for carrying WebSocket connections.
pub enum WebSocketStream {
	/// A TCP stream.
	Tcp(TcpStream),
	/// An SSL-backed TCP Stream
	Ssl(SslStream<TcpStream>)
}

impl Read for WebSocketStream {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		match *self {
		WebSocketStream::Tcp(ref mut inner) => inner.read(buf),
			WebSocketStream::Ssl(ref mut inner) => inner.read(buf),
		}
	}
}

impl Write for WebSocketStream {
	fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.write(msg),
			WebSocketStream::Ssl(ref mut inner) => inner.write(msg),
		}
	}

	fn flush(&mut self) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.flush(),
			WebSocketStream::Ssl(ref mut inner) => inner.flush(),
		}
	}
}

impl WebSocketStream {
	/// See `TcpStream.peer_addr()`.
	pub fn peer_addr(&mut self) -> io::Result<SocketAddr> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.peer_addr(),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().peer_addr(),
		}
	}
	/// See `TcpStream.local_addr()`.
	pub fn local_addr(&mut self) -> io::Result<SocketAddr> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.local_addr(),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().local_addr(),
		}
	}
	/// See `TcpStream.shutdown()`.
	pub fn shutdown(&mut self, shutdown: Shutdown) -> io::Result<()> {
		match *self {
			WebSocketStream::Tcp(ref mut inner) => inner.shutdown(shutdown),
			WebSocketStream::Ssl(ref mut inner) => inner.get_mut().shutdown(shutdown),
		}
	}
	/// See `TcpStream.try_clone()`.
	pub fn try_clone(&self) -> io::Result<WebSocketStream> {
		Ok(match *self {
			WebSocketStream::Tcp(ref inner) => WebSocketStream::Tcp(try!(inner.try_clone())),
			WebSocketStream::Ssl(ref inner) => WebSocketStream::Ssl(try!(inner.try_clone())),
		})
	}
	/// Returns a borrow to the inner TCP Stream
	pub fn inner(&self) -> &TcpStream {
		match self {
			&WebSocketStream::Tcp(ref inner) => inner,
			&WebSocketStream::Ssl(ref inner) => inner.get_ref(),
		}
	}
	/// Returns a mutable borrow to the inner TCP Stream
	pub fn inner_mut(&mut self) -> &mut TcpStream {
		match self {
			&mut WebSocketStream::Tcp(ref mut inner) => inner,
			&mut WebSocketStream::Ssl(ref mut inner) => inner.get_mut(),
		}
	}
}
