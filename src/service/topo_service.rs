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
        ).map_err(|e| {
            debug!("{}", e);
            DATABASE_ERROR
        })?;
        Ok(self)
    }
    pub async fn calc_topo_order(mut self) -> Result<Self, &'static str> {
        let mut domain_map = HashMap::new();
        let mut node_map = HashMap::new();
        let mut edge_map = HashMap::new();
        unsafe {
            let state = self.state.as_mut().unwrap();
            for domain in state.edge_domains.iter() {
                domain_map.insert(domain.id.as_ref().unwrap(), domain as *const EdgeDomain);
            }
            for node in state.compute_nodes.iter_mut() {
                let id = Rc::new(node.id.as_ref().unwrap().clone());
                edge_map.insert(id.clone(), Vec::new());
                node_map.insert(id.clone(), node as *mut ComputeNode);
                debug!("node_id: {}", id);
            }
            for edge in state.compute_node_edges.iter() {
                let id1 = edge.compute_node_id1.as_ref().unwrap();
                let id2 = edge.compute_node_id2.as_ref().unwrap();
                let node1 = *node_map.get(id1).unwrap();
                let node2 = *node_map.get(id2).unwrap();
                edge_map.get_mut(id1).unwrap().push(node2);
                edge_map.get_mut(id2).unwrap().push(node1);
            }
            for domain in state.edge_domains.iter() {
                let root = *node_map.get(domain.root_node_id.as_ref().unwrap()).unwrap();
                let domain_id = (*root).edge_domain_id.as_ref().unwrap();
                let domain = domain_map.get(domain_id).unwrap();
                if (**domain).is_cloud == Some(true) {
                    continue;
                }
                let mut q = vec![root];
                let (mut head, mut tail) = (0, 0);
                while head <= tail {
                    let u = q[head];
                    let u_id = (*u).id.as_ref().unwrap();
                    debug!("i:{}: u_id: {}, u_f: {:?}", head, u_id, (*u).father_hostname);
                    let mut son_cnt = 0;
                    for v in edge_map.remove(u_id).unwrap() {
                        let v_id = (*v).id.as_ref().unwrap();
                        let v_f = &mut (*v).father_hostname as *mut Option<String>;
                        let v_domain_id = (*v).edge_domain_id.as_ref().unwrap();
                        let v_domain = domain_map.get(v_domain_id).unwrap();
                        debug!("v_id: {}, v_f: {:?}, v_in_cloud: {:?}", v_id, *v_f, (**v_domain).is_cloud);
                        if *(**v_domain).is_cloud.as_ref().unwrap() {
                            (*u).father_hostname = Some((*v).ip_addr.as_ref().unwrap().clone());
                            if (*v).node_type == None{
                                (*v).node_type = Some("non-leaf".to_string());
                            }
                        }
                        else if (*v_f).is_none() {
                            *v_f = Some((*u).ip_addr.as_ref().unwrap().clone());
                            (*v).node_type = Some("leaf".to_string());
                            q.push(v);
                            tail = tail + 1;
                            son_cnt = son_cnt + 1;
                        }
                    }
                    if son_cnt > 0 {
                        (*u).node_type = Some("non-leaf".to_string());
                    }
                    head = head + 1;
                }
            }
            for x in node_map.keys() {
                let x = node_map.get(x).unwrap();
                if (**x).node_type == None {
                    (**x).node_type = Some("cloud".to_string());
                }
            }
        }
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
}