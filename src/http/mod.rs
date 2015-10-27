extern crate url;

use std::io::{Read, Write};
use hyper::status::StatusCode;
use hyper::version::HttpVersion;
use hyper::header::Headers;
use hyper::method::Method;
pub use url::Url;

pub type Request = Packet;
pub type Response = Packet;

pub struct Packet {
    destination: Url,

    status: StatusCode,
    method: Method,
    version: HttpVersion,

    headers: Headers,

    payload: Option<Vec<u8>>,
}

impl Packet {
    fn read<R>(reader: &R) -> Result<Packet>
    where R: Read {
        // read packet
    }

    fn send<W>(&self, writer: &mut W) -> Result<()>
    where W: Writer {

    }
}

// NOTE: This is good for the internal imp
// as well as providing a good way for users to check if they should switch to using ws
pub trait IsWsUpgrade {
    fn is_ws_upgrade(&self) -> bool;
}

impl IsWsUpgrade for Packet {
    fn is_ws_upgrade(&self) -> bool {
        // TODO
    }
}

impl<R> IsWsUpgrade for R
where R: Reader {
    fn is_ws_upgrade(&self) -> bool {
        // TODO
    }
}

// TODO: Some trait that returns bool to determine if this message constitutes a
// successful handshake (response from the upgrade request)

// TODO: More impls of above for hyper like libs, like support the hyper::Request

mod server {
    // Maybe turn a rw stream into a into a ws connection
    pub trait IntoWebSocket {
        fn into_ws(self) -> Result<Connection, Self>;
    }

    impl IntoWebSocket for TcpStream {
        fn into_ws(self) -> Result<Connection, Self> {
            // TODO
        }
    }

    // TODO: One for mio as well

    // TODO: Maybe this should be removed since it will require splitting
    // S into two Arc<Mutex<S>>? Better for people to clone it themselves.
    // Also conflicts with TcpStream impl
    impl<S> IntoWebSocket for S
    where S: Read + Write {
        fn into_ws(self) -> Result<Connection, Self> {
            // TODO
        }
    }

    impl<R, W> IntoWebSocket for (R, W)
    where R: Read,
          W: Write
    {
        fn into_ws(self) -> Result<Connection, Self> {
            // TODO
        }
    }

    // TODO: More impls for hyper maybe?
}

mod client {
    // TODO: Trait to turn a stream into a ws connection
    // by initiating handshake
}
