use std::ffi::OsStr;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use metrics::{describe_gauge, gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use regex::Regex;
use serde_json::{json, Value};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot::Sender;
use crate::config::profile_config::CONFIG;
use crate::DAEMON_STATE;
use crate::model::{MonitorConfig, NetEdgeInfo};
use crate::AUTHENTICATION;
use crate::fetch::exec;

pub fn daemon_main() -> JoinHandle<()> {
    let daemon = move || {
        let metrics_url = format!("http://{}/metrics/job/net_edge_info/instance/{}",
                                  &CONFIG.prometheus.pushgateway_address, DAEMON_STATE.get().unwrap().self_host.id.clone());
        let builder = PrometheusBuilder::new()
            .with_push_gateway(metrics_url, Duration::from_secs(CONFIG.monitor.interval as u64))
            .expect("push gateway built error")
            .install()
            .expect("failed to install prometheus recorder");
        describe_gauge!("net_edge_bandwidth", "bandwidth of computeNodeEdge");
        describe_gauge!("net_edge_delay", "delay of computeNodeEdge");

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build().unwrap();
        loop {
            thread::sleep(Duration::from_secs(CONFIG.monitor.interval as u64));
            println!("守护进程⭐");
            if DAEMON_STATE.get().unwrap().targets.is_empty() {
                continue;
            }
            let response = exec(&rt);
            for info in response {
                println!("{:?}", info);
                gauge!("net_edge_bandwidth", info.bandwidth,
                "source" => info.source.clone(), "target" => info.target.clone(), "domainId" => DAEMON_STATE.get().unwrap().self_host.domain_id.clone());
                gauge!("net_edge_delay", info.delay,
                "source" => info.source.clone(), "target" => info.target.clone(), "domainId" => DAEMON_STATE.get().unwrap().self_host.domain_id.clone());
            }
        }
    };
    thread::spawn(daemon)
}