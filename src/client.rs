extern crate avia;

extern crate futures;
extern crate hyper;
extern crate tokio_core;

extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use avia::{Path, Solution};

use futures::{Future, Stream};
use tokio_core::reactor::Core;

use hyper::{Method, Request, Client, Body, Error};
use hyper::header::{ContentLength, ContentType};
use hyper::client::HttpConnector;
use serde_json::Value;


fn run_client() {

	let batch = json!({
						"tickets": [
						    {
						      "id":             "51e91cabbc513365f132b449742220d3",
						      "departure_code": "LED",
						      "arrival_code":   "DME",
						      "departure_time": 1509876000,
						      "arrival_time":   1509883200,
						      "price":          1500
						    },
						    {
						      "id":             "900b49120b93d07b2f69316a843abba1",
						      "departure_code": "DME",
						      "arrival_code":   "AER",
						      "departure_time": 1509904800,
						      "arrival_time":   1509915600,
						      "price":          2000
						    }]						
					}
	);

	let need = json!({
						"departure_code":       "LED",
						"arrival_code":         "AER",
						"departure_time_start": 1509840000,
						"departure_time_end":   1509926399,
					}
	);

	let mut core = Core::new().unwrap();
	let client = Client::new(&core.handle());

	let post1 = make_post(&client, "batch_insert", batch);
	let post2 = make_post(&client, "search", need);

	let res = post1.join(post2);

	core.run(res).unwrap();
}


fn make_post(client: &Client<HttpConnector, Body>, 
             opt:    &str, 
             data:   serde_json::Value) -> Box<Future<Item=(), Error=hyper::Error>>
{

	let uri = format!("http://127.0.0.1:8080/{}", opt).parse().unwrap();

	let mut req = Request::new(Method::Post, uri);

	req.headers_mut().set(ContentType::json());
	req.set_body(serde_json::to_vec(&data).unwrap());

	let post = client.request(req).and_then(|res| {
	    println!("POST: {}", res.status());

	    res.body().concat2().map(|b| {

	    	if let Ok(answer) = serde_json::from_slice::<Solution>(b.as_ref()) {	    		
	    		println!("---------------------- Solutions: ----------------------- {}", answer);			    						    					   					    				  
		    }			
	    })
	});

	Box::new(post)

}

fn main() {
    run_client();
}
