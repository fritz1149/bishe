use std::env;
use std::sync::Mutex;
use lazy_static::lazy_static;
use tokio::join;
use once_cell::sync::OnceCell;
use crate::config::profile_config::CONFIG;
use crate::daemon::daemon_main;
use crate::model::{MonitorConfig};

mod config;
mod model;
mod daemon;
mod tasks;
mod fetch;

lazy_static! {
    pub static ref AUTHENTICATION: String = env::args().nth(1).unwrap_or_else(|| panic!("未输入监控凭据作为命令行参数"));
}
static DAEMON_STATE: OnceCell<MonitorConfig> = OnceCell::new();

#[tokio::main]
async fn main() {
    // 使用一次HOSTNAME，若读取不到直接退出
    println!("authentication: {}", AUTHENTICATION.as_str());

    DAEMON_STATE.set(tasks::request_config()).unwrap();
    daemon_main().join();
}
