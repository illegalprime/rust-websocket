//! The result type used within Rust-WebSocket

use std::io;
use std::str::Utf8Error;
use std::error::Error;
use std::convert::From;
use std::fmt;
use hyper::Error as HttpError;
use url::ParseError;

#[cfg(feature="ssl")]
use openssl::error::ErrorStack as SslError;
#[cfg(feature="ssl")]
use openssl::ssl::HandshakeError as SslHandshakeError;

/// The type used for WebSocket results
pub type WebSocketResult<T> = Result<T, WebSocketError>;

/// Represents a WebSocket error
#[derive(Debug)]
pub enum WebSocketError {
	/// A WebSocket protocol error
	ProtocolError(&'static str),
	/// Invalid WebSocket request error
	RequestError(&'static str),
	/// Invalid WebSocket response error
	ResponseError(&'static str),
	/// Invalid WebSocket data frame error
	DataFrameError(&'static str),
	/// No data available
	NoDataAvailable,
	/// An input/output error
	IoError(io::Error),
	/// An HTTP parsing error
	HttpError(HttpError),
	/// A URL parsing error
	UrlError(ParseError),
	/// A WebSocket URL error
	WebSocketUrlError(WSUrlErrorKind),
	/// An SSL error
	#[cfg(feature="ssl")]
	SslError(SslError),
	/// an ssl handshake failure
	#[cfg(feature="ssl")]
	SslHandshakeFailure,
	/// an ssl handshake interruption
	#[cfg(feature="ssl")]
	SslHandshakeInterruption,
	/// A UTF-8 error
	Utf8Error(Utf8Error),
}

impl fmt::Display for WebSocketError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		try!(fmt.write_str("WebSocketError: "));
		try!(fmt.write_str(self.description()));
		Ok(())
	}
}

impl Error for WebSocketError {
	fn description(&self) -> &str {
		match *self {
			WebSocketError::ProtocolError(_) => "WebSocket protocol error",
			WebSocketError::RequestError(_) => "WebSocket request error",
			WebSocketError::ResponseError(_) => "WebSocket response error",
			WebSocketError::DataFrameError(_) => "WebSocket data frame error",
			WebSocketError::NoDataAvailable => "No data available",
			WebSocketError::IoError(_) => "I/O failure",
			WebSocketError::HttpError(_) => "HTTP failure",
			WebSocketError::UrlError(_) => "URL failure",
			#[cfg(feature="ssl")]
			      WebSocketError::SslError(_) => "SSL failure",
			#[cfg(feature="ssl")]
            WebSocketError::SslHandshakeFailure => "SSL Handshake failure",
			#[cfg(feature="ssl")]
            WebSocketError::SslHandshakeInterruption => "SSL Handshake interrupted",
			WebSocketError::Utf8Error(_) => "UTF-8 failure",
			WebSocketError::WebSocketUrlError(_) => "WebSocket URL failure",
		}
	}

	fn cause(&self) -> Option<&Error> {
		match *self {
			WebSocketError::IoError(ref error) => Some(error),
			WebSocketError::HttpError(ref error) => Some(error),
			WebSocketError::UrlError(ref error) => Some(error),
			#[cfg(feature="ssl")]
			      WebSocketError::SslError(ref error) => Some(error),
			WebSocketError::Utf8Error(ref error) => Some(error),
			WebSocketError::WebSocketUrlError(ref error) => Some(error),
			_ => None,
		}
	}
}

impl From<io::Error> for WebSocketError {
	fn from(err: io::Error) -> WebSocketError {
		if err.kind() == io::ErrorKind::UnexpectedEof {
			return WebSocketError::NoDataAvailable;
		}
		WebSocketError::IoError(err)
	}
}

impl From<HttpError> for WebSocketError {
	fn from(err: HttpError) -> WebSocketError {
		WebSocketError::HttpError(err)
	}
}

impl From<ParseError> for WebSocketError {
	fn from(err: ParseError) -> WebSocketError {
		WebSocketError::UrlError(err)
	}
}

#[cfg(feature="ssl")]
impl From<SslError> for WebSocketError {
	fn from(err: SslError) -> WebSocketError {
		WebSocketError::SslError(err)
	}
}

#[cfg(feature="ssl")]
impl<T> From<SslHandshakeError<T>> for WebSocketError {
	fn from(err: SslHandshakeError<T>) -> WebSocketError {
		match err {
			SslHandshakeError::SetupFailure(err) => WebSocketError::SslError(err),
			SslHandshakeError::Failure(_) => WebSocketError::SslHandshakeFailure,
			SslHandshakeError::Interrupted(_) => WebSocketError::SslHandshakeInterruption,
		}
	}
}

impl From<Utf8Error> for WebSocketError {
	fn from(err: Utf8Error) -> WebSocketError {
		WebSocketError::Utf8Error(err)
	}
}

impl From<WSUrlErrorKind> for WebSocketError {
	fn from(err: WSUrlErrorKind) -> WebSocketError {
		WebSocketError::WebSocketUrlError(err)
	}
}

/// Represents a WebSocket URL error
#[derive(Debug)]
pub enum WSUrlErrorKind {
	/// Fragments are not valid in a WebSocket URL
	CannotSetFragment,
	/// The scheme provided is invalid for a WebSocket
	InvalidScheme,
	/// There is no hostname or IP address to connect to
	NoHostName,
}

impl fmt::Display for WSUrlErrorKind {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		try!(fmt.write_str("WebSocket Url Error: "));
		try!(fmt.write_str(self.description()));
		Ok(())
	}
}

impl Error for WSUrlErrorKind {
	fn description(&self) -> &str {
		match *self {
			WSUrlErrorKind::CannotSetFragment => "WebSocket URL cannot set fragment",
			WSUrlErrorKind::InvalidScheme => "WebSocket URL invalid scheme",
			WSUrlErrorKind::NoHostName => "WebSocket URL no host name provided",
		}
	}
}
