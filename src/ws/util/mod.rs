//! Utility functions for various portions of Rust-WebSocket.

pub mod header;
pub mod mask;

use std::str::from_utf8;
use std::str::Utf8Error;
use std::io::Write;
use std::io::Error as IoError;

/// Transforms a u8 slice into an owned String
pub fn bytes_to_string(data: &[u8]) -> Result<String, Utf8Error> {
	let utf8 = try!(from_utf8(data));
	Ok(utf8.to_string())
}

pub trait Serialize {
    fn serialize<W>(&self, stream: &mut W) -> Result<(), IoError>
    where W: Write;
}
