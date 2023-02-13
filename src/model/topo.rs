use rbatis::Error;
use rbatis::executor::Executor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeDomain {
    pub id: Option<String>,
    pub name: Option<String>,
    pub is_cloud: Option<bool>,
    pub root_node_id: Option<String>
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComputeNode {
    pub id: Option<String>,
    pub ip_addr: Option<String>,
    pub slot: Option<i32>,
    pub edge_domain_id: Option<String>,
    pub father_hostname: Option<String>,
    pub node_type: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComputeNodeEdge {
    pub compute_node_id1: Option<String>,
    pub compute_node_id2: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Target {
    pub hostname: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetInfo {
    pub origin_hostname: String,
    pub target_hostname: String,
    pub bandwidth: f64,
    pub delay: f64
}
pub struct EdgeDomainGroup {
    pub edge_domains: Vec<EdgeDomain>,
    pub compute_nodes: Vec<ComputeNode>,
    pub compute_node_edges: Vec<ComputeNodeEdge>
}

impl EdgeDomainGroup {
    pub fn new() -> Self {
        Self{
            edge_domains: Vec::new(),
            compute_nodes: Vec::new(),
            compute_node_edges: Vec::new()
        }
    }
}