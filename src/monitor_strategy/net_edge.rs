use std::process::{Command, Stdio};
use serde_json::Value;
use tokio::runtime::Runtime;
use crate::AUTHENTICATION;
use crate::model::{NetInfo, Target};
use super::MonitorStrategy;

pub struct NetEdgeStrategy;

impl MonitorStrategy for NetEdgeStrategy {
    fn exec(&self, rt: &Runtime, targets: &Vec<Target>) -> Value {
        let results = rt.block_on(action(targets));
        let mut response = Vec::new();
        for result in results {
            if let Ok(infos) = result {
                let mut infos = infos.to_vec();
                response.append(&mut infos);
            } else {
                println!("err: {}", result.err().unwrap());
            }
        }
        serde_json::to_value(response).unwrap()
    }
}

async fn action(targets: &Vec<Target>) -> Vec<Result<[NetInfo; 2], &str>> {
    let mut handles = Vec::with_capacity(targets.len());
    for target in targets.iter() {
        println!("{}:", target.hostname);
        handles.push(tokio::spawn(get_info(target.hostname.clone())));
    }
    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}

const BANDWIDTH_MEASURE_FAILED: &str = "网络参数测量失败";
const DELAY_MEASURE_FAILED: &str = "网络延迟测量失败";
const PARSE_FAILED: &str = "信息解析失败";
// fn get_info(hostname: String) -> Result<NetInfo, &'static str> {
async fn get_info(hostname: String) -> Result<[NetInfo;2], &'static str> {
    let test_bandwidth = || {
        let output = Command::new("iperf3")
            .arg("-c")
            .arg(&hostname)
            .arg("-J")
            .arg("-t")
            .arg("3")
            .output().map_err(|_| BANDWIDTH_MEASURE_FAILED)?;
        let out = String::from_utf8(output.stdout).map_err(|_|PARSE_FAILED)?;
        let out: Value = serde_json::from_str(&out).map_err(|_|PARSE_FAILED)?;
        let out = out.as_object().ok_or(PARSE_FAILED)?
            .get("end").ok_or(PARSE_FAILED)?
            .as_object().ok_or(PARSE_FAILED)?
            ;
        let sender_bandwidth = out.get("sum_sent").ok_or(PARSE_FAILED)?
            .as_object().ok_or(PARSE_FAILED)?.get("bits_per_second").ok_or(PARSE_FAILED)?
            .as_f64().ok_or(PARSE_FAILED)?;
        let receiver_bandwidth = out.get("sum_received").ok_or(PARSE_FAILED)?
            .as_object().ok_or(PARSE_FAILED)?.get("bits_per_second").ok_or(PARSE_FAILED)?
            .as_f64().ok_or(PARSE_FAILED)?;
        Ok::<(f64, f64), &'static str>((sender_bandwidth, receiver_bandwidth))
    };
    let test_delay = || {
        let ping = Command::new("ping")
            .args([&hostname, "-W", "5", "-c", "2"])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .stdout.unwrap();
        let awk = Command::new("awk")
            .args(["-F/", "END{print $5}"])
            .stdin(Stdio::from(ping))
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            ;
        let out = awk.wait_with_output().unwrap();
        let out = String::from_utf8(out.stdout).map_err(|_|DELAY_MEASURE_FAILED)?;
        let delay= out.trim().parse::<f64>().map_err(|_|DELAY_MEASURE_FAILED)?;
        Ok::<f64, &'static str>(delay)
    };
    let (sender_bandwidth, receiver_bandwidth) = test_bandwidth()?;
    let delay = test_delay()? / 2.0;
    Ok([
        NetInfo{
            origin_hostname: AUTHENTICATION.clone(),
            target_hostname: hostname.clone(),
            bandwidth: sender_bandwidth,
            delay
        },
        NetInfo{
            origin_hostname: hostname.clone(),
            target_hostname: AUTHENTICATION.clone(),
            bandwidth: receiver_bandwidth,
            delay
        }
    ])
}