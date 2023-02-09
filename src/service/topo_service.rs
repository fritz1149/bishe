use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};
use axum::body::HttpBody;
use axum::Json;
use rbatis::Rbatis;
use serde_json::{Map, Value};
use tokio::try_join;
use crate::model::{ComputeNode, ComputeNodeEdge, EdgeDomain, EdgeDomainGroup};
use crate::config::sqlite_config::RB;
use crate::orm::common_mapper;

const KEY_NOT_FOUND: &str = "传入数据键缺失";
const FORMAT_ERROR: &str = "格式错误";
const DATABASE_ERROR: &str = "数据库交互错误";
const LOCK_ERROR: &str = "锁获取错误";

pub struct TopoService{
    state: Option<EdgeDomainGroup>
}


impl TopoService {
    pub fn new() -> Self {
        Self{state: None}
    }
    pub async fn parse(mut self, payload: Value) -> Result<Self, &'static str> {
        let mut data;
        match payload {
            Value::Object(payload)=> data = payload,
            _ => return Err(FORMAT_ERROR)
        }

        let edge_domains = data.remove("edgeDomains").ok_or_else(||KEY_NOT_FOUND)?;
        let edge_domains: Vec<EdgeDomain> = serde_json::from_value(edge_domains).map_err(|_|FORMAT_ERROR)?;

        let compute_nodes = data.remove("computeNodes").ok_or_else(||KEY_NOT_FOUND)?;
        let compute_nodes: Vec<ComputeNode> = serde_json::from_value(compute_nodes).map_err(|_|FORMAT_ERROR)?;

        let compute_node_edges = data.remove("computeNodeEdges").ok_or_else(||KEY_NOT_FOUND)?;
        let compute_node_edges: Vec<ComputeNodeEdge> = serde_json::from_value(compute_node_edges).map_err(|_|FORMAT_ERROR)?;

        self.state = Some(EdgeDomainGroup {
            edge_domains,
            compute_nodes,
            compute_node_edges,
        });
        Ok(self)
    }
    pub async fn clear(self) -> Result<Self, &'static str> {
        let delete_all = |table_name|async move{
            let mut rb = RB.lock().await;
            common_mapper::delete_all(&mut *rb, table_name).await
        };
        try_join!(delete_all("edge_domains"),
            delete_all("compute_nodes"),
            delete_all("compute_node_edges"),
        ).map_err(|_|DATABASE_ERROR)?;
        Ok(self)
    }
    pub async fn save(self) -> Result<Self, &'static str> {
        let edge_domains = &self.state.as_ref().unwrap().edge_domains;
        let mut rb = RB.lock().await;
        if edge_domains.len() > 0 {
            EdgeDomain::insert_batch(&mut *rb, edge_domains, 20)
                .await.map_err(|_|DATABASE_ERROR)?;
        }

        let compute_nodes = &self.state.as_ref().unwrap().compute_nodes;
        if compute_nodes.len() > 0 {
            ComputeNode::insert_batch(&mut *rb, compute_nodes, 20)
                .await.map_err(|_|DATABASE_ERROR)?;
        }

        let compute_node_edges = &self.state.as_ref().unwrap().compute_node_edges;
        if compute_node_edges.len() > 0 {
            ComputeNodeEdge::insert_batch(&mut *rb, compute_node_edges, 20)
                .await.map_err(|_|DATABASE_ERROR)?;
        }
        Ok(self)
    }
}