extern crate url;

use std::io::{Read, Write};
use std::io::Result as IoResult;
use hyper::http::h1::Incoming;
use hyper::http::h1::parse_response;
use hyper::http::h1::parse_request;
use hyper::http::RawStatus;
use hyper::status::StatusCode;
use hyper::version::HttpVersion;
use hyper::header::Headers;
use hyper::method::Method;
use hyper::uri::RequestUri;

use receiver::Receiver;
use sender::Sender;
use dataframe::DataFrame;
use stream::WebSocketStream;
use openssl::ssl::SslStream;
use std::net::TcpStream;

pub use hyper::buffer::BufReader;
pub use hyper::error::Result as HyperResult;

pub mod headers;

pub struct Message<S>(Incoming<S>);
pub type Response = RawStatus;
pub type Request = (Method, RequestUri);

impl<S> Message<S> {
    fn send<W>(&self, writer: &mut W) -> IoResult<()>
    where W: Write {
        unimplemented!();
    }
}

impl Message<Response> {
    pub fn new<R>(reader: &mut BufReader<R>) -> HyperResult<Self>
    where R: Read {
        Ok(Message(
            try!(parse_response(reader))
        ))
    }
}

impl Message<Request> {
    fn new<R>(reader: &mut BufReader<R>) -> HyperResult<Self>
    where R: Read {
        Ok(Message(
            try!(parse_request(reader))
        ))
    }
}

/// Take a message and determine if it is a WebSocket upgrade request.
pub trait IsWsUpgrade {
    fn is_ws_upgrade(&self) -> bool;
}

impl IsWsUpgrade for Message<Request> {
    fn is_ws_upgrade(&self) -> bool {
        unimplemented!();
    }
}

impl<R> IsWsUpgrade for R
where R: Read {
    fn is_ws_upgrade(&self) -> bool {
        unimplemented!();
    }
}

/// Take a message and determine if it constitutes a successful ws handshake,
/// or a failed ws handshake. An error value is returned if this message was
/// not a response to a ws handshake.
pub trait WsHandshakeSucceeded {
    fn handshake_succeeded(&self) -> Result<bool, ()>;
}

impl WsHandshakeSucceeded for Message<Response> {
    fn handshake_succeeded(&self) -> Result<bool, ()> {
        unimplemented!();
    }
}

impl<R> WsHandshakeSucceeded for R
where R: Read {
    fn handshake_succeeded(&self) -> Result<bool, ()> {
        unimplemented!();
    }
}

pub mod server {
    // TODO: Servers should get an itermediate form
    // that shows the original ws request and lets the server filter
    // through protocols, route, etc. Then send it back
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use openssl::ssl::SslStream;
    use stream::WebSocketStream;
    use server::Connection;
    use result::WebSocketError;
    use client::Client;
    use sender::Sender;
    use receiver::Receiver;
    use dataframe::DataFrame;
    /// Turns a RW stream into a ws connection if the ws handshake was successful
    /// Blocking read for a WebSocket
    pub trait IntoWebSocket: Sized {
        type Client;

        fn into_ws(self) -> Result<Self::Client, (Self, WebSocketError)>;
    }

    impl IntoWebSocket for TcpStream {
        type Client = Client<DataFrame, Sender<Self>, Receiver<Self>>;

        fn into_ws(self) -> Result<Self::Client, (Self, WebSocketError)> {
            unimplemented!();
        }
    }

    impl IntoWebSocket for SslStream<TcpStream> {
        type Client = Client<DataFrame, Sender<Self>, Receiver<Self>>;

        fn into_ws(self) -> Result<Self::Client, (Self, WebSocketError)> {
            unimplemented!();
        }
    }

    impl IntoWebSocket for WebSocketStream {
        type Client = Client<DataFrame, Sender<Self>, Receiver<Self>>;

        fn into_ws(self) -> Result<Self::Client, (Self, WebSocketError)> {
            unimplemented!();
        }
    }

    impl<R, W> IntoWebSocket for Connection<R, W>
    where R: Read,
          W: Write,
    {
        type Client = Client<DataFrame, Sender<W>, Receiver<R>>;

        fn into_ws(self) -> Result<Self::Client, (Self, WebSocketError)> {
            unimplemented!();
        }
    }

    impl<R, W> IntoWebSocket for (R, W)
    where R: Read,
          W: Write,
    {
        type Client = Client<DataFrame, Sender<W>, Receiver<R>>;

        fn into_ws(self) -> Result<Self::Client, (Self, WebSocketError)> {
            unimplemented!();
        }
    }

    // TODO: One for mio as well
    // TODO: More impls for hyper maybe?
}

pub mod client {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use openssl::ssl::SslStream;
    use stream::WebSocketStream;
    use client::Client;
    use sender::Sender;
    use receiver::Receiver;
    use dataframe::DataFrame;
    use result::WebSocketError;
    /// Trait to turn a stream into a ws client by handshaking with the server
    /// Note the stream should already be connected to the server
    pub trait IntoWebSocket: Sized {
        type Client;

        fn into_ws(self) -> Result<Self::Client, (Self, WebSocketError)>;
    }

    impl IntoWebSocket for WebSocketStream {
        type Client = Client<DataFrame, Sender<Self>, Receiver<Self>>;

        fn into_ws(self) -> Result<Self::Client, (Self, WebSocketError)> {
            unimplemented!();
        }
    }

    impl IntoWebSocket for TcpStream {
        type Client = Client<DataFrame, Sender<WebSocketStream>, Receiver<WebSocketStream>>;

        fn into_ws(self) -> Result<Self::Client, (Self, WebSocketError)> {
            unimplemented!();
        }
    }

    impl IntoWebSocket for SslStream<TcpStream> {
        type Client = Client<DataFrame, Sender<WebSocketStream>, Receiver<WebSocketStream>>;

        fn into_ws(self) -> Result<Self::Client, (Self, WebSocketError)> {
            unimplemented!();
        }
    }

    impl<R, W> IntoWebSocket for (R, W)
    where R: Read,
          W: Write,
    {
        type Client = Client<DataFrame, Sender<W>, Receiver<R>>;

        fn into_ws(self) -> Result<Self::Client, (Self, WebSocketError)> {
            unimplemented!();
        }
    }

    // TODO: Add one for mio
}
