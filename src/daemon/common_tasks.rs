use std::collections::HashMap;
use std::future::Future;
use std::process::Output;
use log::debug;
use std::sync::mpsc::Sender;
use std::time::Duration;
use axum::body::HttpBody;
use k8s_openapi::api::apps::v1::DaemonSet;
use k8s_openapi::api::core::v1::{Node, Pod};
use kube::{Api, Client, ResourceExt};
use kube::api::{DeleteParams, ListParams, Patch, PatchParams, PostParams};
use lazy_static::lazy_static;
use serde_json::{json, Value};
use tokio::join;
use tokio::runtime::{Handle, Runtime};
use tokio::time::sleep;
use crate::config::profile_config::CONFIG;
use crate::config::sqlite_config::RB;
use crate::model::ComputeNode;
use crate::service::topo_service::TopoService;

const REQWEST_FAILED: &str = "网络请求失败";
const REQWEST_ERROR: &str = "网络请求状态异常";

pub(super) fn get_topo(rt: &Runtime, _: &mut Value) -> Result<(), &'static str> {
    // 初始化请求
    debug!("初始阶段任务：请求网络拓扑");
    let client = reqwest::Client::builder()
        .no_proxy()
        .build().unwrap();
    let url = format!("http://{}/edge_domain/all", &CONFIG.info_management_address);
    debug!("url: {}", url);

    let request = async {
        debug!("请求之前 ");
        // 请求过程
        let response = client.get(url)
            .header("Authorization", "")
            .timeout(Duration::from_secs(3))
            .send()
            .await.map_err(|e| { debug!("{:?}", e); REQWEST_FAILED })?;
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
const SELECT_NODES_ERROR: &str = "读取节点错误";
pub(super) fn deploy_traffic_monitor(rt: &Runtime, _: &mut Value) -> Result<(), &'static str> {
    let select_all = async {
        let mut rb = RB.lock().await;
        ComputeNode::select_all(&mut *rb).await
    };
    // 给节点打标签
    let label = async {
        debug!("准备给集群打标签");
        let client = Client::try_default().await.map_err(|_|K8S_CONTACT_ERROR)?;
        let node_api: Api<Node> = Api::all(client);
        let nodes = select_all.await.map_err(|e|SELECT_NODES_ERROR)?;
        for node in nodes {
            let hostname = node.ip_addr;
            let node_type = node.node_type.unwrap();
            let patch = json!({
                    "metadata": {
                        "labels": {
                            "node_type": node_type
                        }
                    }
                });
            let params: PatchParams = Default::default();
            let patch = Patch::Strategic(&patch);
            let res = node_api.patch(&hostname, &params, &patch)
                .await.unwrap();
        }
        Ok::<(),&'static str>(())
    };
    // 部署流量监测
    let deploy = async {
        debug!("准备部署流量监测");
        let client = Client::try_default().await.map_err(|_|K8S_CONTACT_ERROR)?;
        let ds_api: Api<DaemonSet> = Api::namespaced(client, "acbot-edge");

        let err = "配置文件\"monitor-netedge.yml\"读取错误，流量监测部署失败";
        let monitor_ds = std::fs::File::open("resources/monitor-netedge.yml").map_err(|_|err)?;
        let monitor_ds: DaemonSet = serde_yaml::from_reader(monitor_ds).map_err(|_|err)?;
        debug!("monitor: {:?}", monitor_ds);

        let err = "配置文件\"iperf-server.yml\"读取错误，流量监测部署失败";
        let iperf_ds = std::fs::File::open("resources/iperf-server.yml").map_err(|_|err)?;
        let iperf_ds: DaemonSet = serde_yaml::from_reader(iperf_ds).map_err(|_|err)?;
        debug!("iperf: {:?}", iperf_ds);

        let params: PostParams = Default::default();
        let res = ds_api.create(&params, &monitor_ds).await.unwrap();
        let res = ds_api.create(&params, &iperf_ds).await.unwrap();
        Ok::<(),&'static str>(())
    };
    rt.block_on(label)?;
    rt.block_on(deploy)
}

// 这就是Stop阶段的唯一任务
pub(super) fn stop(rt: &Runtime, _: &mut Value) -> Result<(), &'static str> {
    let remove_deploy = async {
        let client = Client::try_default().await.map_err(|_|K8S_CONTACT_ERROR)?;
        let ds_api: Api<DaemonSet> = Api::namespaced(client, "acbot-edge");
        let params: DeleteParams = Default::default();
        let res = ds_api.delete("monitor", &params).await.unwrap();
        let res = ds_api.delete("iperf-server", &params).await.unwrap();
        Ok::<(),&'static str>(())
    };
    rt.block_on(remove_deploy)
}