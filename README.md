# http-server
Rust implementation of http-server able to upload batches of ticket and search need ticket in the uploaded batches

### Dependencies
hyper = "0.11.18"
futures = "0.1.14"
serde = "1.0.27"
serde_json = "1.0.9"
serde_derive = "1.0.27"

- [hyper](https://crates.io/crates/hyper) is used to HTTP implementation
- [futures](https://docs.rs/futures/0.1.18/futures/) is used to provide a robust implementation of handling asynchronous computations
- [serde](https://crates.io/crates/serde) is used to serialize and deserialize Rust data structures 
- [serde_json](https://crates.io/crates/serde_json) is used with [serde](https://crates.io/crates/serde)
- [serde_derive](https://crates.io/crates/hyper) is used with [serde](https://crates.io/crates/serde)






