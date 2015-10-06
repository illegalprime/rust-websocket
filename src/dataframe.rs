//! Module containing the default implementation of data frames.

/// Represents a WebSocket data frame.
///
/// The data held in a DataFrame is never masked.
/// Masking/unmasking is done when sending and receiving the data frame,
use std::io::{Read, Write};

use result::{WebSocketResult, WebSocketError};

use ws::util::header as dfh;
use ws::util::mask;


#[derive(Debug, Clone, PartialEq)]
pub struct DataFrame {
	/// Whether or no this constitutes the end of a message
	pub finished: bool,
	/// The reserved portion of the data frame (RFC6455 5.2)
	pub reserved: [bool; 3],
	/// The opcode associated with this data frame
	pub opcode: Opcode,
	/// The payload associated with this data frame
	pub data: Vec<u8>,
}

impl DataFrame {
	/// Creates a new DataFrame.
	pub fn new(finished: bool, opcode: Opcode, data: Vec<u8>) -> DataFrame {
		DataFrame {
			finished: finished,
			reserved: [false; 3],
			opcode: opcode,
			data: data,
		}
	}
}

pub trait WritableDataFrame {
    #[inline(always)]
    fn is_last(&self) -> bool;

    #[inline(always)]
    fn reserved(&self) -> [bool; 3];

    #[inline(always)]
    fn opcode(&self) -> Opcode;

    #[inline(always)]
    fn data(&self) -> &[u8];

    fn write<W>(&self, writer: &mut W, mask: bool) -> WebSocketResult<()>
    where W: Write {
        let reserved = self.reserved();
        let mut flags = dfh::DataFrameFlags::empty();
        if self.is_last() {
            flags.insert(dfh::FIN);
        }
        if reserved[0] {
            flags.insert(dfh::RSV1);
        }
        if reserved[1] {
            flags.insert(dfh::RSV2);
        }
        if reserved[2] {
            flags.insert(dfh::RSV3);
        }

        let masking_key = if mask {
            Some(mask::gen_mask())
        } else {
            None
        };

        let header = dfh::DataFrameHeader {
            flags: flags,
            opcode: self.opcode() as u8,
            mask: masking_key,
            len: self.data().len() as u64,
        };

        try!(dfh::write_header(writer, header));

        match masking_key {
            Some(mask) => try!(writer.write_all(&mask::mask_data(mask, self.data())[..])),
            None => try!(writer.write_all(self.data())),
        }
        try!(writer.flush());
        Ok(())
    }
}

impl WritableDataFrame for DataFrame {
    fn opcode(&self) -> Opcode {
        return self.opcode;
    }

    fn is_last(&self) -> bool {
        return self.finished;
    }

    fn reserved(&self) -> [bool; 3] {
        return self.reserved;
    }

    fn data(&self) -> &[u8] {
        return &self.data[..];
    }
}

