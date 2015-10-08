//! Provides a trait for receiving data frames and messages.
//!
//! Also provides iterators over data frames and messages.
//! See the `ws` module documentation for more information.

use std::marker::PhantomData;
use ws::Message;
use result::WebSocketResult;

/// A trait for receiving data frames and messages.
pub trait Receiver<'d, D: 'd>: Sized {
	/// Reads a single data frame from this receiver.
	fn recv_dataframe(&'d mut self) -> WebSocketResult<D>;
	/// Returns the data frames that constitute one message.
	fn recv_message_dataframes(&'d mut self) -> WebSocketResult<Vec<D>>;

	/// Returns an iterator over incoming data frames.
	fn incoming_dataframes(&'d mut self) -> DataFrameIterator<'d, Self, D> {
		DataFrameIterator {
			inner: self,
			_dataframe: PhantomData
		}
	}
	/// Reads a single message from this receiver.
	fn recv_message<M, I>(&'d mut self) -> WebSocketResult<M>
		where M: Message<D, DataFrameIterator = I>, I: Iterator<Item = D> {

		let dataframes = try!(self.recv_message_dataframes());
		Message::from_dataframes(dataframes)
	}

	/// Returns an iterator over incoming messages.
	fn incoming_messages<M>(&'d mut self) -> MessageIterator<'d, Self, D, M>
		where M: Message<D> {

		MessageIterator {
			inner: self,
			_dataframe: PhantomData,
			_message: PhantomData
		}
	}
}

/// An iterator over data frames from a Receiver.
pub struct DataFrameIterator<'a, R, D>
	where R: 'a + Receiver<'a, D> {

	inner: &'a mut R,
	_dataframe: PhantomData<D>
}

impl<'a, R, D> Iterator for DataFrameIterator<'a, R, D>
	where R: for<'b> Receiver<'b, D> {

	type Item = WebSocketResult<D>;

	/// Get the next data frame from the receiver. Always returns `Some`.
	fn next(&mut self) -> Option<WebSocketResult<D>> {
		 Some(self.inner.recv_dataframe())
	}
}

/// An iterator over messages from a Receiver.
pub struct MessageIterator<'a, R, D, M>
	where R: 'a + Receiver<'a, D>, M: Message<D> {

	inner: &'a mut R,
	_dataframe: PhantomData<D>,
	_message: PhantomData<M>
}

impl<'a, R, D, M, I> Iterator for MessageIterator<'a, R, D, M>
	where R: for<'b> Receiver<'b, D>, M: Message<D, DataFrameIterator = I>, I: Iterator<Item = D> {
	
	type Item = WebSocketResult<M>;
	
	/// Get the next message from the receiver. Always returns `Some`.
	fn next(&mut self) -> Option<WebSocketResult<M>> {
		Some(self.inner.recv_message())
	}
}
