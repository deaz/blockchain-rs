#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate blockchain_rs;
extern crate rocket;
extern crate ws;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use blockchain_rs::add_block;
use blockchain_rs::generate_next_block;
use rocket::request::Form;
use std::sync::RwLock;
use std::thread;
use ws::{connect, listen, Sender};

lazy_static! {
    static ref PEERS: RwLock<Vec<Sender>> = RwLock::new(vec![]);
}

#[derive(Serialize, Deserialize)]
enum Message {
    QueryLatest,
}

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
    format!("{:#?}", *blockchain_rs::BLOCKCHAIN)
}

#[post("/mine-block", data = "<form>")]
fn mine_block(form: Form<Block>) {
    let new_block = generate_next_block(&form.get().data);
    println!("New block:\n{:#?}", new_block);
    add_block(new_block);
    // TODO: broadcast message
}

#[get("/peers")]
fn peers() -> String {
    "peers will be here".to_owned()
}

#[post("/add-peer", data = "<form>")]
fn add_peer(form: Form<Peer>) {
    let peer_address = form.get().peer.clone();
    thread::spawn(move || {
        connect(peer_address, |_out| {
            |msg| {
                println!("Got message as client: {:?}", msg);
                if let ws::Message::Text(ref text) = msg {
                    let _message: Message = serde_json::from_str(&text).unwrap();
                }
                Result::Ok(())
            }
        }).unwrap()
    });
}

fn main() {
    thread::spawn(|| {
        rocket::ignite()
            .mount("/", routes![blocks, mine_block, peers, add_peer])
            .launch()
    });

    listen("0.0.0.0:3012", |out| {
        let message: String = serde_json::to_string(&Message::QueryLatest).unwrap();
        out.send(message).unwrap();
        PEERS.write().unwrap().push(out);
        |msg: ws::Message| {
            println!("Got message as server: {:?}", msg);
            Result::Ok(())
        }
    }).unwrap();
}
