extern crate websocket;
extern crate rustc_serialize as serialize;

use websocket::client::request::Url;
use websocket::{Client, Message, Sender, Receiver};
use websocket::message::CloseData;
use serialize::json;

fn main() {
	let addr = "ws://127.0.0.1:9001".to_string();
	let agent = "rust-websocket";
	
	println!("Using fuzzingserver {}", addr);
	println!("Using agent {}", agent);
	
	println!("Running test suite...");
	
	let mut current_case_id = 1;
	let case_count = get_case_count(addr.clone());
	
	while current_case_id <= case_count {
		let url = addr.clone() + "/runCase?case=" + &current_case_id.to_string()[..] + "&agent=" + agent;
		
		let ws_uri = Url::parse(&url[..]).unwrap();
		let request = Client::connect(ws_uri).unwrap();
		let response = request.send().unwrap();
		match response.validate() {
			Ok(()) => (),
			Err(e) => {
				println!("{:?}", e);
				current_case_id += 1;
				continue;
			}
		}
		let (mut sender, mut receiver) = response.begin().split();
		
		println!("Executing test case: {}/{}", current_case_id, case_count);
		
		for message in receiver.incoming_messages() {
			let message = match message {
				Ok(message) => message,
				Err(e) => {
					println!("Error: {:?}", e);
					let _ = sender.send_message(&Message::Close(None));
					break;
				}
			};
			
			match message {
				Message::Text(data) => {
					sender.send_message(&Message::Text(data)).unwrap();
				}
				Message::Binary(data) => {
					sender.send_message(&Message::Binary(data)).unwrap();
				}
				Message::Close(_) => {
					let _ = sender.send_message(&Message::Close(None));
					break;
				}
				Message::Ping(data) => {
					sender.send_message(&Message::Pong(data)).unwrap();
				}
				_ => (),
			}
		}
		
		current_case_id += 1;
	}
	
	update_reports(addr.clone(), agent);
}

fn get_case_count(addr: String) -> usize {
	let url = addr + "/getCaseCount";
	let ws_uri = Url::parse(&url[..]).unwrap();
	let request = Client::connect(ws_uri).unwrap();
	let response = request.send().unwrap();
	match response.validate() {
		Ok(()) => (),
		Err(e) => {
			println!("{:?}", e);
			return 0;
		}
	}
	let (mut sender, mut receiver) = response.begin().split();
	
	let mut count = 0;
	
	for message in receiver.incoming_messages() {
		let message = match message {
			Ok(message) => message,
			Err(e) => {
				println!("Error: {:?}", e);
				let _ = sender.send_message(&Message::Close(Some(CloseData::new(1002, "".to_string()).unwrap())));
				break;
			}
		};
		match message {
			Message::Text(data) => {
				count = json::decode(&data[..]).unwrap();
				println!("Will run {} cases...", count);
			}
			Message::Close(_) => {
				let _ = sender.send_message(&Message::Close(None));
				break;
			}
			Message::Ping(data) => {
				sender.send_message(&Message::Pong(data)).unwrap();
			}
			_ => (),
		}
	}
	
	count
}

fn update_reports(addr: String, agent: &str) {
	let url = addr + "/updateReports?agent=" + agent;
	let ws_uri = Url::parse(&url[..]).unwrap();
	let request = Client::connect(ws_uri).unwrap();
	let response = request.send().unwrap();
	match response.validate() {
		Ok(()) => (),
		Err(e) => {
			println!("{:?}", e);
			return;
		}
	}
	let (mut sender, mut receiver) = response.begin().split();
	
	println!("Updating reports...");
	
	for message in receiver.incoming_messages() {
		let message = match message {
			Ok(message) => message,
			Err(e) => {
				println!("Error: {:?}", e);
				let _ = sender.send_message(&Message::Close(None));
				return;
			}
		};
		match message {
			Message::Close(_) => {
				let _ = sender.send_message(&Message::Close(None));
				println!("Reports updated.");
				println!("Test suite finished!");
				return;
			}
			Message::Ping(data) => {
				sender.send_message(&Message::Pong(data)).unwrap();
			}
			_ => (),
		}
	}
}
