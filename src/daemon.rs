use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use serde_json::{json, Value};
use tokio::sync::oneshot::Sender;
use crate::config::profile_config::CONFIG;
use crate::DAEMON_STATE;
use crate::model::{DaemonState, NetInfo};
use crate::HOSTNAME;

pub fn daemon_main(should_write: Sender<Value>) {
    let daemon = move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build().unwrap();
        loop {
            thread::sleep(Duration::from_secs(CONFIG.edge.interval as u64));
            println!("守护进程⭐");
            // 应当调用一次iperf程序
            let daemon_state = DAEMON_STATE.lock().unwrap();
            rt.block_on(async {
                let mut handles = Vec::with_capacity(daemon_state.targets.len());
                for target in daemon_state.targets.iter() {
                    println!("{}:", target.hostname);
                    handles.push(tokio::spawn(get_info(target.hostname.clone())));
                }
                let mut results = Vec::with_capacity(handles.len());
                for handle in handles {
                    results.push(handle.await.unwrap());
                }
                println!("{}", json!({"net_info": results}).to_string());
            });
        }
    };
    thread::spawn(daemon);
}

const NET_MEASURE_FAILED: &str = "网络参数测量失败";
const PARSE_FAILED: &str = "信息解析失败";
// fn get_info(hostname: String) -> Result<NetInfo, &'static str> {
async fn get_info(hostname: String) -> Result<[NetInfo;2], &'static str> {
    let output = Command::new("iperf3")
        .arg("-c")
        .arg(&hostname)
        .arg("-J")
        .output().map_err(|_|NET_MEASURE_FAILED)?;
    let out = String::from_utf8(output.stdout).map_err(|_|PARSE_FAILED)?;
    let out: Value = serde_json::from_str(&out).map_err(|_|PARSE_FAILED)?;
    let out = out.as_object().ok_or(PARSE_FAILED)?;
    let sender_bandwidth = out.get("sum_sent").ok_or(PARSE_FAILED)?
        .as_object().ok_or(PARSE_FAILED)?.get("bits_per_second").ok_or(PARSE_FAILED)?
        .as_f64().ok_or(PARSE_FAILED)?;
    let receiver_bandwidth = out.get("sum_received").ok_or(PARSE_FAILED)?
        .as_object().ok_or(PARSE_FAILED)?.get("bits_per_second").ok_or(PARSE_FAILED)?
        .as_f64().ok_or(PARSE_FAILED)?;
    Ok([
        NetInfo{
            origin_hostname: HOSTNAME.clone(),
            target_hostname: hostname.clone(),
            bandwidth: sender_bandwidth
        },
        NetInfo{
            origin_hostname: hostname.clone(),
            target_hostname: HOSTNAME.clone(),
            bandwidth: receiver_bandwidth
        }
    ])
}