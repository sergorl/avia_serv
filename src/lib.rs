extern crate hyper;
extern crate futures;

extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use serde_json::{Value, Error};

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


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ticket<'t> {
	id:              &'t str,
	departure_code:  &'t str,
	arrival_code:    &'t str,
	departure_time:  u64,
	arrival_time:    u64,
	price:           f64,
}

impl<'t> Ticket<'t> {		
	fn get_from(&self) -> &'t str {
		self.departure_code
	}

	fn get_to(&self) -> &'t str {
		self.arrival_code
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

	fn get_id(&self) -> &'t str {
		self.id
	}
}

fn cmp_tick(one: & Rc<Ticket>, other: & Rc<Ticket>) -> Ordering {
		one.get_dep_time().cmp(&other.get_dep_time())
}

/// Trait to display Ticket
impl<'t> fmt::Display for Ticket<'t> {
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


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BatchTick<'t> {
	#[serde(borrow)]
	data: Vec<Ticket<'t>>, 
}

impl<'t> BatchTick<'t> {
	pub fn new(batch: Vec<Ticket<'t>>) -> BatchTick<'t> {
		BatchTick{data: batch}
	}

	fn size(&self) -> usize {
		self.data.len()
	}

	fn get_data(self) -> Vec<Ticket<'t>> {
		self.data
	}
}


#[derive(Debug)]
struct Path<'t> {
	path:  LinkedList<Rc<Ticket<'t>>>,
	price: f64,
}

impl<'t> Path<'t> {
	fn new(path: LinkedList<Rc<Ticket<'t>>>) -> Path<'t> {
		let price = path.iter()
		                .fold(0f64, |acc, tick| acc + tick.get_price()); 

		Path{path:  path, price: price}
	}
}

/// Trait to display Path
impl<'t> fmt::Display for Path<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    	      
       	let mut i = 0;

		for ticket in &self.path {
			write!(f, "id{}  : {}\n", i, ticket.get_id());	
			i += 1;
		}       
        
        write!(f, "price: {}\n\n", self.price);
    

        Ok(())
    }
}

#[derive(Debug)]
struct Paths<'t> {
	paths: LinkedList<Path<'t>>,	
}

impl<'t> Paths<'t> {
	fn new() -> Paths<'t> {
		Paths{paths: LinkedList::new()}
	}

	fn add(&mut self, path: Path<'t>) {
		self.paths.push_front(path);
	}

	fn is_empty(&self) -> bool {
		self.paths.is_empty()
	}
}


/// Trait to display Path
impl<'t> fmt::Display for Paths<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       
       	let mut i = 0;

		for path in &self.paths {
			write!(f, "Path ##{}\n{}", i, path);	
			i += 1;
		}       
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct Solution<'t> {
	paths: LinkedList<Paths<'t>>,
	from: &'t str,
	to: &'t str,
}


/// Trait to display Path
impl<'t> fmt::Display for Solution<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       
       	write!(f, "________________________ All paths from {} to {}: ________________________\n", self.from, self.to);

       	if self.paths.is_empty() {
       		write!(f, "Empty :( No find.");	
       	} else {

	       	let mut i = 0;

			for path in &self.paths {
				write!(f, "-------------------------- Group of Paths #{} --------------------------\n{}", i, path);	
				i += 1;
			}       		
        }

        write!(f, "--------------------------------------------------------------------------\n");


        Ok(())
    }
}

impl<'t> Solution<'t> {
	fn new(from: &'t str, to: &'t str) -> Solution<'t> {
		Solution{paths: LinkedList::new(),
		         from:  from,
		         to:    to}
	}

	fn add(&mut self, paths: Paths<'t>) {
		self.paths.push_front(paths);
	}

	fn is_empty(&self) -> bool {
		self.paths.is_empty()
	}

	pub fn info(&self) -> String {
		format!("Path from {} to {}", self.from, self.to)
	}
}


#[derive(Debug)]
pub struct StoreTick<'s> {
	tickets: HashMap<&'s str, LinkedList<Rc<Ticket<'s>>>>,
}


/// Trait for display StoreTick
impl<'s> fmt::Display for StoreTick<'s> {
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


impl<'s> StoreTick<'s> {

	pub fn new(size: usize) -> StoreTick<'s> {

		let mut tickets: HashMap<&'s str, LinkedList<Rc<Ticket<'s>>>> = HashMap::with_capacity(size);		

		StoreTick {tickets: tickets}
			       
	}

	pub fn insert(&mut self, batch: BatchTick<'s>) {

		let mut from: &str;

		for ticket in batch.data {
						
			from = ticket.get_from();
			
			if self.tickets.contains_key(from) {
				if let Some(key) = self.tickets.get_mut(from) {
					(*key).push_back(Rc::new(ticket));				
				}
			} else {
				self.tickets.insert(from, LinkedList::new());
				if let Some(key) = self.tickets.get_mut(from) {
					(*key).push_back(Rc::new(ticket));				
				}
			}
		}

		self.tickets = sort(&self.tickets, &cmp_tick);		

	}
	
	pub fn search_from(&self, from: &'s str, start_dep: u64, finish_dep: u64) -> LinkedList<Rc<Ticket<'s>>> {

		if start_dep >= finish_dep {
			panic!("Start time should less finish time!");
		}

		let mut find_tickets: LinkedList<Rc<Ticket<'s>>> = LinkedList::new();

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

		if let Some(tickets) = self.tickets.get(from) {

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

	pub fn search(&mut self, from: &'s str, to: &'s str, start_dep: u64, finish_dep: u64) -> Option<Solution<'s>> {

		let mut paths = Solution::new(from, to);

		let start_tickets = self.search_from(from, start_dep, finish_dep);

		for ticket in &start_tickets {
			match self.find_paths(ticket.clone(), to) {
				Some(path) => {paths.add(path);}
				None       => {continue;}
			}			
		}

		if paths.is_empty() {
			None
		} else {
			Some(paths)
		}		 
	}

	fn find_paths(&self, start_ticket: Rc<Ticket<'s>>, finish: &'s str) -> Option<Paths<'s>> {

		fn recover_path<'s>(parents: &mut HashMap<&'s str, Rc<Ticket<'s>>>, node: Rc<Ticket<'s>>) -> Path<'s>  {

			let mut path: LinkedList<Rc<Ticket<'s>>> = LinkedList::new();
			path.push_front(node.clone());

			let mut keys_to_delete: LinkedList<&'s str> = LinkedList::new();

			let mut key = node.get_id();

			while let Some(parent) = parents.get(key) {
				path.push_front(parent.clone());										
				key = parent.get_id();

				keys_to_delete.push_front(key);
			}

			for key in keys_to_delete {
				parents.remove(key);
			}
	
			Path::new(path)
		}

		// Find paths with DFS
		let nodes = &self.tickets;

		let mut paths = Paths::new();
		
		let mut stack: LinkedList<Rc<Ticket<'s>>> = LinkedList::new();
		stack.push_front(start_ticket.clone());

		let mut parents: HashMap<&'s str, Rc<Ticket<'s>>> = HashMap::new();

		let mut node: Rc<Ticket<'s>>;
		let mut next_key: &'s str;

		while !stack.is_empty() {
			node = stack.pop_front().unwrap();

			next_key = node.get_to();

			if next_key == finish {				
				let path = recover_path(&mut parents, node.clone());				
				paths.add(path);
			}

			if let Some(children) = nodes.get(next_key) {

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
