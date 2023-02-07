use std::collections::HashMap;
use std::future::Future;
use std::process::Output;
use log::debug;
use std::sync::mpsc::Sender;
use std::time::Duration;
use axum::body::HttpBody;
use lazy_static::lazy_static;
use tokio::runtime::{Handle, Runtime};
use tokio::time::sleep;
use crate::config::profile_config::CONFIG;

const REQWEST_ERROR: &str = "网络请求失败";

pub(super) fn get_topo(rt: &Runtime) -> Result<String, &'static str> {
    debug!("初始阶段任务：请求网络拓扑");
    let client = reqwest::Client::builder()
        .no_proxy()
        .build().unwrap();
    let url = format!("http://{}/edge_domain_group", &CONFIG.public_cloud.ip_port);
    let mut params = HashMap::new();
    params.insert("id", &CONFIG.public_cloud.edge_domain_group_id);
    let request = async {
        client.get(url)
            .header("Authorization", "")
            .query(&params)
            .timeout(Duration::from_secs(3))
            .send()
            .await.map_err(|_|REQWEST_ERROR)?
            .text()
            .await.map_err(|_|REQWEST_ERROR)
        // sleep(Duration::from_secs(1)).await;
        // Ok(String::from("跳过"))
    };
    rt.block_on(request)
}