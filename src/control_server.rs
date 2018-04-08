use rocket::request::Form;

use super::{add_block, broadcast_latest_block, connect_to_peers, generate_next_block, BLOCKCHAIN,
            PEERS};
use rocket;

#[derive(FromForm)]
struct Block {
    data: String,
}

#[derive(FromForm)]
struct Peer {
    peer: String,
}

#[get("/blocks")]
fn blocks() -> String {
    format!("{:#?}", *BLOCKCHAIN.read().unwrap())
}

#[post("/mine-block", data = "<form>")]
fn mine_block(form: Form<Block>) {
    let new_block = generate_next_block(&form.get().data);
    println!("New block:\n{:#?}", new_block);
    add_block(new_block);
    broadcast_latest_block();
}

#[get("/peers")]
fn peers() -> String {
    format!("{:?}", *PEERS.read().unwrap())
}

#[post("/add-peer", data = "<form>")]
fn add_peer(form: Form<Peer>) {
    let peer_address = form.get().peer.clone();
    connect_to_peers(vec![peer_address]);
}

pub fn start() {
    rocket::ignite()
        .mount("/", routes![blocks, mine_block, peers, add_peer])
        .launch();
}
