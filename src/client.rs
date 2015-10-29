//! Contains the WebSocket client.
extern crate url;

use std::net::TcpStream;
use std::marker::PhantomData;
use std::io::Result as IoResult;

use ws;
use ws::receiver::{DataFrameIterator, MessageIterator};
use result::{WebSocketResult, WebSocketError, WSUrlErrorKind};
use stream::WebSocketStream;
use dataframe::DataFrame;
use ws::dataframe::DataFrame as DataFrameable;
use openssl::ssl::{SslContext, SslMethod, SslStream};
use url::Url;
use url::ParseError;
use http::client::IntoWebSocket;

pub use super::sender::Sender;
pub use super::receiver::Receiver;

/// Represents a WebSocket client, which can send and receive messages/data frames.
///
/// `D` is the data frame type, `S` is the type implementing `Sender<D>` and `R`
/// is the type implementing `Receiver<D>`.
///
/// For most cases, the data frame type will be `dataframe::DataFrame`, the Sender
/// type will be `client::Sender<stream::WebSocketStream>` and the receiver type
/// will be `client::Receiver<stream::WebSocketStream>`.
///
/// A `Client` can be split into a `Sender` and a `Receiver` which can then be moved
/// to different threads, often using a send loop and receiver loop concurrently,
/// as shown in the client example in `examples/client.rs`.
///
///#Connecting to a Server
///
///```no_run
///extern crate websocket;
///# fn main() {
///
///use websocket::{Client, Message};
///use websocket::client::request::Url;
///
///let url = Url::parse("ws://127.0.0.1:1234").unwrap(); // Get the URL
///let request = Client::connect(url).unwrap(); // Connect to the server
///let response = request.send().unwrap(); // Send the request
///response.validate().unwrap(); // Ensure the response is valid
///
///let mut client = response.begin(); // Get a Client
///
///let message = Message::text("Hello, World!");
///client.send_message(&message).unwrap(); // Send message
///# }
///```
pub struct Client<F, S, R> {
	sender: S,
	receiver: R,
	_dataframe: PhantomData<fn(F)>
}

impl Client<DataFrame, Sender<WebSocketStream>, Receiver<WebSocketStream>> {
	/// Connects to the given ws:// or wss:// URL and return a Request to be sent.
	///
	/// A connection is established, however the request is not sent to
	/// the server until a call to ```send()```.
	pub fn connect(url: &Url) -> WebSocketResult<Self> {
		Client::create(url, None)
	}
	/// Connects to the specified wss:// URL using the given SSL context.
	///
	/// If a ws:// URL is supplied, a normal, non-secure connection is established
	/// and the context parameter is ignored.
	///
	/// A connection is established, however the request is not sent to
	/// the server until a call to ```send()```.
	pub fn connect_ssl(url: &Url, ssl: &SslContext) -> WebSocketResult<Self> {
		Client::create(url, Some(ssl))
	}

	// TODO: Don't hardcode "ws" and "wss"
	fn create(url: &Url, ssl: Option<&SslContext>) -> WebSocketResult<Self> {
		// Find the port number
		let port = match url.port() {
			Some(p) => p,
			None => {
				match &url.scheme as &str {
					"wss" => 443u16,
					"ws" => 80u16,
					_ => return Err(WebSocketError::WebSocketUrlError(
						WSUrlErrorKind::InvalidScheme
					)),
				}
			}
		};

		// Connect to the server
		let stream = match url.domain() {
			Some(host) => try!(TcpStream::connect((host, port))),
			None => return Err(WebSocketError::UrlError(
				ParseError::EmptyHost
			)),
		};

		// Add SSL if necessary
		let stream = if &url.scheme as &str == "wss" {
			let sslstream = if let Some(ref context) = ssl {
				SslStream::new(context, stream)
			} else {
				let context = try!(SslContext::new(SslMethod::Tlsv1));
				SslStream::new(&context, stream)
			};
			WebSocketStream::Ssl(try!(sslstream))
		} else {
			WebSocketStream::Tcp(stream)
		};

		// Start handshake
		stream.into_ws().map_err(|r| r.1)
	}

    /// Shuts down the sending half of the client connection, will cause all pending
    /// and future IO to return immediately with an appropriate value.
    pub fn shutdown_sender(&mut self) -> IoResult<()> {
        self.sender.shutdown()
    }

    /// Shuts down the receiving half of the client connection, will cause all pending
    /// and future IO to return immediately with an appropriate value.
    pub fn shutdown_receiver(&mut self) -> IoResult<()> {
        self.receiver.shutdown()
    }

    /// Shuts down the client connection, will cause all pending and future IO to
    /// return immediately with an appropriate value.
    pub fn shutdown(&mut self) -> IoResult<()> {
        self.receiver.shutdown_all()
    }
}

