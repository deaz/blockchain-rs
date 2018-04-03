#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate blockchain_rs;
extern crate rocket;

use blockchain_rs::add_block;
use blockchain_rs::generate_next_block;
use rocket::request::Form;

#[derive(FromForm)]
struct Block {
    data: String,
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

#[post("/add-peer")]
fn add_peer() {
    // TODO: add peer to list
}

fn main() {
    rocket::ignite()
        .mount("/", routes![blocks, mine_block, peers, add_peer])
        .launch();
}
