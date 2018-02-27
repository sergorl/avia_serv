#[macro_use]
extern crate avia;

extern crate hyper;
extern crate futures;

extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::cmp::Ordering;
use serde_json::{Value, Error};

use std::collections::{HashMap, LinkedList};
use std::fmt;

use futures::future::Future;
use futures::{Async, Stream};
use std::sync::Arc;

use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode, Body, Chunk};

use std::io;
use std::cell::RefCell;
use std::rc::Rc;

use avia::{RequestTick, Ticket, BatchTick, StoreTick, Solution};

struct Server {
	data: Rc<RefCell<StoreTick>>,
}

impl Service for Server {
	
    type Request  = Request;
    type Response = Response;
    type Error    = hyper::Error;
    type Future   = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {

         match (req.method(), req.path()) {            

            (&Method::Post, "/batch_insert") => {
            	let send = insert(self.data.clone(), req);
            	send           
            }           

            (&Method::Post, "/search") => {
            	let send = search(self.data.clone(), req);
            	send                       
            }

            _ => {
                Box::new(futures::future::ok(
                    Response::new().with_status(StatusCode::BadRequest)
                ))
            }
            
        }
    }
}


fn insert(data: Rc<RefCell<StoreTick>>, req: Request) -> Box<Future<Item=Response, Error=hyper::Error>> {

	let send = req.body().concat2().map(move |b| {

		println!("Start to process...");

	    if let Ok(batch) = serde_json::from_slice::<BatchTick>(b.as_ref()) {

	    	println!("Batch is received by server.");
		    		
		    let mut store = data.borrow_mut();
		    		
		    match store.poll().unwrap() {
		    	Async::Ready(_) => {
		    		store.insert(batch.clone());
		    		println!("Batch is inserted to store of server.");					    	
				    println!("Store:\n{}", store);
		    	}
		    	_ => {}
		    }		
		    Response::new().with_status(StatusCode::Ok)						    						    					   					    				  
	    } else {
	    	println!("No received data.");
	    	Response::new().with_status(StatusCode::NoContent)
	    }					 
    });
                
    Box::new(send)

}

fn search(data: Rc<RefCell<StoreTick>>, req: Request) -> Box<Future<Item=Response, Error=hyper::Error>> {

	let send = req.body().concat2().map(move |b| {

	    if let Ok(need) = serde_json::from_slice::<RequestTick>(b.as_ref()) {
		    		
		    let mut store = data.borrow_mut();
		    		
		    match store.poll().unwrap() {
		    	Async::Ready(_) => {
		    		if let Some(solution) = store.search(need.get_from(), 
					    			                need.get_to(), 
					    			                need.get_start_time(), 
					    			                need.get_finish_time()) {

		    			Response::new().with_status(StatusCode::Ok)
		                               .with_body(serde_json::to_vec(&solution).unwrap())
		    		} else {
		    			Response::new().with_status(StatusCode::NotFound)
		    		}					    	
		    	}
		    	_ => Response::new().with_status(StatusCode::NotFound)
		    }		
	    } else {
	    	Response::new().with_status(StatusCode::NoContent)
	    }					 
    });

    Box::new(send)
}

pub fn run_server() {
	let addr = "127.0.0.1:8080".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(Server{data: 
    										   Rc::new(RefCell::new(StoreTick::new(10000)))})).unwrap();
    server.run().unwrap();
}
