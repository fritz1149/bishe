use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use crate::config::profile_config::CONFIG;
use crate::DAEMON_STATE;
use crate::model::{DaemonState, NetInfo};

pub fn daemon_main() {
    let daemon = || {
        loop {
            thread::sleep(Duration::from_secs(CONFIG.edge.interval as u64));
            println!("守护进程⭐");
            // 应当调用一次iperf程序
            let daemon_state = DAEMON_STATE.lock().unwrap();
            for target in daemon_state.targets.iter() {
                println!("{}:", target.hostname);
                get_info(&target.hostname);
            }
        }
    };
    thread::spawn(daemon);
}

const NET_MEASURE_FAILED: &str = "网络参数测量失败";
// fn get_info(hostname: String) -> Result<NetInfo, &'static str> {
fn get_info(hostname: &String) {
    let output = Command::new("iperf3")
        .arg("-c")
        .arg(hostname)
        .arg("-J")
        .output().unwrap();
    let out = String::from_utf8(output.stdout).unwrap();
    println!("{}", out);
}