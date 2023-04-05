use std::ffi::OsStr;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use regex::Regex;
use serde_json::{json, Value};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot::Sender;
use crate::config::profile_config::CONFIG;
use crate::DAEMON_STATE;
use crate::model::{DaemonState, NetEdgeInfo, Targets};
use crate::AUTHENTICATION;
use crate::monitor_strategy::{MonitorStrategy, parse_strategy};

pub fn daemon_main(should_write: UnboundedSender<Value>) {
    let daemon = move || {
        let monitor_type = CONFIG.monitor.monitor_type.clone();
        println!("monitor_type: {}" ,monitor_type);
        let strategy = parse_strategy(&monitor_type);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build().unwrap();
        loop {
            thread::sleep(Duration::from_secs(CONFIG.monitor.interval as u64));
            println!("守护进程⭐");
            // 应当调用一次iperf程序
            let daemon_state = DAEMON_STATE.lock().unwrap();
            if let &Targets::None = &daemon_state.targets {
                continue;
            }
            let response = strategy.exec(&rt, &daemon_state.targets);
            rt.block_on(async {
                should_write.send(json!({"type": &monitor_type, "data": response})).unwrap();
            });
        }
    };
    thread::spawn(daemon);
}