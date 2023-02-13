use std::collections::HashMap;
use std::future::Future;
use std::process::Output;
use log::debug;
use std::sync::mpsc::Sender;
use std::time::Duration;
use axum::body::HttpBody;
use k8s_openapi::api::core::v1::{Node, Pod};
use kube::{Api, Client, ResourceExt};
use kube::api::ListParams;
use lazy_static::lazy_static;
use serde_json::Value;
use tokio::join;
use tokio::runtime::{Handle, Runtime};
use tokio::time::sleep;
use crate::config::profile_config::CONFIG;
use crate::service::topo_service::TopoService;

const REQWEST_FAILED: &str = "网络请求失败";
const REQWEST_ERROR: &str = "网络请求状态异常";

pub(super) fn get_topo(rt: &Runtime) -> Result<(), &'static str> {
    // 初始化请求
    debug!("初始阶段任务：请求网络拓扑");
    let client = reqwest::Client::builder()
        .no_proxy()
        .build().unwrap();
    let url = format!("http://{}/edge_domain_group", &CONFIG.public_cloud.ip_port);
    let mut params = HashMap::new();
    params.insert("id", &CONFIG.public_cloud.edge_domain_group_id);

    let request = async {
        debug!("请求之前 ");
        // 请求过程
        let response = client.get(url)
            .header("Authorization", "")
            .query(&params)
            .timeout(Duration::from_secs(3))
            .send()
            .await.map_err(|_|REQWEST_FAILED)?;
        debug!("解析之前");
        // 解析过程
        if response.status().as_u16() != 200{
            return Err(REQWEST_ERROR);
        }
        let text = response
            .text()
            .await.map_err(|_|REQWEST_ERROR)?;
        debug!("落库之前");
        // 落库
        let text: Value = serde_json::from_str(&*text).map_err(|_|REQWEST_ERROR)?;
        debug!("请求的网络拓扑：{}", text.to_string());
        TopoService::new()
            .parse(text).await?
            .clear().await?
            .calc_topo_order().await?
            .save().await?;
        Ok(())
    };
    rt.block_on(request)
}

const K8S_CONTACT_ERROR: &str = "k8s集群交互错误";
pub(super) fn deploy_traffic_monitor(rt: &Runtime) -> Result<(), &'static str> {
    // 部署流量监测
    let deploy = async {
        debug!("准备获取集群信息");
        let client = Client::try_default().await.map_err(|_|K8S_CONTACT_ERROR)?;
        let node_api: Api<Node> = Api::all(client);
        let mut hostnames = Vec::new();
        for node in node_api.list(&ListParams::default()).await.map_err(|_|K8S_CONTACT_ERROR)? {
            // println!("found node {}", node.name_any());
            hostnames.push(node.name_any().clone());
        }
        debug!("{:?}", hostnames);
        Ok::<(),&'static str>(())
    };
    // 处理收到的流量
    // rt.block_on(deploy)
    Ok(())
}