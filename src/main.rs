use std::env;
use std::sync::Mutex;
use lazy_static::lazy_static;
use tokio::join;
use crate::config::profile_config::CONFIG;
use crate::daemon::daemon_main;
use crate::model::DaemonState;
use crate::ws::connect;

mod config;
mod handler;
mod model;
mod ws;
mod daemon;
mod tasks;
mod monitor_strategy;

lazy_static! {
    pub static ref AUTHENTICATION: String = env::args().nth(1).unwrap_or_else(|| panic!("未输入监控凭据作为命令行参数"));
    pub static ref DAEMON_STATE: Mutex<DaemonState> = Mutex::new(DaemonState{targets:Vec::new()});
}

#[tokio::main]
async fn main() {
    // 使用一次HOSTNAME，若读取不到直接退出
    println!("authentication: {}", AUTHENTICATION.as_str());
    let (should_write, handle_read, handle_write) = connect().await;
    daemon_main(should_write);
    if let Err(text) = tasks::request_config().await {
        println!("请求配置失败：{}", text);
    }
    join!(handle_read, handle_write);
}
