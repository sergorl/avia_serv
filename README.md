# HTTP-server for storing and searching avia tickets
Rust implementation of http-server is able 
- to upload batches of ticket 
- to store the uploaded batches in special internal sorted order
- to search need ticket in the uploaded batches: 
search is based on [binary search algorithm](https://en.wikipedia.org/wiki/Binary_search_algorithm) and [DFS](https://en.wikipedia.org/wiki/Depth-first_search) with many author's improved modification 

### Dependencies
- [hyper](https://crates.io/crates/hyper) is used to HTTP implementation
- [futures](https://docs.rs/futures/0.1.18/futures/) is used to provide a robust implementation of handling asynchronous computations
- [serde](https://crates.io/crates/serde) is used to serialize and deserialize Rust data structures 
- [serde_json](https://crates.io/crates/serde_json) is used with [serde](https://crates.io/crates/serde)
- [serde_derive](https://crates.io/crates/hyper) is used with [serde](https://crates.io/crates/serde)

### Advantages
  Internal algorithms is able 
- to find a set of different paths between two nodes (if it exists) regardless of path length (or count of internal nodes) 
- and to provide a robust implementation of even cycles in graph

### Disadvantages
- Now library doesn't support parallel computations



