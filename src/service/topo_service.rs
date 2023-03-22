use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};
use axum::body::HttpBody;
use axum::Json;
use log::debug;
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
    pub state: Option<EdgeDomainGroup>
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
        ).map_err(|e| {
            debug!("{}", e);
            DATABASE_ERROR
        })?;
        Ok(self)
    }
    pub async fn calc_topo_order(mut self) -> Result<Self, &'static str> {
        let mut domain_map = HashMap::new();
        let mut node_map = HashMap::new();
        let mut indegree_map = HashMap::new();
        let mut edge_map = HashMap::new();
        unsafe {
            let state = self.state.as_mut().unwrap();
            for domain in state.edge_domains.iter_mut() {
                let domain = domain as *mut EdgeDomain;
                let id = &(*domain).id;
                domain_map.insert(id.clone(), domain);
            }
            for node in state.compute_nodes.iter_mut() {
                let node = node as *mut ComputeNode;
                let id = &(*node).id;
                edge_map.insert(id.clone(), Vec::new());
                node_map.insert(id.clone(), node);
                indegree_map.insert(id.clone(), 0);
                debug!("node_id: {}", id);
            }
            for edge in state.compute_node_edges.iter() {
                let id1 = &edge.compute_node_id1;
                let id2 = &edge.compute_node_id2;
                let node2 = *node_map.get(id2).unwrap();
                edge_map.get_mut(id1).unwrap().push(node2);
                let indegree = indegree_map.get_mut(id2).unwrap();
                *indegree = *indegree + 1;
            }
            for node in state.compute_nodes.iter() {
                let node = node as *const ComputeNode;
                let id = &(*node).id;
                if *indegree_map.get(id).unwrap() == 0 {
                    let domain_id = &(*node).edge_domain_id;
                    let domain = *domain_map.get(domain_id).unwrap();
                    (*domain).root_node_id = Some(id.clone());
                }
            }
            for domain in state.edge_domains.iter() {
                let domain = domain as *const EdgeDomain;
                if (*domain).is_cloud {
                    continue;
                }
                let root = (*domain).root_node_id.as_ref().unwrap();
                let mut q = vec![root];
                let (mut head, mut tail) = (0, 0);
                while head <= tail {
                    let uid = q[head];
                    let u = *node_map.get(uid).unwrap();
                    let edges = edge_map.remove(uid).unwrap();
                    if edges.len() == 0 {
                        (*u).node_type = Some("leaf".to_string())
                    } else {
                        (*u).node_type = Some("non-leaf".to_string())
                    }
                    for v in edges {
                        (*v).father_hostname = Some(uid.clone());
                        q.push(&(*v).id);
                        tail = tail + 1;
                    }
                    head = head + 1;
                }
            }
            for node in state.compute_nodes.iter_mut() {
                if node.node_type == None {
                    node.node_type = Some("cloud".to_string())
                }
            }
        }

        debug!("{:?}", self.state);
        Ok(self)
    }
    pub async fn save(self) -> Result<Self, &'static str> {
        let edge_domains = &self.state.as_ref().unwrap().edge_domains;
        let mut rb = RB.lock().await;
        if edge_domains.len() > 0 {
            EdgeDomain::insert_batch(&mut *rb, edge_domains, 20)
                .await.map_err(|e| {
                        debug!("{}", e);
                        DATABASE_ERROR
                    })?;
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
    pub async fn load(mut self) -> Result<Self, &'static str> {
        let mut rb = RB.lock().await;
        let edge_domains = EdgeDomain::select_all(&mut *rb).await.map_err(|e| {
            debug!("{}", e);
            DATABASE_ERROR
        })?;
        let compute_nodes = ComputeNode::select_all(&mut *rb).await.map_err(|e| {
            debug!("{}", e);
            DATABASE_ERROR
        })?;
        let compute_node_edges = ComputeNodeEdge::select_all(&mut *rb).await.map_err(|e| {
            debug!("{}", e);
            DATABASE_ERROR
        })?;
        self.state = Some(EdgeDomainGroup{
            edge_domains,
            compute_nodes,
            compute_node_edges
        });
        Ok(self)
    }
}