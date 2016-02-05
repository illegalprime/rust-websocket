extern crate url;
extern crate openssl;

use std::borrow::Borrow;
use std::io::Error as IoError;
use std::net::{
    TcpStream,
    Ipv6Addr,
    Ipv4Addr,
};

use self::openssl::ssl::{
    SslContext,
    SslMethod,
    SslStream,
};
use self::openssl::ssl::error::SslError;
use self::url::{
    Url,
    Host,
};
use super::super::stream::WebSocketStream;
use super::super::dataframe::DataFrame;
use super::super::sender::Sender;
use super::super::receiver::Receiver;

pub type Client = super::Client<DataFrame, Sender<WebSocketStream>, Receiver<WebSocketStream>>;
pub type Request = super::request::Request<WebSocketStream, WebSocketStream>;

/// Build clients with a builder-style API
pub struct ClientBuilder<'u, 'p, 'e, 's> {
    url: &'u Url,
    protocols: Option<Vec<&'p Borrow<str>>>,
    extensions: Option<Vec<&'e Borrow<str>>>,
    ssl_context: Option<&'s SslContext>,
}

impl<'u, 'p, 'e, 's> ClientBuilder<'u, 'p, 'e, 's> {
    pub fn new(url: &'u Url) -> Self {
        ClientBuilder {
            url: url,
            protocols: None,
            extensions: None,
            ssl_context: None,
        }
    }

    pub fn protocols<I>(mut self, protocols: I) -> Self
    where I: IntoIterator<Item = &'p Borrow<str>>,
    {
        let mut collected = Vec::new();
        for protocol in protocols.into_iter() {
            collected.push(protocol);
        }
        self.protocols = Some(collected);
        self
    }

    pub fn extensions<I>(mut self, extensions: I) -> Self
    where I: IntoIterator<Item = &'e Borrow<str>>,
    {
        let mut collected = Vec::new();
        for extension in extensions.into_iter() {
            collected.push(extension);
        }
        self.extensions = Some(collected);
        self
    }

    pub fn ssl_context(mut self, context: &'s SslContext) -> Self {
        self.ssl_context = Some(context);
        self
    }

    pub fn prepare(&self) -> Result<Request, ConnErr> {
        // Get info about ports
        let is_ssl = &self.url.scheme == "wss";
        let port = if let Some(port) = self.url.port() {
            port
        } else {
            if is_ssl {
                443
            } else {
                80
            }
        };

        // Make the TcpStream!
        let stream = if let Some(host) = self.url.host() {
            match *host {
                Host::Domain(ref d) => TcpStream::connect((d as &str, port)),
                Host::Ipv6(ip) => TcpStream::connect((ip, port)),
                Host::Ipv4(ip) => TcpStream::connect((ip, port)),
            }
        } else {
            return Err(ConnErr::NoHost);
        };
        let stream = match stream {
            Ok(s) => s,
            Err(e) => return Err(ConnErr::TcpConnect(e)),
        };

        // Make the WebSocketStream!
        let stream = if is_ssl {
            let ssl_stream = if let Some(context) = self.ssl_context {
                SslStream::connect(context, stream)
            } else {
                let context = match SslContext::new(SslMethod::Tlsv1) {
                    Ok(c) => c,
                    Err(e) => return Err(ConnErr::MakingDefaultContext(e)),
                };
                SslStream::connect(&context, stream)
            };
            let ssl_stream = match ssl_stream {
                Ok(s) => s,
                Err(e) => return Err(ConnErr::SslConnect(e)),
            };
            WebSocketStream::Ssl(ssl_stream)
        } else {
            WebSocketStream::Tcp(stream)
        };

        unimplemented!();
    }

    pub fn connect(&self) -> Result<Client, ConnErr> {
        unimplemented!();
    }
}

pub enum ConnErr {
    NoHost,
    MakingDefaultContext(SslError),
    SslConnect(SslError),
    TcpConnect(IoError),
}
