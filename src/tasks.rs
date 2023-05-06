use std::collections::HashMap;
use std::time::Duration;
use futures::executor::block_on;
use serde_json::Value;
use crate::{AUTHENTICATION, DAEMON_STATE};
use crate::config::profile_config::CONFIG;
use crate::model::{MonitorConfig, NetEdgeInfo};

const REQWEST_FAILED: &str = "网络请求失败";
const REQWEST_ERROR: &str = "网络请求状态异常";
const PARSE_FAILED: &str = "解码失败";
pub fn request_config() -> MonitorConfig {
    let client = reqwest::Client::builder()
        .no_proxy()
        .build().unwrap();
    let url = format!("http://{}/ws/config", &CONFIG.dispatcher.server_address);
    let mut params = HashMap::new();
    params.insert("authentication", AUTHENTICATION.as_str());
    let request = async {
        // 请求过程
        let response = client.get(url)
            .timeout(Duration::from_secs(3))
            .query(&params)
            .send()
            .await.map_err(|e| { println!("{:?}", e); REQWEST_FAILED })?;
        // 解析过程
        if response.status().as_u16() != 200{
            println!("请求失败： {}", response.text().await.unwrap());
            return Err(REQWEST_ERROR);
        }
        let monitor_config = response
            .json::<MonitorConfig>().await
            .map_err(|_|PARSE_FAILED)?;
        println!("请求的配置：{:?}", monitor_config);
        Ok(monitor_config)
    };
    block_on(request).unwrap_or(MonitorConfig::new())
}