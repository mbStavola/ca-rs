#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

extern crate rocket;
extern crate ws;
extern crate redis;

mod game;
mod player;
mod action;

use game::{Config, Game, SharedGameState, SharedPlayerList};

use std::env;
use std::thread;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() {
    let client: redis::Client;
    {
        let host = env::var("REDIS_HOST").unwrap_or(String::from("ca-rs-redis"));
        let port = env::var("REDIS_PORT").unwrap_or(String::from("6379"));

        let uri = format!("redis://{}:{}", host, port);
        client = redis::Client::open(uri.as_str()).expect("Could not create redis client.");
    }

    let websocket_uri: String;
    {
        let host = env::var("WEBSOCKET_HOST").unwrap_or(String::from(""));
        let port = env::var("WEBSOCKET_PORT").unwrap_or(String::from(""));

        websocket_uri = format!("{}:{}", host, port);
    }

    let config = Config::new(3, 15, 15);

    let state = SharedGameState::new();

    let watching = SharedPlayerList::new();
    let playing = SharedPlayerList::new();

    let _ = thread::Builder::new().name("game_thread".to_owned()).spawn(move || {
        ws::listen(websocket_uri.as_str(), |out| {
            let state = state.clone();

            let watching = watching.clone();
            let playing = playing.clone();

            Game::new(out, &client, &config, state, watching, playing)
        }).expect("Could not start websocket server.");
    }).unwrap();

    rocket::ignite().mount("/", routes![index]).launch();
}