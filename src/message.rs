//! Module containing the default implementation for messages.

use std::io;
use std::iter::{Take, Repeat, repeat};
use result::{WebSocketResult, WebSocketError};
use dataframe::{DataFrame, Opcode};
use byteorder::{WriteBytesExt, BigEndian};
use ws::util::message::message_from_data;
use ws;

/// Represents a WebSocket message.
#[derive(PartialEq, Clone, Debug)]
pub enum Message {
	/// A message containing UTF-8 text data
	Text(String),
	/// A message containing binary data
	Binary(Vec<u8>),
	/// A message which indicates closure of the WebSocket connection.
	/// This message may or may not contain data.
	Close(Option<CloseData>),
	/// A ping message - should be responded to with a pong message.
	/// Usually the pong message will be sent with the same data as the
	/// received ping message.
	Ping(Vec<u8>),
	/// A pong message, sent in response to a Ping message, usually
	/// containing the same data as the received ping message.
	Pong(Vec<u8>),
}

impl<'d> ws::Message<DataFrame<'d>> for Message {
	type DataFrameIterator = Take<Repeat<DataFrame<'d>>>;
	/// Attempt to form a message from a series of data frames
	fn from_dataframes(frames: Vec<DataFrame>) -> WebSocketResult<Message> {
		let mut iter = frames.iter();
		
		let first = try!(iter.next().ok_or(WebSocketError::ProtocolError(
			"No dataframes provided".to_string()
		)));
		
		let mut data = first.data.clone().into_owned();
		
		if first.reserved != [false; 3] {
			return Err(WebSocketError::ProtocolError(
				"Unsupported reserved bits received".to_string()
			));
		}
		
		for dataframe in iter {
			if dataframe.opcode != Opcode::Continuation {
				return Err(WebSocketError::ProtocolError(
					"Unexpected non-continuation data frame".to_string()
				));
			}
			if dataframe.reserved != [false; 3] {
				return Err(WebSocketError::ProtocolError(
					"Unsupported reserved bits received".to_string()
				));
			}
			for i in dataframe.data.iter() {
				data.push(*i);
			}
		}
		
		message_from_data(first.opcode, data)
	}
	/// Turns this message into an iterator over data frames
	fn into_iter(self) -> Self::DataFrameIterator {
		// Just return a single data frame representing this message.
		let (opcode, data) = match self {
			Message::Text(payload) => (Opcode::Text, payload.into_bytes()),
			Message::Binary(payload) => (Opcode::Binary, payload),
			Message::Close(payload) => (
					Opcode::Close,
					match payload {
						Some(payload) => { payload.into_bytes().unwrap() }
						None => { Vec::new() }
					} 
			),
			Message::Ping(payload) => (Opcode::Ping, payload),
			Message::Pong(payload) => (Opcode::Pong, payload),
		};
		let dataframe = DataFrame::new(true, opcode, data);
		repeat(dataframe).take(1)
	}

    /// Turns this message into an iterator over references to dataframes
    fn iter(&self) -> Self::DataFrameIterator {
        unimplemented!();
    }
}

/// Represents data contained in a Close message
#[derive(PartialEq, Clone, Debug)]
pub struct CloseData {
	/// The status-code of the CloseData
	pub status_code: u16,
	/// The reason-phrase of the CloseData
	pub reason: String,
}

impl CloseData {
	/// Create a new CloseData object
	pub fn new(status_code: u16, reason: String) -> CloseData {
		CloseData {
			status_code: status_code,
			reason: reason,
		}
	}
	/// Convert this into a vector of bytes
	pub fn into_bytes(self) -> io::Result<Vec<u8>> {
		let mut buf = Vec::new();
		try!(buf.write_u16::<BigEndian>(self.status_code));
		for i in self.reason.as_bytes().iter() {
			buf.push(*i);
		}
		Ok(buf)
	}
}
