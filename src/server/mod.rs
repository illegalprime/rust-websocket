//! Provides an implementation of a WebSocket server
use std::net::{SocketAddr, ToSocketAddrs, TcpListener};
use std::io::{Read, Write};
use std::io;
pub use self::request::Request;
pub use self::response::Response;
pub use self::sender::Sender;
pub use self::receiver::Receiver;

use stream::WebSocketStream;

use openssl::ssl::SslContext;
use openssl::ssl::SslStream;

pub mod request;
pub mod response;
pub mod sender;
pub mod receiver;

/// Represents a WebSocket server which can work with either normal (non-secure) connections, or secure WebSocket connections.
///
/// This is a convenient way to implement WebSocket servers, however it is possible to use any sendable Reader and Writer to obtain
/// a WebSocketClient, so if needed, an alternative server implementation can be used.
///#Non-secure Servers
///
/// ```no_run
///extern crate websocket;
///# fn main() {
///use std::thread;
///use websocket::{Server, Message};
///
///let server = Server::bind("127.0.0.1:1234").unwrap();
///
///for connection in server {
///    // Spawn a new thread for each connection.
///    thread::spawn(move || {
///		   let request = connection.unwrap().read_request().unwrap(); // Get the request
///		   let response = request.accept(); // Form a response
///		   let mut client = response.send().unwrap(); // Send the response
///
///		   let message = Message::Text("Hello, client!".to_string());
///		   let _ = client.send_message(&message);
///
///		   // ...
///    });
///}
/// # }
/// ```
///
///#Secure Servers
/// ```no_run
///extern crate websocket;
///extern crate openssl;
///# fn main() {
///use std::thread;
///use std::path::Path;
///use websocket::{Server, Message};
///use openssl::ssl::{SslContext, SslMethod};
///use openssl::x509::X509FileType;
///
///let mut context = SslContext::new(SslMethod::Tlsv1).unwrap();
///let _ = context.set_certificate_file(&(Path::new("cert.pem")), X509FileType::PEM);
///let _ = context.set_private_key_file(&(Path::new("key.pem")), X509FileType::PEM);
///let server = Server::bind_secure("127.0.0.1:1234", &context).unwrap();
///
///for connection in server {
///    // Spawn a new thread for each connection.
///    thread::spawn(move || {
///		   let request = connection.unwrap().read_request().unwrap(); // Get the request
///		   let response = request.accept(); // Form a response
///		   let mut client = response.send().unwrap(); // Send the response
///
///		   let message = Message::Text("Hello, client!".to_string());
///		   let _ = client.send_message(&message);
///
///		   // ...
///    });
///}
/// # }
/// ```
pub struct Server<'a> {
	inner: TcpListener,
	context: Option<&'a SslContext>,
}

impl<'a> Server<'a> {
	/// Bind this Server to this socket
	pub fn bind<T: ToSocketAddrs>(addr: T) -> io::Result<Server<'a>> {
		Ok(Server {
			inner: try!(TcpListener::bind(&addr)),
			context: None,
		})
	}
	/// Bind this Server to this socket, utilising the given SslContext
	pub fn bind_secure<T: ToSocketAddrs>(addr: T, context: &'a SslContext) -> io::Result<Server<'a>> {
		Ok(Server {
			inner: try!(TcpListener::bind(&addr)),
			context: Some(context),
		})
	}
	/// Get the socket address of this server
	pub fn local_addr(&mut self) -> io::Result<SocketAddr> {
		self.inner.local_addr()
	}
	
	/// Create a new independently owned handle to the underlying socket.
	pub fn try_clone(&self) -> io::Result<Server<'a>> {
		let inner = try!(self.inner.try_clone());
		Ok(Server {
			inner: inner,
			context: self.context
		})
	}

	/// Wait for and accept an incoming WebSocket connection, returning a WebSocketRequest
	pub fn accept(&mut self) -> io::Result<Connection<WebSocketStream, WebSocketStream>> {
		let stream = try!(self.inner.accept()).0;
		let wsstream = match self.context {
			Some(context) => {
				let sslstream = match SslStream::new_server(context, stream) {
					Ok(s) => s,
					Err(err) => {
						return Err(io::Error::new(io::ErrorKind::Other, err));
					}
				};
				WebSocketStream::Ssl(sslstream)
			}
			None => { WebSocketStream::Tcp(stream) }
		};
		Ok(Connection(try!(wsstream.try_clone()), try!(wsstream.try_clone())))
	}
}

impl<'a> Iterator for Server<'a> {
	type Item = io::Result<Connection<WebSocketStream, WebSocketStream>>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		Some(self.accept())
	}
}

/// Represents a connection to the server that has not been processed yet.
pub struct Connection<R: Read, W: Write>(R, W);

impl<R: Read, W: Write> Connection<R, W> {
	/// Process this connection and read the request.
	pub fn read_request(self) -> io::Result<Request<R, W>> {
		match Request::read(self.0, self.1) {
			Ok(result) => { Ok(result) },
			Err(err) => {
				Err(io::Error::new(io::ErrorKind::InvalidInput, err))
			}
		}
	}
}
