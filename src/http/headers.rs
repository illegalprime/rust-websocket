//! A struct representing the headers in HTTP requests and responses
use std::collections::HashMap;
pub use header::*;

pub mod handshake {
    pub struct Request<'a> {
        pub host: Host<'a>,
        pub upgrade: Upgrade<'a>,
        pub connection: Connection<'a>,
        pub key: SecWebSocketKey<'a>,
        pub protocol: Option<SecWebSocketProtocol<'a>>,
        pub version: SecWebSocketVersion<'a>,
        pub origin: Option<Origin<'a>>,
    }

    pub struct Response<'a> {
        pub upgrade: Upgrade<'a>,
        pub connection: Connection<'a>,
        pub accept: SecWebSocketAccept<'a>,
        pub protocol: Option<SecWebSocketProtocol<'a>>,
    }
}

pub mod header {
    pub struct Host<'a>(pub &'a str);
    pub struct Upgrade<'a>(pub &'a str);
    pub struct Connection<'a>(pub &'a str);
    pub struct SecWebSocketKey<'a>(pub &'a str);
    pub struct SecWebSocketAccept<'a>(pub &'a str);
    pub struct SecWebSocketProtocol<'a>(pub [&'a str]);
    pub struct SecWebSocketVersion<'a>(pub &'a str);

    pub trait HeaderField {
        fn field(&self) -> &'static str;
    }

    impl HeaderField for Host {
        #[inline(always)]
        fn field(&_self) -> &'static str {
            "Host"
        }
    }

    impl HeaderField for Upgrade {
        #[inline(always)]
        fn field(&_self) -> &'static str {
            "Upgrade"
        }
    }

    impl HeaderField for Connection {
        #[inline(always)]
        fn field(&_self) -> &'static str {
            "Connection"
        }
    }

    impl HeaderField for SecWebSocketKey {
        #[inline(always)]
        fn field(&_self) -> &'static str {
            "Sec-WebSocket-Key"
        }
    }

    impl HeaderField for SecWebSocketAccept {
        #[inline(always)]
        fn field(&_self) -> &'static str {
            "Sec-WebSocket-Accept"
        }
    }

    impl HeaderField for SecWebSocketProtocol {
        #[inline(always)]
        fn field(&_self) -> &'static str {
            "Sec-WebSocket-Protocol"
        }
    }

    impl HeaderField for SecWebSocketVersion {
        #[inline(always)]
        fn field(&_self) -> &'static str {
            "Sec-WebSocket-Version"
        }
    }
}
