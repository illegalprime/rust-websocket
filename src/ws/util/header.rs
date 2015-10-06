//! Utility functions for reading and writing data frame headers.

use std::io::{Read, Write};
use result::{WebSocketResult, WebSocketError};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

bitflags! {
	/// Flags relevant to a WebSocket data frame.
	flags DataFrameFlags: u8 {
		const FIN = 0x80,
		const RSV1 = 0x40,
		const RSV2 = 0x20,
		const RSV3 = 0x10,
	}
}

/// Represents a data frame header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataFrameHeader {
	/// The bit flags for the first byte of the header.
	pub flags: DataFrameFlags, 
	/// The opcode of the header - must be <= 16.
	pub opcode: u8, 
	/// The masking key, if any.
	pub mask: Option<[u8; 4]>, 
	/// The length of the payload.
	pub len: u64
}

impl DataFrameHeader {
    fn new(fin: bool, reserved: [bool; 3], opcode: Opcode) -> Self {
        let mut flags = DataFrameFlags::empty();
        if fin {
            flags.insert(FIN);
        }
        if reserved[0] {
            flags.insert(RSV1);
        }
        if reserved[1] {
            flags.insert(RSV2);
        }
        if reserved[2] {
            flags.insert(RSV3);
        }

        let masking_key = if mask {
            Some(mask::gen_mask())
        } else {
            None
        };

        DataFrameHeader {
            flags: flags,
            opcode: opcode as u8,
            mask: masking_key,
            len: self.data().len() as u64,
        }
    }
}

/// Writes a data frame header.
pub fn write_header<W>(writer: &mut W, header: DataFrameHeader) -> WebSocketResult<()>
	where W: Write {

	if header.opcode > 0xF {
		return Err(WebSocketError::DataFrameError(
			"Invalid data frame opcode".to_string()
		));
	}
	if header.opcode >= 8 && header.len >= 126 {
		return Err(WebSocketError::DataFrameError(
			"Control frame length too long".to_string()
		));
	}
	
	// Write 'FIN', 'RSV1', 'RSV2', 'RSV3' and 'opcode'
	try!(writer.write_u8((header.flags.bits) | header.opcode));
	
	try!(writer.write_u8(
		// Write the 'MASK'
		if header.mask.is_some() { 0x80 } else { 0x00 } |
		// Write the 'Payload len'
		if header.len <= 125 { header.len as u8 }
		else if header.len <= 65535 { 126 }
		else { 127 }
	));
	
	// Write 'Extended payload length'
	if header.len >= 126 && header.len <= 65535 {
		try!(writer.write_u16::<BigEndian>(header.len as u16));
	}
	else if header.len > 65535 {
		try!(writer.write_u64::<BigEndian>(header.len));
	}
	
	// Write 'Masking-key'
	match header.mask {
		Some(mask) => try!(writer.write_all(&mask)),
		None => (),
	}
	
	Ok(())
}

/// Reads a data frame header.
pub fn read_header<R>(reader: &mut R) -> WebSocketResult<DataFrameHeader>
	where R: Read {

	let byte0 = try!(reader.read_u8());
	let byte1 = try!(reader.read_u8());
	
	let flags = DataFrameFlags::from_bits_truncate(byte0);
	let opcode = byte0 & 0x0F;
	
	let len = match byte1 & 0x7F {
		0...125 => (byte1 & 0x7F) as u64,
		126 => {
			let len = try!(reader.read_u16::<BigEndian>()) as u64;
			if len <= 125 {
				return Err(WebSocketError::DataFrameError(
					"Invalid data frame length".to_string()
				));
			}
			len
		}
		127 => {
			let len = try!(reader.read_u64::<BigEndian>());
			if len <= 65535 {
				return Err(WebSocketError::DataFrameError(
					"Invalid data frame length".to_string()
				));
			}
			len
		}
		_ => unreachable!(),
	};
	
	if opcode >= 8 {
		if len >= 126 {
			return Err(WebSocketError::DataFrameError(
				"Control frame length too long".to_string()
			));
		}
		if !flags.contains(FIN) {
			return Err(WebSocketError::ProtocolError(
				"Illegal fragmented control frame".to_string()
			));
		}
	}
	
	let mask = if byte1 & 0x80 == 0x80 {
		Some([
			try!(reader.read_u8()),
			try!(reader.read_u8()),
			try!(reader.read_u8()),
			try!(reader.read_u8())
		])
	}
	else {
		None
	};
	
	Ok(DataFrameHeader {
		flags: flags, 
		opcode: opcode, 
		mask: mask, 
		len: len
	})
}

