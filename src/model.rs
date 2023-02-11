use serde::{Serialize, Deserialize};

// 测量目标，本pod开启iperf client，向目标pod的server发起测量请求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Target {
    pub hostname: String
}
// 守护线程里面用到的状态
pub struct DaemonState {
    pub targets: Vec<Target>
}
// 网络性能相关参数
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetInfo {
    pub origin_hostname: String,
    pub target_hostname: String,
    pub bandwidth: f64,
    // pub delay: u32
}