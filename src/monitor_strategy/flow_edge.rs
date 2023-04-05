use std::collections::HashMap;
use std::time::Duration;
use serde_json::{json, Value};
use tokio::runtime::Runtime;
use tokio_tungstenite::tungstenite::client;
use crate::model::{FlowEdgeInfo, FlowEdgeTarget, RabbitmqStats, Targets};
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
        let results = rt.block_on(action(targets_));
        let mut response = Vec::new();
        for result in results {
            if let Ok(info) = result {
                response.push(info);
            } else {
                println!("err: {}", result.err().unwrap());
            }
        }
        serde_json::to_value(response).unwrap()
    }
}
async fn action(targets: &Vec<FlowEdgeTarget>) -> Vec<Result<FlowEdgeInfo, &str>> {
    let mut handles = Vec::with_capacity(targets.len());
    for target in targets.iter() {
        handles.push(tokio::spawn(request((*target).clone())));
    }
    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}

const REQWEST_FAILED: &str = "网络请求失败";
const REQWEST_ERROR: &str = "网络请求状态异常";
const PARSE_FAILED: &str = "解码失败";
async fn request(target: FlowEdgeTarget) -> Result<FlowEdgeInfo, &'static str> {
    let mut params = HashMap::new();
    params.insert("columns", "message_stats.publish_details.rate");
    // 请求过程
    let client = reqwest::Client::builder()
        .no_proxy()
        .build().unwrap();
    let url = format!("http://{}/api/queues/%2f/{}", target.endpoint, target.queue_name);
    println!("url: {}", url);
    let response = client.get(url)
        .timeout(Duration::from_secs(3))
        .basic_auth("guest", Some("guest"))
        // .query(&params)
        .send()
        .await.map_err(|e| { println!("{:?}", e); REQWEST_FAILED })?;
    if response.status().as_u16() != 200{
        println!("请求失败： {}", response.text().await.unwrap());
        return Err(REQWEST_ERROR);
    }
    // let text = response.text().await.unwrap();
    // println!("原始结果: {}", text);
    // Err(PARSE_FAILED)
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