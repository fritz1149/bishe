use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Host {
    pub id: String,
    pub name: String,
    pub domain_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub targets: Vec<Host>,
    pub self_host: Host,
    pub cross_domain: bool
}
