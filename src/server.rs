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

use std::ascii::AsciiExt;
use std::io;
use std::cell::RefCell;
use std::rc::Rc;

use avia::{Ticket, BatchTick, StoreTick, Solution};

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

            	let data = self.data.clone();
     
                let send = req.body().concat2().map(move |b| {

				    if let Ok(batch) = serde_json::from_slice::<BatchTick>(b.as_ref()) {
					    		
					    let mut store = data.borrow_mut();
					    		
					    match store.poll().unwrap() {
					    	Async::Ready(_) => {
					    		store.insert(batch.clone());					    	
							    println!("Store:\n{}", store);
					    	}
					    	_ => {}
					    }		
					    Response::new().with_status(StatusCode::Ok)						    						    					   					    				  
				    } else {
				    	Response::new().with_status(StatusCode::NoContent)
				    }					 
                });
                
                Box::new(send)
            },           

            _ => {
                Box::new(futures::future::ok(
                    Response::new().with_status(StatusCode::NotFound)
                ))
            }
            
        }
    }
}

pub fn run_server() {
	let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(Server{data: 
    										   Rc::new(RefCell::new(StoreTick::new(100)))})).unwrap();
    server.run().unwrap();
}
