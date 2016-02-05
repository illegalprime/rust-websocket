extern crate url;
extern crate openssl;

use std::borrow::Borrow;

use self::openssl::ssl::SslContext;
use self::url::Url;
use super::Client;
use super::super::stream::WebSocketStream;
use super::super::dataframe::DataFrame;
use super::super::sender::Sender;
use super::super::receiver::Receiver;

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

    pub fn connect(&self) -> Client<DataFrame, Sender<WebSocketStream>, Receiver<WebSocketStream>> {
        unimplemented!();
    }
}
