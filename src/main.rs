use std::env;
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::daemon::daemon_main;
use crate::model::DaemonState;
use crate::ws::connect;

mod config;
mod handler;
mod model;
mod ws;
mod daemon;

lazy_static! {
    pub static ref HOSTNAME: String = env::args().nth(1).unwrap_or_else(|| panic!("本程序需要pod所在hostname作为命令行参数"));
    pub static ref DAEMON_STATE: Mutex<DaemonState> = Mutex::new(DaemonState{targets:Vec::new()});
}

#[tokio::main]
async fn main() {
    // 使用一次HOSTNAME，若读取不到直接退出
    println!("hostname: {}", HOSTNAME.as_str());
    daemon_main();
    connect().await;
}
