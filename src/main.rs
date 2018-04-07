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

use blockchain_rs::{add_block, generate_next_block, get_latest_block, is_valid_blockchain,
                    BLOCKCHAIN};
use rocket::request::Form;
use std::env;
use std::sync::RwLock;
use std::thread;
use ws::{connect, listen, Sender};

lazy_static! {
    static ref PEERS: RwLock<Vec<Sender>> = RwLock::new(vec![]);
}

#[derive(Serialize, Deserialize)]
enum Message {
    QueryLatest,
    QueryAll,
    ResponseBlockchain(Vec<blockchain_rs::Block>),
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
    format!("{:#?}", *BLOCKCHAIN)
}

#[post("/mine-block", data = "<form>")]
fn mine_block(form: Form<Block>) {
    let new_block = generate_next_block(&form.get().data);
    println!("New block:\n{:#?}", new_block);
    add_block(new_block);
    broadcast(&get_latest_response());
}

#[get("/peers")]
fn peers() -> String {
    format!(
        "{:?}",
        PEERS
            .read()
            .unwrap()
            .iter()
            .map(|peer| peer.connection_id())
            .collect::<Vec<_>>()
    )
}

#[post("/add-peer", data = "<form>")]
fn add_peer(form: Form<Peer>) {
    let peer_address = form.get().peer.clone();
    connect_to_peers(vec![peer_address]);
}

fn message_handler(msg: ws::Message, out: Sender) -> Result<(), ws::Error> {
    if let ws::Message::Text(ref text) = msg {
        let message: Message = serde_json::from_str(&text).unwrap();
        match message {
            Message::QueryLatest => {
                out.send(get_latest_response()).unwrap();
            }
            Message::QueryAll => {
                out.send(get_chain_response()).unwrap();
            }
            Message::ResponseBlockchain(block) => handle_blockchain_response(block),
        };
    }
    Result::Ok(())
}

fn get_chain_response() -> String {
    serde_json::to_string(&Message::ResponseBlockchain(
        BLOCKCHAIN.read().unwrap().to_vec(),
    )).unwrap()
}

fn get_latest_response() -> String {
    serde_json::to_string(&Message::ResponseBlockchain(vec![get_latest_block()])).unwrap()
}

fn get_query_all() -> String {
    serde_json::to_string(&Message::QueryAll).unwrap()
}

fn replace_chain(new_blocks: Vec<blockchain_rs::Block>) {
    if is_valid_blockchain(&new_blocks) && new_blocks.len() > BLOCKCHAIN.read().unwrap().len() {
        println!(
            "Received blockchain is valid. Replacing current blockchain with received blockchain"
        );
        *BLOCKCHAIN.write().unwrap() = new_blocks;
        broadcast(&get_latest_response());
    } else {
        println!("Received blockchain invalid");
    }
}

fn handle_blockchain_response(blockchain: Vec<blockchain_rs::Block>) {
    let received_blocks_count = blockchain.len();
    let latest_received_block = blockchain.last().unwrap().clone();
    let latest_block = get_latest_block();
    if latest_received_block.index > latest_block.index {
        println!(
            "Blockchain possibly behind. We got: {} Peer got: {}",
            latest_block.index, latest_received_block.index
        );
        if latest_block.hash == latest_received_block.previous_hash {
            println!("We can append the received block to our chain");
            BLOCKCHAIN
                .write()
                .unwrap()
                .push(latest_received_block.clone());
            broadcast(&get_latest_response())
        } else if received_blocks_count == 1 {
            println!("We have to query the chain from our peer");
            broadcast(&get_query_all());
        } else {
            println!("Received blockchain is longer than current blockchain");
            replace_chain(blockchain);
        }
    } else {
        println!("Received chain is no longer than current chain");
    }
}

fn broadcast(message: &str) {
    for peer in PEERS.read().unwrap().iter() {
        peer.send(message).unwrap();
    }
}

fn connect_to_peers(peers: Vec<String>) {
    for peer in peers {
        thread::spawn(move || {
            connect(peer, |out| {
                PEERS.write().unwrap().push(out.clone());
                move |msg| message_handler(msg, out.clone())
            }).unwrap();
        });
    }
}

fn main() {
    let t1 = thread::spawn(|| {
        rocket::ignite()
            .mount("/", routes![blocks, mine_block, peers, add_peer])
            .launch()
    });

    let t2 = thread::spawn(|| {
        listen("0.0.0.0:3012", |out| {
            PEERS.write().unwrap().push(out.clone());

            let message: String = serde_json::to_string(&Message::QueryLatest).unwrap();
            out.send(message).unwrap();
            PEERS.write().unwrap().push(out.clone());
            move |msg: ws::Message| message_handler(msg, out.clone())
        }).unwrap();
    });

    if let Ok(peers_string) = env::var("PEERS") {
        let peers: Vec<String> = peers_string.split(",").map(String::from).collect();
        connect_to_peers(peers);
    }

    t1.join().unwrap();
    t2.join().unwrap();
}
