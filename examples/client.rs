#![cfg_attr(feature = "nightly", feature(scoped))]

extern crate websocket;

#[cfg(not(feature = "nightly"))]
fn main() {
	println!("This example is not available on stable rust");
}

#[cfg(feature = "nightly")]
fn main() {
	use std::thread;
	use std::sync::mpsc::channel;
	use std::io::stdin;

	use websocket::{Message, Sender, Receiver};
	use websocket::client::request::Url;
	use websocket::Client;

	let url = Url::parse("ws://127.0.0.1:2794").unwrap();
	
	println!("Connecting to {}", url);
	
	let request = Client::connect(url).unwrap(); 
	
	let response = request.send().unwrap(); // Send the request and retrieve a response
	
	println!("Validating response...");
	
	response.validate().unwrap(); // Validate the response

	println!("Successfully connected");
	
	let (mut sender, mut receiver) = response.begin().split();
	
	let (tx, rx) = channel();
	
	let tx_1 = tx.clone();
	
	let send_loop = thread::scoped(move || {
		loop {
			// Send loop
			let message = match rx.recv() {
				Ok(m) => m,
				Err(e) => {
					println!("Send Loop: {:?}", e);
					return;
				}
			};
			match message {
				Message::Close(_) => {
					let _ = sender.send_message(&message);
					// If it's a close message, just send it and then return.
					return;
				}
				_ => (),
			}
			// Send the message
			match sender.send_message(&message) {
				Ok(()) => (),
				Err(e) => {
					println!("Send Loop: {:?}", e);
					let _ = sender.send_message(&Message::Close(None));
					return;
				}
			}
		}
	});

	let receive_loop = thread::scoped(move || {
		// Receive loop
		for message in receiver.incoming_messages() {
			let message = match message {
				Ok(m) => m,
				Err(e) => {
					println!("Receive Loop: {:?}", e);
					let _ = tx_1.send(Message::Close(None));
					return;
				}
			};
			match message {
				Message::Close(_) => {
					// Got a close message, so send a close message and return
					let _ = tx_1.send(Message::Close(None));
					return;
				}
				Message::Ping(data) => match tx_1.send(Message::Pong(data)) {
					// Send a pong in response
					Ok(()) => (),
					Err(e) => {
						println!("Receive Loop: {:?}", e);
						return;
					}
				},
				// Say what we received
				_ => println!("Receive Loop: {:?}", message),
			}
		}
	});
	
	loop {
		let mut input = String::new();
	
		stdin().read_line(&mut input).unwrap();
		
		let trimmed = input.trim();
		
		let message = match trimmed {
			"/close" => {
				// Close the connection
				let _ = tx.send(Message::Close(None));
				break;
			}
			// Send a ping
			"/ping" => Message::Ping(b"PING".to_vec()),
			// Otherwise, just send text
			_ => Message::Text(trimmed.to_string()),
		};
		
		match tx.send(message) {
			Ok(()) => (),
			Err(e) => {
				println!("Main Loop: {:?}", e);
				break;
			}
		}
	}
	
	// We're exiting
	
	println!("Waiting for child threads to exit");
	
	let _ = send_loop.join();
	let _ = receive_loop.join();
	
	println!("Exited");
}
