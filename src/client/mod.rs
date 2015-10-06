//! Contains the WebSocket client.

use std::net::TcpStream;
use std::marker::PhantomData;

use ws;
use ws::util::url::ToWebSocketUrlComponents;
use ws::receiver::{DataFrameIterator, MessageIterator};
use result::WebSocketResult;
use stream::WebSocketStream;
use dataframe::DataFrame;

use openssl::ssl::{SslContext, SslMethod, SslStream};

pub use self::request::Request;
pub use self::response::Response;

pub use self::sender::Sender;
pub use self::receiver::Receiver;

pub mod sender;
pub mod receiver;
pub mod request;
pub mod response;

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
///let message = Message::Text("Hello, World!".to_string());
///client.send_message(message).unwrap(); // Send message
///# }
///```
pub struct Client<D, S, R> {
	sender: S,
	receiver: R,
	_dataframe: PhantomData<D>
}

impl<'r, 'd> Client<DataFrame<'d>, Sender<WebSocketStream>, Receiver<'r, WebSocketStream>> {
	/// Connects to the given ws:// or wss:// URL and return a Request to be sent.
	///
	/// A connection is established, however the request is not sent to
	/// the server until a call to ```send()```.
	pub fn connect<T: ToWebSocketUrlComponents>(components: T) -> WebSocketResult<Request<WebSocketStream, WebSocketStream>> {
		let context = try!(SslContext::new(SslMethod::Tlsv1));
		Client::connect_ssl_context(components, &context)
	}
	/// Connects to the specified wss:// URL using the given SSL context.
	///
	/// If a ws:// URL is supplied, a normal, non-secure connection is established
	/// and the context parameter is ignored.
	///
	/// A connection is established, however the request is not sent to
	/// the server until a call to ```send()```.
	pub fn connect_ssl_context<T: ToWebSocketUrlComponents>(components: T, context: &SslContext) -> WebSocketResult<Request<WebSocketStream, WebSocketStream>> {
		let (host, resource_name, secure) = try!(components.to_components());
		
		let connection = try!(TcpStream::connect(
			(&host.hostname[..], host.port.unwrap_or(if secure { 443 } else { 80 }))
		));
		
		let stream = if secure {
			let sslstream = try!(SslStream::new(context, connection));
			WebSocketStream::Ssl(sslstream)
		}
		else {
			WebSocketStream::Tcp(connection)
		};
		
		Request::new((host, resource_name, secure), try!(stream.try_clone()), stream)
	}
}

impl<'r, D: 'r, S: ws::Sender<D>, R: ws::Receiver<'r, D>> Client<D, S, R> {
	/// Creates a Client from the given Sender and Receiver.
	///
	/// Essentially the opposite of `Client.split()`.
	pub fn new(sender: S, receiver: R) -> Client<D, S, R> {
		Client {
			sender: sender,
			receiver: receiver,
			_dataframe: PhantomData
		}
	}
	/// Sends a single data frame to the remote endpoint.
	pub fn send_dataframe(&mut self, dataframe: &D) -> WebSocketResult<()> {
        self.sender.send_dataframe(dataframe)
	}
	/// Sends a single message to the remote endpoint.
	pub fn send_message<M>(&mut self, message: &M) -> WebSocketResult<()>
	where M: ws::Message<D> {
        self.sender.send_message(message)
	}
	/// Reads a single data frame from the remote endpoint.
	pub fn recv_dataframe(&'r mut self) -> WebSocketResult<D> {
		self.receiver.recv_dataframe()
	}
	/// Returns an iterator over incoming data frames.
	pub fn incoming_dataframes(&'r mut self) -> DataFrameIterator<'r, R, D> {
		self.receiver.incoming_dataframes()
	}
	/// Reads a single message from this receiver.
	pub fn recv_message<M, I>(&'r mut self) -> WebSocketResult<M>
		where M: ws::Message<D, DataFrameIterator = I>, I: Iterator<Item = D> {
		
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
	///for message in client.incoming_messages::<Message>() {
	///    println!("Recv: {:?}", message.unwrap());
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
	///for message in receiver.incoming_messages::<Message>() {
	///    // Echo the message back
	///    sender.send_message(message.unwrap()).unwrap();
	///}
	///# }
	///```
	pub fn incoming_messages<M>(&'r mut self) -> MessageIterator<'r, R, D, M>
		where M: ws::Message<D> {
		
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
	///    for message in receiver.incoming_messages::<Message>() {
	///        println!("Recv: {:?}", message.unwrap());
	///    }
	///});
	///
	///let message = Message::Text("Hello, World!".to_string());
	///sender.send_message(message).unwrap();
	///# }
	///```
	pub fn split(self) -> (S, R) {
		(self.sender, self.receiver)
	}
}
