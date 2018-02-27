extern crate hyper;
extern crate futures;

extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use serde_json::{Value, Error};
use serde::ser::{Serialize, Serializer, SerializeStruct};

use std::collections::{HashMap, LinkedList, VecDeque};
use std::fmt;

use futures::future::Future;
use futures::{Async, Stream};
use std::sync::Arc;

use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode, Body, Chunk};

use std::ascii::AsciiExt;
use std::io;
use std::cell::Cell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::ops::FnMut;


type Str = Box<str>;

 #[macro_export]
macro_rules! str2str {
    ($x:expr) => {
        $x.to_string().into_boxed_str()   
    };
}

// -------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestTick {
	departure_code:       Str,
	arrival_code:         Str,
	departure_time_start: u64,
	departure_time_end:   u64,
}

impl RequestTick {

	pub fn get_from(&self) -> Str {
		self.departure_code.clone()
	}

	pub fn get_to(&self) -> Str{
		self.arrival_code.clone()
	}

	pub fn get_start_time(&self) -> u64 {
		self.departure_time_start
	}

	pub fn get_finish_time(&self) -> u64 {
		self.departure_time_end
	}
}

// -------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ticket {
	id:              Str,
	departure_code:  Str,
	arrival_code:    Str,
	departure_time:  u64,
	arrival_time:    u64,
	price:           f64,
}

impl Ticket {
	fn get_id(&self) -> Str {
		self.id.clone()
	}

	fn get_from(&self) -> Str {
		self.departure_code.clone()
	}

	fn get_to(&self) -> Str{
		self.arrival_code.clone()
	}

	fn get_dep_time(&self) -> u64 {
		self.departure_time
	}

	fn get_arv_time(&self) -> u64 {
		self.arrival_time
	}

	fn get_price(&self) ->f64 {
		self.price
	}
	
}

fn cmp_tick(one: &Rc<Ticket>, other: &Rc<Ticket>) -> Ordering {
		one.get_dep_time().cmp(&other.get_dep_time())
}

/// Trait to display Ticket
impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       
        write!(f, "id        : {}\ndeparture : {}\narrival   : {}\ndep_time  : {}\narriv_time: {}\nprice:    : {}\n", 
        	   self.id,
        	   self.departure_code, 
               self.arrival_code, 
               self.departure_time, 
               self.arrival_time,
               self.price);
       
        Ok(())
    }
}

// -------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BatchTick {
	tickets: Vec<Ticket>, 
}

impl BatchTick {
	pub fn new(batch: Vec<Ticket>) -> BatchTick {
		BatchTick{tickets: batch}
	}

	fn size(&self) -> usize {
		self.tickets.len()
	}

	fn get_data(self) -> Vec<Ticket> {
		self.tickets
	}
}

// -------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Path {
	ticket_ids: LinkedList<Str>,
	price:      f64,
}

/// Trait to display Path
impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

       	let mut i = 0;

		for id in &self.ticket_ids {
			write!(f, "id{}  : {}\n", i, id);	
			i += 1;
		}       
        
        write!(f, "price: {}\n", self.price);
    
        Ok(())
    }
}

// This is what #[derive(Serialize)] would generate
// impl Serialize for Path {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//         where S: Serializer
//     {
//         let mut s = serializer.serialize_struct("Path", 2)?;
//         s.serialize_field("ticket_ids", &self.ticket_ids)?;
//         s.serialize_field("price",      &self.price)?;
//         s.end()
//     }
// }

// -------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Solution {
	paths: LinkedList<Path>,
	from:  Str,
	to:    Str,
}

/// Trait to display Path
impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       
       	write!(f, "_____________________ All paths from {} to {}: ____________________\n", self.from, self.to);

       	if self.paths.is_empty() {
       		write!(f, "Empty :( No find.");	
       	} else {

	       	let mut i = 0;

			for path in &self.paths {
				write!(f, "------------------------------- Path #{} -----------------------------\n{}", i, path);	
				i += 1;
			}       		
        }

        write!(f, "---------------------------------------------------------------------\n\n");

        Ok(())
    }
}

impl Solution {
	fn new(from: Str, to: Str) -> Solution {
		Solution{paths: LinkedList::new(),
		         from:  from, 
		         to:    to}
	}

	fn add(&mut self, paths: &mut LinkedList<Path>) {
		self.paths.append(paths);
	}

	fn is_empty(&self) -> bool {
		self.paths.is_empty()
	}

	pub fn info(&self) -> String {
		format!("Path from {} to {}", self.from, self.to)
	}
}

#[derive(Debug)]
pub struct StoreTick {
	tickets: HashMap<Str, LinkedList<Rc<Ticket>>>,
	ready:   bool,
}

impl Future for StoreTick {
	type Item  = ();
	type Error = io::Error;

	fn poll(&mut self) -> Result<Async<()>, io::Error> {
		if self.ready {
			Ok(Async::Ready(()))
		} else {
			Ok(Async::NotReady)
		}
	}
}


/// Trait for display StoreTick
impl fmt::Display for StoreTick {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      
        write!(f, "---------------------- Tickets -----------------------\n");

        for (from, tickets) in self.tickets.iter() {

        	write!(f, "                         {}\n", from);

        	for ticket in tickets {
        		write!(f, "{} \n", ticket);
        	}

        	write!(f, "____________________________________________________\n");
        }
       
        Ok(())
    }
}


impl StoreTick {

	pub fn new(size: usize) -> StoreTick {

		let mut tickets: HashMap<Str, LinkedList<Rc<Ticket>>> = HashMap::with_capacity(size);		

		StoreTick {tickets: tickets, ready: true}			       
	}

