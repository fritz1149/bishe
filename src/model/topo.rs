use std::collections::HashMap;
use std::fmt::Formatter;
use log::debug;
use rbatis::Error;
use rbatis::executor::Executor;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Visitor;

struct BoolVisitor;
impl<'de> Visitor<'de> for BoolVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a boolean or binary")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> where E: serde::de::Error {
        Ok(v)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> where E: serde::de::Error {
        debug!("这是i64");
        if v < 0 || v > 1 {
            Err(E::custom("i64 input is not a binary"))
        } else {
            Ok(v == 1)
        }
    }
}

pub fn deserde_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
{
    deserializer.deserialize_bool(BoolVisitor)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeDomain {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "deserde_from_int")]
    pub is_cloud: bool,
    pub root_node_id: Option<String>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComputeNode {
    pub id: String,
    pub name: String,
    pub slot: i32,
    pub edge_domain_id: String,
    pub father_hostname: Option<String>,
    pub node_type: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComputeNodeEdge {
    pub compute_node_id1: String,
    pub compute_node_id2: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetInfo {
    pub origin_hostname: String,
    pub target_hostname: String,
    pub bandwidth: f64,
    pub delay: f64
}
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub fn node_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for node in self.compute_nodes.iter() {
            map.insert(node.id.clone(), node.name.clone());
        }
        map
    }
}