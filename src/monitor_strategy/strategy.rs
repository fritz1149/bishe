use serde_json::Value;
use tokio::runtime::Runtime;
use crate::model::{Targets};
use crate::monitor_strategy::flow_edge::FlowEdgeStrategy;
use crate::monitor_strategy::net_edge::NetEdgeStrategy;

pub trait MonitorStrategy {
    fn exec(&self, rt: &Runtime, targets: &Targets) -> Value;
}

pub fn parse_strategy(monitor_type: &String) -> Box<dyn MonitorStrategy> {
    match monitor_type.as_str() {
        "NetEdge" => Box::new(NetEdgeStrategy{}),
        "FlowEdge" => Box::new(FlowEdgeStrategy{}),
        _ => panic!("解析监控类型失败")
    }
}