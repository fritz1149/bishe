use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct NetEdgeTarget {
    pub name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlowEdgeTarget {
    pub endpoint: String,
    pub queue_name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Targets {
    NetEdgeTargets(Vec<NetEdgeTarget>),
    FlowEdgeTargets(Vec<FlowEdgeTarget>)
}

#[derive(Serialize, Deserialize)]
pub struct MonitorConfig {
    pub targets: Targets
}
