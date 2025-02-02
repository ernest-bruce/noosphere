#![cfg(not(target_arch = "wasm32"))]

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate lazy_static;

pub mod builder;
mod client;
pub mod dht;
mod name_system;
mod records;
pub mod utils;
mod validator;

//#[cfg(feature = "api_server")]
pub mod server;

pub use builder::NameSystemBuilder;
pub use client::NameSystemClient;
pub use dht::{DHTConfig, DHTKeyMaterial, NetworkInfo, Peer};
pub use libp2p::{multiaddr::Multiaddr, PeerId};
pub use name_system::{NameSystem, BOOTSTRAP_PEERS};
pub use records::NSRecord;
pub use validator::Validator;
