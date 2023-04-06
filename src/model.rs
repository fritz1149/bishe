use serde::{Serialize, Deserialize};

// 测量目标，本pod开启iperf client，向目标pod的server发起测量请求
#[derive(Debug, Serialize, Deserialize)]
pub struct NetEdgeTarget {
    pub name: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlowEdgeTarget {
    pub endpoint: String,
    pub queue_name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Targets {
    NetEdgeTargets(Vec<NetEdgeTarget>),
    FlowEdgeTargets(Vec<FlowEdgeTarget>),
    None
}

#[derive(Debug, Deserialize)]
pub struct MonitorConfig {
    pub targets: Targets
}

// 守护线程里面用到的状态
pub struct DaemonState {
    pub targets: Targets
}

// 网络性能相关参数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetEdgeInfo {
    pub origin_hostname: String,
    pub target_hostname: String,
    pub bandwidth: f64,
    pub delay: f64
}
// 流式计算边相关参数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlowEdgeInfo {
    pub queue_name: String,
    pub delivery_rate: f64,
}
// 算子相关参数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpInfo {
    pub op_exec_time: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlowInfo {
    pub flow_edge_infos: Vec<FlowEdgeInfo>,
    pub op_info: OpInfo
}

// rabbitmq请求结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RateDetails {
    pub rate: f64
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublishDetails {
    pub publish_details: RateDetails
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RabbitmqStats {
    pub message_stats: PublishDetails
}