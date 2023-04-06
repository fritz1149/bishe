use std::collections::HashMap;
use std::fmt::format;
use std::time::Duration;
use serde_json::{json, Value};
use tokio::runtime::Runtime;
use tokio_tungstenite::tungstenite::client;
use crate::model::{FlowEdgeInfo, FlowEdgeTarget, FlowInfo, OpInfo, RabbitmqStats, Targets};
use super::MonitorStrategy;

pub struct FlowEdgeStrategy;

impl MonitorStrategy for FlowEdgeStrategy {
    fn exec(&self, rt: &Runtime, targets: &Targets) -> Value {
        let targets_;
        if let Targets::FlowEdgeTargets(x) = targets {
            targets_ = x;
        } else {
            panic!("解析目标失败");
        }
        let response = rt.block_on(action(targets_));
        serde_json::to_value(response).unwrap()
    }
}
async fn action(targets: &Vec<FlowEdgeTarget>) -> FlowInfo {
    let mut handles = Vec::with_capacity(targets.len());
    for target in targets.iter() {
        handles.push(tokio::spawn(request_flow_edge_stats((*target).clone())));
    }
    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.await.unwrap() {
            Ok(info) => results.push(info),
            Err(e) => println!("err: {}", e.to_string())
        }
    }
    let bad_op_info: Result<OpInfo, &str> = Ok(OpInfo{ op_exec_time:-1.0});
    let op_info = request_op_stats().await.or(bad_op_info).unwrap();
    FlowInfo{flow_edge_infos: results, op_info}
}

const REQWEST_FAILED: &str = "网络请求失败";
const REQWEST_ERROR: &str = "网络请求状态异常";
const PARSE_FAILED: &str = "解码失败";
async fn request_flow_edge_stats(target: FlowEdgeTarget) -> Result<FlowEdgeInfo, &'static str> {
    let mut params = HashMap::new();
    params.insert("columns", "message_stats.publish_details.rate");
    // 请求过程
    let client = reqwest::Client::builder()
        .no_proxy()
        .build().unwrap();

    let url = format!("http://{}/api/queues/%2f/{}", target.endpoint, target.queue_name);
    let response = client.get(url)
        .timeout(Duration::from_secs(3))
        .basic_auth("guest", Some("guest"))
        .query(&params)
        .send()
        .await.map_err(|e| { println!("{:?}", e); REQWEST_FAILED })?;
    if response.status().as_u16() != 200{
        println!("请求失败： {}", response.text().await.unwrap());
        return Err(REQWEST_ERROR);
    }
    let rabbitmq_stats = response
        .json::<RabbitmqStats>().await
        .map_err(|e| {
            println!("{:?}", e);
            PARSE_FAILED
        })?;
    Ok(FlowEdgeInfo{
        queue_name: target.queue_name.clone(),
        delivery_rate: rabbitmq_stats.message_stats.publish_details.rate
    })
}
async fn request_op_stats() -> Result<OpInfo, &'static str> {
    let client = reqwest::Client::builder()
        .no_proxy()
        .build().unwrap();
    let url = format!("http://localhost:8080/stats");
    let response = client.get(url)
        .timeout(Duration::from_secs(3))
        .send()
        .await.map_err(|e| { println!("{:?}", e); REQWEST_FAILED })?;
    if response.status().as_u16() != 200{
        println!("请求失败： {}", response.text().await.unwrap());
        return Err(REQWEST_ERROR);
    }
    // let text = response.text().await.unwrap();
    // println!("stats: {}", text);
    let op_info = response
        .json::<OpInfo>().await
        .map_err(|e| {
            println!("{:?}", e);
            PARSE_FAILED
        })?;
    Ok(op_info)
}