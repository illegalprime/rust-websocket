//! Provides the Sec-WebSocket-Extensions header.

use hyper::header::{Header, HeaderFormat};
use hyper::header::parsing::{from_comma_delimited, fmt_comma_delimited};
use hyper;
use std::fmt;
use std::ops::Deref;
pub use extensions::Extension;

// TODO: check if extension name is valid according to spec

/// Represents a Sec-WebSocket-Extensions header
#[derive(PartialEq, Clone, Debug)]
pub struct WebSocketExtensions(pub Vec<Extension>);

impl Deref for WebSocketExtensions {
	type Target = Vec<Extension>;

	fn deref(&self) -> &Vec<Extension> {
		&self.0
	}
}

impl Header for WebSocketExtensions {
	fn header_name() -> &'static str {
		"Sec-WebSocket-Extensions"
	}

	fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<WebSocketExtensions> {
		from_comma_delimited(raw).map(WebSocketExtensions)
	}
}

impl HeaderFormat for WebSocketExtensions {
	fn fmt_header(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let WebSocketExtensions(ref value) = *self;
		fmt_comma_delimited(fmt, &value[..])
	}
}

impl fmt::Display for WebSocketExtensions {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.fmt_header(fmt)
	}
}

#[cfg(all(feature = "nightly", test))]
mod tests {
	use super::*;
	use hyper::header::Header;
	use test;
	#[test]
	fn test_header_extensions() {
		use header::Headers;
		let value = vec![b"foo, bar; baz; qux=quux".to_vec()];
		let extensions: WebSocketExtensions = Header::parse_header(&value[..]).unwrap();

		let mut headers = Headers::new();
		headers.set(extensions);

		assert_eq!(&headers.to_string()[..],
		           "Sec-WebSocket-Extensions: foo, bar; baz; qux=quux\r\n");
	}
	#[bench]
	fn bench_header_extensions_parse(b: &mut test::Bencher) {
		let value = vec![b"foo, bar; baz; qux=quux".to_vec()];
		b.iter(|| {
			       let mut extensions: WebSocketExtensions = Header::parse_header(&value[..])
			           .unwrap();
			       test::black_box(&mut extensions);
			      });
	}
	#[bench]
	fn bench_header_extensions_format(b: &mut test::Bencher) {
		let value = vec![b"foo, bar; baz; qux=quux".to_vec()];
		let val: WebSocketExtensions = Header::parse_header(&value[..]).unwrap();
		b.iter(|| {
			       format!("{}", val);
			      });
	}
}