impl<F: DataFrameable, S: ws::Sender, R: ws::Receiver<F>> Client<F, S, R> {
	/// Creates a Client from the given Sender and Receiver.
	///
	/// Essentially the opposite of `Client.split()`.
	pub fn new(sender: S, receiver: R) -> Client<F, S, R> {
		Client {
			sender: sender,
			receiver: receiver,
			_dataframe: PhantomData
		}
	}
	/// Sends a single data frame to the remote endpoint.
	pub fn send_dataframe<D>(&mut self, dataframe: &D) -> WebSocketResult<()>
	where D: DataFrameable {
		self.sender.send_dataframe(dataframe)
	}
	/// Sends a single message to the remote endpoint.
	pub fn send_message<'m, M, D>(&mut self, message: &'m M) -> WebSocketResult<()>
	where M: ws::Message<'m, D>, D: DataFrameable {
		self.sender.send_message(message)
	}
	/// Reads a single data frame from the remote endpoint.
	pub fn recv_dataframe(&mut self) -> WebSocketResult<F> {
		self.receiver.recv_dataframe()
	}
	/// Returns an iterator over incoming data frames.
	pub fn incoming_dataframes<'a>(&'a mut self) -> DataFrameIterator<'a, R, F> {
		self.receiver.incoming_dataframes()
	}
	/// Reads a single message from this receiver.
	pub fn recv_message<'m, M, I>(&mut self) -> WebSocketResult<M>
	where M: ws::Message<'m, F, DataFrameIterator = I>, I: Iterator<Item = F> {
		self.receiver.recv_message()
	}
	/// Returns an iterator over incoming messages.
	///
	///```no_run
	///# extern crate websocket;
	///# fn main() {
	///use websocket::{Client, Message};
	///# use websocket::client::request::Url;
	///# let url = Url::parse("ws://127.0.0.1:1234").unwrap(); // Get the URL
	///# let request = Client::connect(url).unwrap(); // Connect to the server
	///# let response = request.send().unwrap(); // Send the request
	///# response.validate().unwrap(); // Ensure the response is valid
	///
	///let mut client = response.begin(); // Get a Client
	///
	///for message in client.incoming_messages() {
    ///    let message: Message = message.unwrap();
	///    println!("Recv: {:?}", message);
	///}
	///# }
	///```
	///
	/// Note that since this method mutably borrows the `Client`, it may be necessary to
	/// first `split()` the `Client` and call `incoming_messages()` on the returned
	/// `Receiver` to be able to send messages within an iteration.
	///
	///```no_run
	///# extern crate websocket;
	///# fn main() {
	///use websocket::{Client, Message, Sender, Receiver};
	///# use websocket::client::request::Url;
	///# let url = Url::parse("ws://127.0.0.1:1234").unwrap(); // Get the URL
	///# let request = Client::connect(url).unwrap(); // Connect to the server
	///# let response = request.send().unwrap(); // Send the request
	///# response.validate().unwrap(); // Ensure the response is valid
	///
	///let client = response.begin(); // Get a Client
	///let (mut sender, mut receiver) = client.split(); // Split the Client
	///for message in receiver.incoming_messages() {
    ///    let message: Message = message.unwrap();
	///    // Echo the message back
	///    sender.send_message(&message).unwrap();
	///}
	///# }
	///```
	pub fn incoming_messages<'a, M, D>(&'a mut self) -> MessageIterator<'a, R, D, F, M>
	where M: ws::Message<'a, D>,
          D: DataFrameable
    {
		self.receiver.incoming_messages()
	}
	/// Returns a reference to the underlying Sender.
	pub fn get_sender(&self) -> &S {
		&self.sender
	}
	/// Returns a reference to the underlying Receiver.
	pub fn get_reciever(&self) -> &R {
		&self.receiver
	}
	/// Returns a mutable reference to the underlying Sender.
	pub fn get_mut_sender(&mut self) -> &mut S {
		&mut self.sender
	}
	/// Returns a mutable reference to the underlying Receiver.
	pub fn get_mut_reciever(&mut self) -> &mut R {
		&mut self.receiver
	}
	/// Split this client into its constituent Sender and Receiver pair.
	///
	/// This allows the Sender and Receiver to be sent to different threads.
	///
	///```no_run
	///# extern crate websocket;
	///# fn main() {
	///use websocket::{Client, Message, Sender, Receiver};
	///use std::thread;
	///# use websocket::client::request::Url;
	///# let url = Url::parse("ws://127.0.0.1:1234").unwrap(); // Get the URL
	///# let request = Client::connect(url).unwrap(); // Connect to the server
	///# let response = request.send().unwrap(); // Send the request
	///# response.validate().unwrap(); // Ensure the response is valid
	///
	///let client = response.begin(); // Get a Client
	///
	///let (mut sender, mut receiver) = client.split();
	///
	///thread::spawn(move || {
	///    for message in receiver.incoming_messages() {
    ///        let message: Message = message.unwrap();
	///        println!("Recv: {:?}", message);
	///    }
	///});
	///
	///let message = Message::text("Hello, World!");
	///sender.send_message(&message).unwrap();
	///# }
	///```
	pub fn split(self) -> (S, R) {
		(self.sender, self.receiver)
	}
}