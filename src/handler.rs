use std::collections::HashMap;
use futures::StreamExt;
use log::debug;
use serde_json::Value;
use tokio::runtime::Handle;
use crate::config::sqlite_config::RB;
use crate::model::NetInfo;
use crate::orm::common_mapper;

pub fn handler_map() -> HashMap<String, fn(Value) -> Result<(), &'static str>> {
    let mut map: HashMap<String, fn(Value) -> Result<(), &'static str>> = HashMap::new();
    map.insert("NetInfo".to_string(), save_net_info);
    map
}

const FORMAT_ERROR: &str = "数据格式错误";
const DATABASE_ERROR: &str = "数据库交互错误";
pub fn save_net_info(data: Value) -> Result<(), &'static str> {
    let net_infos: Vec<NetInfo> = serde_json::from_value(data).map_err(|_| FORMAT_ERROR)?;
    if net_infos.len() == 0 {
        return Ok(());
    }
    let action = async move {
        let mut rb = RB.lock().await;
        if let Err(e) = common_mapper::insert_net_infos(&mut *rb, &net_infos).await {
            debug!("网络拓扑信息存储错误：{}", e.to_string());
        }
    };
    tokio::spawn(action);
    Ok(())
}