//! All supported websocket extensions.
use std::fmt;
use std::str::FromStr;
use result::{WebSocketResult, WebSocketError};

const INVALID_EXTENSION: &'static str = "Invalid Sec-WebSocket-Extensions extension name";

#[cfg(feature="deflate")]
pub mod deflate;

/// Used to define the extensions used in this connection.
#[derive(Eq,PartialEq,Debug,Clone)]
pub enum Extension {
	/// The `permessage-deflate` extension.
	/// Used to compress payloads on a per-message basis.
	#[cfg(feature="deflate")]
	Deflate(self::deflate::DeflateConfig),
	// TODO: skip validation when an unkown extension is used.
	/// A custom extension unknown to this crate.
	/// Using this will disable message validation relating to extensions
	/// and the user will have to decode the extension themselves.
	Custom(CustomExtension),
}

impl fmt::Display for Extension {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Extension::Custom(ref ext) => ext.fmt(f),
			#[cfg(feature="deflate")]
			Extension::Deflate(ref config) => {
				write!(f, "permessage-deflate")?;
				if let Some(()) = config.server_no_context_takeover {
					write!(f, "; server_no_context_takeover")?;
        }
				if let Some(()) = config.client_no_context_takeover {
					write!(f, "; client_no_context_takeover")?;
				}
				if let Some(size) = config.server_max_window_bits {
					write!(f, "; server_max_window_bits={}", size)?;
				}
				if let Some(size) = config.client_max_window_bits {
					write!(f, "; client_max_window_bits={}", size)?;
				}
				Ok(())
			}
		}
	}
}

impl FromStr for Extension {
	type Err = WebSocketError;

	fn from_str(s: &str) -> WebSocketResult<Extension> {
		let mut ext = s.split(';').map(|x| x.trim());
		match ext.next() {
			Some(ref e) if e == &"deflate" => {
				// parse a `permessage-deflate` extension
				unimplemented!();
			}
			Some(ref name) => {
				// parse a custom extension
				let params = ext.map(|x| {
					let mut pair = x.splitn(1, '=')
					                .map(|x| x.trim().to_string());

					Parameter {
						name: pair.next().unwrap(),
						value: pair.next(),
					}
				});
				let params = params.collect();
				Ok(Extension::Custom(CustomExtension {
				                         name: name.to_string(),
				                         params: params,
				                     }))
			}
			None => Err(WebSocketError::ProtocolError(INVALID_EXTENSION)),
		}
	}
}

/// A custom WebSocket extension
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct CustomExtension {
	/// The name of this extension
	pub name: String,
	/// The parameters for this extension
	pub params: Vec<Parameter>,
}

impl CustomExtension {
	/// Creates a new extension with the given name
	pub fn new(name: String) -> Self {
		CustomExtension {
			name: name,
			params: Vec::new(),
		}
	}
}

impl fmt::Display for CustomExtension {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		try!(write!(f, "{}", self.name));
		for param in &self.params {
			try!(write!(f, "; {}", param));
		}
		Ok(())
	}
}

/// A parameter for an custom Extension
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Parameter {
	/// The name of this parameter
	pub name: String,
	/// The value of this parameter, if any
	pub value: Option<String>,
}

impl Parameter {
	/// Creates a new parameter with the given name and value
	pub fn new(name: String, value: Option<String>) -> Parameter {
		Parameter {
			name: name,
			value: value,
		}
	}
}

impl fmt::Display for Parameter {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		try!(write!(f, "{}", self.name));
		if let Some(ref x) = self.value {
			try!(write!(f, "={}", x));
		}
		Ok(())
	}
}
