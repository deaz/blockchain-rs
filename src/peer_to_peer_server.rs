use std::env;
use std::sync::RwLock;

use serde_json;
use url::Url;
use ws::{CloseCode, Factory, Handler, Handshake, Message as WsMessage, Result as WsResult, Sender,
         WebSocket};

use super::{get_latest_block, replace_chain, Block, BLOCKCHAIN};

lazy_static! {
    pub static ref PEERS: RwLock<Vec<String>> = RwLock::new(vec![]);
    static ref BROADCASTER: RwLock<Option<Sender>> = RwLock::new(None);
}

#[derive(Serialize, Deserialize)]
enum Message {
    QueryLatest,
    QueryAll,
    ResponseBlockchain(Vec<Block>),
}

struct Server;

impl Factory for Server {
    type Handler = MessageHandler;

    fn connection_made(&mut self, ws: Sender) -> Self::Handler {
        // client connection
        MessageHandler {
            out: ws,
            address: None,
        }
    }

    fn server_connected(&mut self, ws: Sender) -> Self::Handler {
        let message = serde_json::to_string(&Message::QueryLatest).unwrap();
        ws.send(message).unwrap();
        MessageHandler {
            out: ws,
            address: None,
        }
    }
}

struct MessageHandler {
    out: Sender,
    address: Option<String>,
}

impl Handler for MessageHandler {
    fn on_open(&mut self, shake: Handshake) -> WsResult<()> {
        let addr_str = shake
            .peer_addr
            .map_or(String::from(""), |addr| addr.to_string());
        self.address = Some(addr_str.clone());
        PEERS.write().unwrap().push(addr_str);
        Ok(())
    }

    fn on_message(&mut self, msg: WsMessage) -> WsResult<()> {
        if let WsMessage::Text(ref text) = msg {
            let message: Message = serde_json::from_str(&text).unwrap();
            match message {
                Message::QueryLatest => self.out.send(get_latest_response()),
                Message::QueryAll => self.out.send(get_chain_response()),
                Message::ResponseBlockchain(block) => handle_blockchain_response(block),
            }
        } else {
            Ok(())
        }
    }

    fn on_close(&mut self, _code: CloseCode, _reason: &str) {
        if let Some(ref address) = self.address {
            PEERS.write().unwrap().remove_item(address);
        }
    }
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

fn handle_blockchain_response(blockchain: Vec<Block>) -> WsResult<()> {
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
            broadcast(&get_query_all())
        } else {
            println!("Received blockchain is longer than current blockchain");
            replace_chain(blockchain);
            Ok(())
        }
    } else {
        println!("Received chain is no longer than current chain");
        Ok(())
    }
}

fn broadcast(message: &str) -> WsResult<()> {
    if let Some(ref sender) = *BROADCASTER.read().unwrap() {
        sender.send(message)
    } else {
        Ok(())
    }
}

pub fn connect_to_peers(peers: Vec<String>) {
    if let Some(ref sender) = *BROADCASTER.write().unwrap() {
        for peer in peers {
            sender.connect(Url::parse(&peer).unwrap()).unwrap();
        }
    }
}

pub fn broadcast_latest_block() {
    broadcast(&get_latest_response()).unwrap();
}

pub fn start() {
    let ws = WebSocket::new(Server).unwrap();
    *BROADCASTER.write().unwrap() = Some(ws.broadcaster());

    if let Ok(peers_string) = env::var("PEERS") {
        let peers: Vec<String> = peers_string.split(",").map(String::from).collect();
        connect_to_peers(peers);
    }

    ws.bind("0.0.0.0:3012").unwrap().run().unwrap();
}
