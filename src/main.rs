extern crate core;

mod controller;
mod model;
mod service;
mod config;
mod orm;
mod daemon;
mod handler;

use std::iter::Map;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use axum::{routing::get, Router, Extension};
use fast_log::Config;
use log::debug;
use rbatis::Rbatis;
use tokio::runtime::Handle;
use crate::config::sqlite_config::RB;
use crate::daemon::main::{daemon_main, Signal};
use std::thread;
use lazy_static::lazy_static;
use log::LevelFilter::Debug;

lazy_static! {
    pub static ref TELL_DAEMON: Mutex<Sender<Signal>> = Mutex::new(daemon_main());
}

#[tokio::main]
async fn main() {
    fast_log::init(Config::new().console().chan_len(Some(100000)).level(Debug)).unwrap();
    config::sqlite_config::create_table().await;
    let tmp = &*TELL_DAEMON;
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/topo", controller::topo_controller::api())
        .nest("/flow", controller::flow_controller::api())
        .nest("/ws", controller::ws_controller::api())
        .nest("/signal", controller::signal_controller::api())
        ;

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}