	pub fn insert(&mut self, batch: BatchTick) {

		self.ready = false;	

		let mut from: Str;

		for ticket in batch.tickets {
						
			from = ticket.get_from();
			
			if !self.tickets.contains_key(&from) {
				self.tickets.insert(from.clone(), LinkedList::new());			
			} 

			if let Some(key) = self.tickets.get_mut(&from) {
				(*key).push_front(Rc::new(ticket));				
			}
			
		}

		self.tickets = sort(&self.tickets, &cmp_tick);
		self.ready = true;		

	}
	
	pub fn search_from(&self, from: Str, start_dep: u64, finish_dep: u64) -> LinkedList<Rc<Ticket>> {

		if start_dep >= finish_dep {
			panic!("Start time should less finish time!");
		}

		let mut find_tickets: LinkedList<Rc<Ticket>> = LinkedList::new();

		let cmp = |ticket: &Rc<Ticket>| {

			let time_dep = ticket.get_dep_time();

			if start_dep <= time_dep && time_dep <= finish_dep {
				Ordering::Equal
			} else if time_dep < start_dep {
				Ordering::Less			
			} else if time_dep > finish_dep {
				Ordering::Greater
			} else {
				// this code is never reached
				unimplemented!()
			}		
		};

		if let Some(tickets) = self.tickets.get(&from) {

			let mut tickets = to_vec(tickets);

			if let Ok(end) = tickets.binary_search_by(&cmp) {

				for i in (0..end+1).rev() {

					let ticket = tickets.get(i).unwrap();

					match cmp(ticket) {
						Ordering::Equal => {find_tickets.push_back(ticket.clone());}
						_               => {break;}
					}			
				}
			}
		}	

		find_tickets
	}

	pub fn search(&mut self, from: Str, to: Str, start_dep: u64, finish_dep: u64) -> Option<Solution> {

		// let from = str2str!(from);
		// let to   = str2str!(to);

		let mut paths = Solution::new(from.clone(), to.clone());

		let start_tickets = self.search_from(from.clone(), start_dep, finish_dep);

		for ticket in &start_tickets {

			match self.find_paths(ticket.clone(), to.clone()) {

				Some(mut path) => {paths.add(&mut path);}

				None           => {continue;}
			}			
		}

		if paths.is_empty() {
			None
		} else {
			Some(paths)
		}		 
	}

	fn find_paths(&self, start_ticket: Rc<Ticket>, finish: Str) -> Option<LinkedList<Path>> {

		fn recover_path(parents: &mut HashMap<Str, Rc<Ticket>>, node: Rc<Ticket>) -> Path  {

			let mut path: LinkedList<Str> = LinkedList::new();
			path.push_front(node.get_id());

			let mut keys_to_delete: LinkedList<Str> = LinkedList::new();

			let mut key   = node.get_id();
			let mut price = node.get_price();

			while let Some(parent) = parents.get(&key) {

				path.push_front(parent.get_id());		

				key    = parent.get_id();
				price += parent.get_price();

				keys_to_delete.push_front(key.clone());
			}

			for key in keys_to_delete {
				parents.remove(&key);
			}
	
			Path{ticket_ids: path, price: price}
		}

		// Find paths with DFS
		let nodes = &self.tickets;

		let mut paths: LinkedList<Path> = LinkedList::new();
		
		let mut stack: LinkedList<Rc<Ticket>> = LinkedList::new();
		stack.push_front(start_ticket.clone());

		let mut parents: HashMap<Str, Rc<Ticket>> = HashMap::new();

		let mut node: Rc<Ticket>;
		let mut next_key: Str;

		while !stack.is_empty() {
			node = stack.pop_front().unwrap();

			next_key = node.get_to();

			if next_key == finish {				
				let path = recover_path(&mut parents, node.clone());				
				paths.push_back(path);
			}

			if let Some(children) = nodes.get(&next_key) {

				for child in children {

					let arv_time = node.get_arv_time();
					let dep_time = child.get_dep_time();

					if arv_time + 3 < dep_time && dep_time < arv_time + 8 {
						stack.push_front(child.clone());
						parents.insert(child.get_id(), node.clone());
					}							
				}
			}			
		}

		if paths.is_empty() {
			None
		} else {
			Some(paths)
		}

	}
}

fn sort<K, T, F>(map: &HashMap<K, LinkedList<Rc<T>>>, cmp: &F) -> HashMap<K, LinkedList<Rc<T>>> 
where
	K: std::cmp::Eq + std::hash::Hash + Clone, 
	for<'r, 's> F:  Fn(&'r Rc<T>, &'s Rc<T>) -> Ordering  
{
	let mut new_map: HashMap<K, LinkedList<Rc<T>>> = HashMap::with_capacity(map.capacity());

	for (key, values) in map.iter() {
		new_map.insert(key.clone(), sort_work(values, cmp));
	}
	
	new_map 
}

fn sort_work<T, F>(list: &LinkedList<Rc<T>>, cmp: &F) -> LinkedList<Rc<T>> 
where 
	for<'r, 's> F:  Fn(&'r Rc<T>, &'s Rc<T>) -> Ordering  
{    
    let mut vec = to_vec(list);

    vec.sort_by(cmp);
    
    let mut sort_list: LinkedList<Rc<T>> = vec.into_iter().collect();

    sort_list
}

fn to_vec<T>(list: &LinkedList<Rc<T>>) -> Vec<Rc<T>> {

	let mut vec: Vec<Rc<T>> = Vec::with_capacity(list.len());

    for item in list {
    	vec.push(item.clone());
    }

    vec
}
