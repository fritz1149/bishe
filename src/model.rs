use serde::{Serialize, Deserialize};

// 测量目标，本pod开启iperf client，向目标pod的server发起测量请求
#[derive(Debug, Serialize, Deserialize)]
pub struct Host {
    pub id: String,
    pub name: String,
    pub domain_id: String,
}

#[derive(Debug, Deserialize)]
pub struct MonitorConfig {
    pub targets: Vec<Host>,
    pub self_host: Host,
    pub cross_domain: bool,
}

impl MonitorConfig {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            self_host: Host {
                id: "".to_string(),
                name: "".to_string(),
                domain_id: "".to_string(),
            },
            cross_domain: false,
        }
    }
}

// 网络性能相关参数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetEdgeInfo {
    pub source: String,
    pub target: String,
    pub bandwidth: f64,
    pub delay: f64
}