#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate blockchain_rs;

#[get("/blocks")]
fn blocks() -> String {
    format!("{:#?}", *blockchain_rs::BLOCKCHAIN)
}

fn main() {
    rocket::ignite().mount("/", routes![blocks]).launch();
}
