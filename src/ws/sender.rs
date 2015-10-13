//! Provides a trait for sending data frames and messages.
//!
//! See the `ws` module documentation for more information.

use ws::Message;
use result::WebSocketResult;

/// A trait for sending data frames and messages.
pub trait Sender<D> {
	/// Sends a single data frame using this sender.
	fn send_dataframe(&mut self, dataframe: &D) -> WebSocketResult<()>;

	/// Sends a single message using this sender.
	fn send_message<'m, M>(&mut self, message: &'m M) -> WebSocketResult<()>
    where M: Message<'m, D>, D: 'm {
		for ref dataframe in message.iter() {
			try!(self.send_dataframe(dataframe));
		}
		Ok(())
	}
}