/// Represents a WebSocket data frame opcode
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Opcode {
	/// A continuation data frame
	Continuation,
	/// A UTF-8 text data frame
	Text,
	/// A binary data frame
	Binary,
	/// An undefined non-control data frame
	NonControl1,
	/// An undefined non-control data frame
	NonControl2,
	/// An undefined non-control data frame
	NonControl3,
	/// An undefined non-control data frame
	NonControl4,
	/// An undefined non-control data frame
	NonControl5,
	/// A close data frame
	Close,
	/// A ping data frame
	Ping,
	/// A pong data frame
	Pong,
	/// An undefined control data frame
	Control1,
	/// An undefined control data frame
	Control2,
	/// An undefined control data frame
	Control3,
	/// An undefined control data frame
	Control4,
	/// An undefined control data frame
	Control5,
}

impl Opcode {
	/// Attempts to form an Opcode from a nibble.
	///
	/// Returns the Opcode, or None if the opcode is out of range.
	pub fn new(op: u8) -> Option<Opcode> {
		Some(match op {
			0 => Opcode::Continuation,
			1 => Opcode::Text,
			2 => Opcode::Binary,
			3 => Opcode::NonControl1,
			4 => Opcode::NonControl2,
			5 => Opcode::NonControl3,
			6 => Opcode::NonControl4,
			7 => Opcode::NonControl5,
			8 => Opcode::Close,
			9 => Opcode::Ping,
			10 => Opcode::Pong,
			11 => Opcode::Control1,
			12 => Opcode::Control2,
			13 => Opcode::Control3,
			14 => Opcode::Control4,
			15 => Opcode::Control5,
			_ => return None,
		})
	}
}

#[cfg(all(feature = "nightly", test))]
mod tests {
	use super::*;
	use test;
	#[test]
	fn test_read_header_simple() {
		let header = [0x81, 0x2B];
		let obtained = read_header(&mut &header[..]).unwrap();
		let expected = DataFrameHeader {
			flags: FIN, 
			opcode: 1, 
			mask: None, 
			len: 43
		};
		assert_eq!(obtained, expected);
	}
	#[test]
	fn test_write_header_simple() {
		let header = DataFrameHeader {
			flags: FIN, 
			opcode: 1, 
			mask: None, 
			len: 43
		};
		let expected = [0x81, 0x2B];
		let mut obtained = Vec::with_capacity(2);
		write_header(&mut obtained, header).unwrap();
		
		assert_eq!(&obtained[..], &expected[..]);
	}
	#[test]
	fn test_read_header_complex() {
		let header = [0x42, 0xFE, 0x02, 0x00, 0x02, 0x04, 0x08, 0x10];
		let obtained = read_header(&mut &header[..]).unwrap();
		let expected = DataFrameHeader {
			flags: RSV1, 
			opcode: 2, 
			mask: Some([2, 4, 8, 16]), 
			len: 512
		};
		assert_eq!(obtained, expected);
	}
	#[test]
	fn test_write_header_complex() {
		let header = DataFrameHeader {
			flags: RSV1, 
			opcode: 2, 
			mask: Some([2, 4, 8, 16]), 
			len: 512
		};
		let expected = [0x42, 0xFE, 0x02, 0x00, 0x02, 0x04, 0x08, 0x10];
		let mut obtained = Vec::with_capacity(8);
		write_header(&mut obtained, header).unwrap();
		
		assert_eq!(&obtained[..], &expected[..]);
	}
	#[bench]
	fn bench_read_header(b: &mut test::Bencher) {
		let header = vec![0x42u8, 0xFE, 0x02, 0x00, 0x02, 0x04, 0x08, 0x10];
		b.iter(|| {
			read_header(&mut &header[..]).unwrap();
		});
	}
	#[bench]
	fn bench_write_header(b: &mut test::Bencher) {
		let header = DataFrameHeader {
			flags: RSV1, 
			opcode: 2, 
			mask: Some([2, 4, 8, 16]), 
			len: 512
		};
		let mut writer = Vec::with_capacity(8);
		b.iter(|| {
			write_header(&mut writer, header).unwrap();
		});
	}
}
