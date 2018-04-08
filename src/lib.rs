#![feature(plugin, custom_derive, vec_remove_item)]
#![plugin(rocket_codegen)]
extern crate chrono;
extern crate crypto;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate rocket;
extern crate serde;
extern crate serde_json;
extern crate url;
extern crate ws;

mod blockchain;
mod control_server;
mod peer_to_peer_server;

pub use blockchain::*;
pub use control_server::start as start_control_server;
pub use peer_to_peer_server::{broadcast_latest_block, connect_to_peers, start as start_p2p_server,
                              PEERS};
