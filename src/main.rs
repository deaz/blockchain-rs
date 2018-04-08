extern crate blockchain_rs;

use std::thread;

use blockchain_rs::{start_control_server, start_p2p_server};

fn main() {
    let control_server_thread = thread::spawn(|| {
        start_control_server();
    });

    start_p2p_server();

    control_server_thread.join().unwrap();
}
