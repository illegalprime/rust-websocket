//! Struct for server-side WebSocket response.
use std::io::{Read, Write};

use hyper::status::StatusCode;
use hyper::version::HttpVersion;
use hyper::header::Headers;
use hyper::header::{Connection, ConnectionOption};
use hyper::header::{Upgrade, Protocol, ProtocolName};

use unicase::UniCase;

use header::{WebSocketAccept, WebSocketProtocol, WebSocketExtensions};
use server::{Request, Sender, Receiver};
use client::Client;
use result::WebSocketResult;
use dataframe::DataFrame;
use ws;

/// Represents a server-side (outgoing) response.
pub struct Response<R: Read, W: Write> {
	/// The status of the response
	pub status: StatusCode,
	/// The headers contained in this response
	pub headers: Headers,
	/// The HTTP version of this response
	pub version: HttpVersion,
	
	request: Request<R, W>
}

unsafe impl<R, W> Send for Response<R, W> where R: Read + Send, W: Write + Send { }

impl<R: Read, W: Write> Response<R, W> {
	/// Short-cut to obtain the WebSocketAccept value
	pub fn accept(&self) -> Option<&WebSocketAccept> {
		self.headers.get()
	}
	/// Short-cut to obtain the WebSocketProtocol value
	pub fn protocol(&self) -> Option<&WebSocketProtocol> {
		self.headers.get()
	}
	/// Short-cut to obtain the WebSocketExtensions value
	pub fn extensions(&self) -> Option<&WebSocketExtensions> {
		self.headers.get()
	}
		/// Returns a reference to the inner Reader.
	pub fn get_reader(&self) -> &R {
		self.request.get_reader()
	}
	/// Returns a reference to the inner Writer.
	pub fn get_writer(&self) -> &W {
		self.request.get_writer()
	}
	/// Returns a mutable reference to the inner Reader.
	pub fn get_mut_reader(&mut self) -> &mut R {
		self.request.get_mut_reader()
	}
	/// Returns a mutable reference to the inner Writer.
	pub fn get_mut_writer(&mut self) -> &mut W {
		self.request.get_mut_writer()
	}
	/// Returns a reference to the request associated with this response/
	pub fn get_request(&self) -> &Request<R, W> {
		&self.request
	}
	/// Return the inner Reader and Writer
	pub fn into_inner(self) -> (R, W) {
		self.request.into_inner()
	}
	/// Create a new outbound WebSocket response.
	pub fn new(request: Request<R, W>) -> Response<R, W> {
		let mut headers = Headers::new();
		headers.set(WebSocketAccept::new(request.key().unwrap()));
		headers.set(Connection(vec![
			ConnectionOption::ConnectionHeader(UniCase("Upgrade".to_string()))
		]));
		headers.set(Upgrade(vec![Protocol::new(ProtocolName::WebSocket, None)]));
		Response {
			status: StatusCode::SwitchingProtocols,
			headers: headers,
			version: HttpVersion::Http11,
			request: request
		}
	}
	/// Create a Bad Request response
	pub fn bad_request(request: Request<R, W>) -> Response<R, W> {
		Response {
			status: StatusCode::BadRequest,
			headers: Headers::new(),
			version: HttpVersion::Http11,
			request: request
		}
	}
	/// Short-cut to obtain a mutable reference to the WebSocketAccept value
	/// Note that to add a header that does not already exist, ```WebSocketResponse.headers.set()```
	/// must be used.
	pub fn accept_mut(&mut self) -> Option<&mut WebSocketAccept> {
		self.headers.get_mut()
	}
	/// Short-cut to obtain a mutable reference to the WebSocketProtocol value
	/// Note that to add a header that does not already exist, ```WebSocketResponse.headers.set()```
	/// must be used.
	pub fn protocol_mut(&mut self) -> Option<&mut WebSocketProtocol> {
		self.headers.get_mut()
	}
	/// Short-cut to obtain a mutable reference to the WebSocketExtensions value
	/// Note that to add a header that does not already exist, ```WebSocketResponse.headers.set()```
	/// must be used.
	pub fn extensions_mut(&mut self) -> Option<&mut WebSocketExtensions> {
		self.headers.get_mut()
	}
	
	/// Send this response with the given data frame type D, Sender B and Receiver C.
	pub fn send_with<'r, D, B, C>(mut self, sender: B, receiver: C) -> WebSocketResult<Client<D, B, C>> 
		where B: ws::Sender<D>, C: ws::Receiver<'r, D> {
		let version = self.version;
		let status = self.status;
		let headers = self.headers.clone();
		try!(write!(self.get_mut_writer(), "{} {}\r\n", version, status));
		try!(write!(self.get_mut_writer(), "{}\r\n", headers));
		Ok(Client::new(sender, receiver))
	 }
	
	/// Send this response, retrieving the inner Reader and Writer
	pub fn send_into_inner(mut self) -> WebSocketResult<(R, W)> {
		let version = self.version;
		let status = self.status;
		let headers = self.headers.clone();
		try!(write!(self.get_mut_writer(), "{} {}\r\n", version, status));
		try!(write!(self.get_mut_writer(), "{}\r\n", headers));
		Ok(self.into_inner())
	}
	
	/// Send this response, returning a Client ready to transmit/receive data frames
	pub fn send<'r, 'd>(mut self) -> WebSocketResult<Client<DataFrame<'d>, Sender<W>, Receiver<'r, R>>> {
		let version = self.version;
		let status = self.status;
		let headers = self.headers.clone();
		try!(write!(self.get_mut_writer(), "{} {}\r\n", version, status));
		try!(write!(self.get_mut_writer(), "{}\r\n", headers));
		let (reader, writer) = self.into_inner();
		let sender = Sender::new(writer);
		let receiver = Receiver::new(reader);
		Ok(Client::new(sender, receiver))
	}
}
