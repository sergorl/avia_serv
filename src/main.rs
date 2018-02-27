extern crate avia;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate hyper;
extern crate futures;

use avia::{Ticket, BatchTick, StoreTick, Solution};

use futures::future::Future;
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

fn main() {
	let json1 = b"{ 
			        \"id\": \"51e91cabbc513365f132b449742220d3\",
			        \"departure_code\": \"LED\",
			        \"arrival_code\": \"DME\",
			        \"departure_time\": 10,
			        \"arrival_time\": 13,
			        \"price\": 1500
			    }";

	let json2 = b"{
					\"id\": \"900b49120b93d07b2f69316a843abba1\",
					\"departure_code\": \"DME\",
					\"arrival_code\": \"AER\",
					\"departure_time\": 17,
					\"arrival_time\": 18,
					\"price\": 2000
				}";

	let json3 = b"{
					\"id\": \"911b49120b93d07b2f69316a843abba1\",
					\"departure_code\": \"DME\",
					\"arrival_code\": \"AER\",
					\"departure_time\": 6,
					\"arrival_time\": 15,
					\"price\": 2200
				}";

	let json4 = b"{
					\"id\": \"917b49120b93d07b2f69316a843abba1\",
					\"departure_code\": \"AER\",
					\"arrival_code\": \"DME\",
					\"departure_time\": 1,
					\"arrival_time\": 2,
					\"price\": 1700
				}";

	let json5 = b"{
					\"id\": \"926b49120b93d07b2f69316a843abba1\",
					\"departure_code\": \"AER\",
					\"arrival_code\": \"DME\",
					\"departure_time\": 20,
					\"arrival_time\": 24,
					\"price\": 1900
				}";

	let json6 = b"{
					\"id\": \"810a49120b93d07b2f69316a843abba1\",
					\"departure_code\": \"AER\",
					\"arrival_code\": \"DME\",
					\"departure_time\": 10,
					\"arrival_time\": 12,
					\"price\": 2330
				}";			

	let json7 = b"{
					\"id\": \"811a49120b93d07b2f69316a843abba1\",
					\"departure_code\": \"DME\",
					\"arrival_code\": \"ABU\",
					\"departure_time\": 16,
					\"arrival_time\": 18,
					\"price\": 1000
				}";				

	let json8 = b"{
					\"id\": \"812a49120b93d07b2f69316a843abba1\",
					\"departure_code\": \"ABU\",
					\"arrival_code\": \"USA\",
					\"departure_time\": 22,
					\"arrival_time\": 24,
					\"price\": 2000
				}";									

	let tick1: Ticket = serde_json::from_slice(json1).unwrap();
	let tick2: Ticket = serde_json::from_slice(json2).unwrap();
	let tick3: Ticket = serde_json::from_slice(json3).unwrap();
	let tick4: Ticket = serde_json::from_slice(json4).unwrap();
	let tick5: Ticket = serde_json::from_slice(json5).unwrap();
	let tick6: Ticket = serde_json::from_slice(json6).unwrap();
	let tick7: Ticket = serde_json::from_slice(json7).unwrap();
	let tick8: Ticket = serde_json::from_slice(json8).unwrap();

	let mut tickets = StoreTick::new(8);

	tickets.insert(BatchTick::new(vec![tick1, tick2, tick3, tick4, 
		                               tick5, tick6, tick7, tick8]));


	for paths in vec![tickets.search("LED", "DME", 8, 14),
	                  tickets.search("LED", "AER", 8, 12),
	                  tickets.search("AER", "USA", 10, 12),
	                  tickets.search("AER", "BUD", 10, 12)] {
		
		match paths {
			Some(solution) => {print!("{}", solution);}
			None           => {
				println!("Sorry, No find :(");
			}
		}	                 	
	}
}	