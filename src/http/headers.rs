//! A struct representing the headers in HTTP requests and responses
static MAGIC_GUID: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
pub static WS_13: [&'static str; 1] = ["13"];

pub use std::io::Error as IoError;
use std::io::Write;
use std::mem::transmute;
use openssl::crypto::hash::{self, hash};
use serialize::base64::{ToBase64, STANDARD};
use rand::random;
use ws::util::Serialize;

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
