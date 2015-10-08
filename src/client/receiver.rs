//! The default implementation of a WebSocket Receiver.

use std::io::Read;
use hyper::buffer::BufReader;

use dataframe::{DataFrame, Opcode};
use result::{WebSocketResult, WebSocketError};
use ws::util::dataframe::read_dataframe;
use ws;

/// A Receiver that wraps a Reader and provides a default implementation using
/// DataFrames and Messages.
pub struct Receiver<'a, R> {
    inner: BufReader<R>,
    buffer: Vec<DataFrame<'a>>
}

impl<'a, R> Receiver<'a, R> {
    /// Create a new Receiver using the specified Reader.
    pub fn new(reader: BufReader<R>) -> Self {
        Receiver {
            inner: reader,
            buffer: Vec::new()
        }
    }
    /// Returns a reference to the underlying Reader.
    pub fn get_ref(&self) -> &BufReader<R> {
        &self.inner
    }
    /// Returns a mutable reference to the underlying Reader.
    pub fn get_mut(&mut self) -> &mut BufReader<R> {
        &mut self.inner
    }
}

impl<'r, R: Read> ws::Receiver<'r, DataFrame<'r>> for Receiver<'r, R> {
    /// Reads a single data frame from the remote endpoint.
    fn recv_dataframe(&'r mut self) -> WebSocketResult<DataFrame<'r>> {
        read_dataframe(&mut self.inner, false)
    }

    /// Returns the data frames that constitute one message.
    fn recv_message_dataframes(&'r mut self) -> WebSocketResult<Vec<DataFrame<'r>>> {
        let mut finished = if self.buffer.is_empty() {
            let first = try!(read_dataframe(&mut self.inner, false));

            if first.opcode == Opcode::Continuation {
                return Err(WebSocketError::ProtocolError(
                    "Unexpected continuation data frame opcode".to_string()
                ));
            }

            let finished = first.finished;
            self.buffer.push(first);
            finished
        }
        else {
            false
        };

        while !finished {
            let next = try!(read_dataframe(&mut self.inner, false));
            finished = next.finished;

            match next.opcode as u8 {
                // Continuation opcode
                0 => self.buffer.push(next),
                // Control frame
                8...15 => {
                    return Ok(vec![next]);
                }
                // Others
                _ => return Err(WebSocketError::ProtocolError(
                    "Unexpected data frame opcode".to_string()
                )),
            }
        }

        let buffer = self.buffer.clone();
        self.buffer.clear();

        Ok(buffer)
    }
}
