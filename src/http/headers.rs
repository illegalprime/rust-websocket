//! A struct representing the headers in HTTP requests and responses
pub mod header {
    static MAGIC_GUID: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    pub static WS_13: [&'static str; 1] = ["13"];

    pub use std::io::Error as IoError;
    use std::io::Write;
    use std::mem::transmute;
    use openssl::crypto::hash::{self, hash};
    use serialize::base64::{ToBase64, STANDARD};
    use rand::random;

    pub struct Host<'a>(pub &'a str);
    pub struct Origin<'a>(pub &'a str);
    pub struct Upgrade<'a>(pub &'a str);
    pub struct Connection<'a>(pub &'a str);
    pub struct WebSocketKey(pub String);
    pub struct WebSocketAccept(pub String);
    pub struct WebSocketVersion<'a>(pub &'a [&'a str]);
    pub struct WebSocketProtocol<'a>(pub &'a [&'a str]);

    impl WebSocketKey {
        pub fn new() -> Self {
    		let key: [u8; 16] = unsafe {
    			// Much faster than calling random() several times
    			transmute(random::<(u64, u64)>())
    		};
    		WebSocketKey(key.to_base64(STANDARD))
        }
    }

    impl Into<WebSocketAccept> for WebSocketKey {
        fn into(mut self) -> WebSocketAccept {
            // Tack on magin GUID
            self.0.push_str(MAGIC_GUID);
            // SHA1 it!
            let output = hash(hash::Type::SHA1, self.0.as_bytes());
            // Into Base64
            WebSocketAccept(output.to_base64(STANDARD))
        }
    }

    pub trait Header {
        fn field() -> &'static str;

        fn write_value<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write;
    }

    impl<'a> Header for Host<'a> {
        #[inline(always)]
        fn field() -> &'static str {
            "Host"
        }

        fn write_value<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write {
            stream.write_all(self.0.as_bytes())
        }
    }

    impl<'a> Header for Upgrade<'a> {
        #[inline(always)]
        fn field() -> &'static str {
            "Upgrade"
        }

        fn write_value<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write {
            stream.write_all(self.0.as_bytes())
        }
    }

    impl<'a> Header for Connection<'a> {
        #[inline(always)]
        fn field() -> &'static str {
            "Connection"
        }

        fn write_value<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write {
            stream.write_all(self.0.as_bytes())
        }
    }

    impl<'a> Header for WebSocketKey {
        #[inline(always)]
        fn field() -> &'static str {
            "Sec-WebSocket-Key"
        }

        fn write_value<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write {
            stream.write_all(self.0.as_bytes())
        }
    }

    impl<'a> Header for WebSocketAccept {
        #[inline(always)]
        fn field() -> &'static str {
            "Sec-WebSocket-Accept"
        }

        fn write_value<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write {
            stream.write_all(self.0.as_bytes())
        }
    }

    impl<'a> Header for WebSocketProtocol<'a> {
        #[inline(always)]
        fn field() -> &'static str {
            "Sec-WebSocket-Protocol"
        }

        fn write_value<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write {
            let mut protocols = self.0.iter();

            if let Some(proto) = protocols.next() {
                try!(stream.write_all(proto.as_bytes()));
            } else {
                return Ok(());
            }

            for proto in protocols {
                try!(stream.write_all(", ".as_bytes()));
                try!(stream.write_all(proto.as_bytes()));
            }
            Ok(())
        }
    }

    impl<'a> Header for WebSocketVersion<'a> {
        #[inline(always)]
        fn field() -> &'static str {
            "Sec-WebSocket-Version"
        }

        fn write_value<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write {
            let mut versions = self.0.iter();

            if let Some(version) = versions.next() {
                try!(stream.write_all(version.as_bytes()));
            } else {
                return Ok(());
            }

            for version in versions {
                try!(stream.write_all(", ".as_bytes()));
                try!(stream.write_all(version.as_bytes()));
            }
            Ok(())
        }
    }

    impl<'a> Header for Origin<'a> {
        #[inline(always)]
        fn field() -> &'static str {
            "Origin"
        }

        fn write_value<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write {
            stream.write_all(self.0.as_bytes())
        }
    }

    pub trait Serialize {
        fn serialize<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write;
    }

    impl<H> Serialize for H
    where H: Header {
        fn serialize<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write {
            try!( stream.write_all(Self::field().as_bytes()) );
            try!( stream.write_all(": ".as_bytes()) );
            try!( self.write_value(stream) );
            stream.write_all("\r\n".as_bytes())
        }
    }
}

pub mod handshake {
    pub use super::header::*;
    use std::io::Write;

    pub struct Request<'a> {
        pub host: Host<'a>,
        pub upgrade: Upgrade<'a>,
        pub connection: Connection<'a>,
        pub key: WebSocketKey,
        pub protocol: Option<WebSocketProtocol<'a>>,
        pub version: WebSocketVersion<'a>,
        pub origin: Option<Origin<'a>>,
    }

    impl<'a> Serialize for Request<'a> {
        fn serialize<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write {
            try!( self.host.serialize(stream) );
            try!( self.upgrade.serialize(stream) );
            try!( self.connection.serialize(stream) );
            try!( self.key.serialize(stream) );
            try!( self.version.serialize(stream) );

            if let Some(ref protocol) = self.protocol {
                try!( protocol.serialize(stream) );
            }

            if let Some(ref origin) = self.origin {
                try!( origin.serialize(stream) );
            }

            stream.write_all("\r\n".as_bytes())
        }
    }

    impl<'a> Request<'a> {
        pub fn new(host: &'a str) -> Self {
            Request {
                host: Host(host),
                upgrade: Upgrade("websocket"),
                connection: Connection("Upgrade"),
                key: WebSocketKey::new(),
                protocol: None,
                // TODO: Support more versions!
                version: WebSocketVersion(&WS_13),
                origin: None,
            }
        }

        pub fn with_protocols(&mut self, protocols: &'a [&'a str]) {
            self.protocol = Some(WebSocketProtocol(protocols));
        }

        pub fn with_origin(&mut self, origin: &'a str) {
            self.origin = Some(Origin(origin));
        }
    }

    pub struct Response<'a> {
        pub upgrade: Upgrade<'a>,
        pub connection: Connection<'a>,
        pub accept: WebSocketAccept,
        pub protocol: Option<WebSocketProtocol<'a>>,
    }

    impl<'a> Serialize for Response<'a> {
        fn serialize<W>(&self, stream: &mut W) -> Result<(), IoError>
        where W: Write {
            try!( self.upgrade.serialize(stream) );
            try!( self.connection.serialize(stream) );
            try!( self.accept.serialize(stream) );

            if let Some(ref protocol) = self.protocol {
                try!( protocol.serialize(stream) );
            }

            stream.write_all("\r\n".as_bytes())
        }
    }

    impl<'a> Response<'a> {
        pub fn accept(request: Request) -> Self {
            Response {
                upgrade: request.upgrade,
                connection: request.connection,
                accept: request.key.into(),
                protocol: None,
            }
        }

        pub fn accept_protocols(request: Request, protocols: &'a [&'a str]) -> Self {
            Response {
                upgrade: request.upgrade,
                connection: request.connection,
                accept: request.key.into(),
                protocol: Some(WebSocketProtocol(protocols)),
            }
        }
    }
}

// TODO: WebSocketExtensions
// TODO: Cookies
// TODO: Custom Headers
// TODO: Header Parsing
