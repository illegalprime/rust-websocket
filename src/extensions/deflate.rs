//! The `permessage-deflate` extension.
//! This can compress you payload data automatically on a per-message basis,
//! saving precious time in the air.

use extensions::Extension;

/// Configure the compressor / decompressor
#[derive(Eq,PartialEq,Debug,Clone)]
pub struct DeflateConfig {
	/// Informs the other peer not to use context takeover.
	/// `Some(())` prevents context takeover and `None` allows it.
	/// Context takeover requires allocating extra space to save the LZ77 sliding
	/// window between messages.
	pub server_no_context_takeover: Option<()>,
	/// Informs the peer that this client will not use context takeover.
	/// This also disables context takeover for client using this config.
	/// This disables the use of context takeover when compressing messages
	/// from this side, the connected peer might still send messages with this
	/// feature enabled. To stop this, set `server_no_context_takeover`.
	pub client_no_context_takeover: Option<()>,
	/// Limit the size of the peer's LZ77 sliding window.
	/// Extra memory has to be allocated to contain the sliding window, this value
	/// sets the maximum window size this client will accept.
	/// The value must be from 8 to 15 and indicates the base-2 logarithm of the
	/// window size.
	/// e.g. a value of 10 will set the max size to be: 2<sup>10</sup> = 1024 bytes
	/// You can disable the sliding window entirely with `server_no_context_takeover`
	pub server_max_window_bits: Option<u8>,
	/// Informs the peer this client will not use an LZ77 sliding window size
	/// that is greater than this value.
	/// This also sets the max window size for the client using this config.
	/// e.g. a value of 10 will limit the max size to: 2<sup>10</sup> = 1024 bytes
	pub client_max_window_bits: Option<u8>,
}

impl Default for DeflateConfig {
	fn default() -> Self {
		DeflateConfig {
			server_no_context_takeover: None,
			client_no_context_takeover: None,
			server_max_window_bits: None,
			client_max_window_bits: None,
		}
	}
}

impl Extension {
	/// Create the default configuration for the `permessage-deflate` extension.
	pub fn deflate() -> Self {
		Extension::Deflate(Default::default())
	}
}
