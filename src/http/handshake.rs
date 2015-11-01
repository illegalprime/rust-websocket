//! Quickly generate WS Requests and Responses
// Everything relating to HTTP reeusts/responses and WebSocket
pub use super::headers::*;
use std::io::Write;

pub struct RequestOpts<'bp, 'p: 'bp> {
    pub protocols: Option<&'bp [&'p str]>,
}

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
    pub fn new(host: &'a str, options: &RequestOpts<'a, 'a>) -> Self {
        Request {
            host: Host(host),
            upgrade: Upgrade("websocket"),
            connection: Connection("Upgrade"),
            key: WebSocketKey::new(),
            protocol: options.protocols.map(|p| WebSocketProtocol(p)),
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
    pub fn accept(request: Request<'a>) -> Self {
        Response {
            upgrade: request.upgrade,
            connection: request.connection,
            accept: request.key.into(),
            protocol: None,
        }
    }

    pub fn accept_protocols(request: Request<'a>, protocols: &'a [&'a str]) -> Self {
        Response {
            upgrade: request.upgrade,
            connection: request.connection,
            accept: request.key.into(),
            protocol: Some(WebSocketProtocol(protocols)),
        }
    }
}

// TODO: WebSocketExtensions
// TODO: Cookies
// TODO: Custom Headers
// TODO: Header Parsing
