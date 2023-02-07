extern crate core;

mod controller;
mod model;
mod service;
mod config;
mod orm;
mod daemon;

use std::iter::Map;
use axum::{routing::get, Router, Extension};
use fast_log::Config;
use log::debug;
use rbatis::Rbatis;
use tokio::runtime::Handle;
use crate::config::sqlite_config::RB;
use crate::daemon::main::daemon_main;
use std::thread;

#[tokio::main]
async fn main() {
    fast_log::init(Config::new().console().chan_len(Some(100000))).unwrap();
    config::sqlite_config::create_table(&*RB.lock().await).await;
    let sender = daemon_main();
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/topo", controller::topo_controller::api())
        ;

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}