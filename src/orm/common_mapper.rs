use std::collections::HashMap;
use rbatis::executor::Executor;
use rbatis::{Error, impl_select, impled};
use rbatis::py_sql;
use rbatis::rbdc::db::ExecResult;
use crate::config::sqlite_config::SQLITE;
use crate::model::{ComputeNode, ComputeNodeEdge, EdgeDomain, FlowEdgeInfo, FlowInstance, FlowInstanceDeploy, FlowInstanceOrigin, Host, Instance, NetInfo};

rbatis::crud!(EdgeDomain {}, "edge_domains");
rbatis::crud!(ComputeNode {}, "compute_nodes");
rbatis::crud!(ComputeNodeEdge {}, "compute_node_edges");
rbatis::crud!(NetInfo {}, "net_infos");
rbatis::crud!(Instance {}, "instances");
rbatis::crud!(FlowEdgeInfo {}, "flow_edge_infos");
rbatis::crud!(FlowInstanceDeploy {}, "flow_instance_deploys");

#[py_sql(
"`delete from ${table_name} `"
)]
pub async fn delete_all(rb: &mut dyn Executor, table_name: &str) -> Result<ExecResult, Error> {
    impled!()
}

#[py_sql(
"`select id, name, edge_domain_id as domain_id from compute_nodes \
where father_hostname = #{hostname}`"
)]
pub async fn select_targets(rb: &mut dyn Executor, hostname: &str) -> Result<Vec<Host>, Error> {
    impled!()
}

#[py_sql(
"`select id, name, edge_domain_id as domain_id from compute_nodes \
where name = #{hostname}`"
)]
pub async fn select_by_hostname(rb: &mut dyn Executor, hostname: &str) -> Result<Host, Error> {
    impled!()
}

#[py_sql(
"`insert into net_infos `
trim ',':
  for idx,table in tables:
    if idx == 0:
      `(`
      trim ',':
        for k,_ in table:
          ${k},
      `) VALUES `
    (
    trim ',':
      for _,v in table:
        #{v},
    ),
`on conflict(origin_hostname, target_hostname) do update set \
bandwidth = excluded.bandwidth, \
delay = excluded.delay`"
)]
pub async fn insert_net_infos(rb: &mut dyn Executor, tables: &[NetInfo]) -> Result<ExecResult, Error> {
    impled!()
}

#[py_sql(
"`insert into flow_edge_infos `
trim ',':
  for idx,table in tables:
    if idx == 0:
      `(`
      trim ',':
        for k,_ in table:
          ${k},
      `) VALUES `
    (
    trim ',':
      for _,v in table:
        #{v},
    ),
`on conflict(delivery_rate) do update set \
delivery_rate = excluded.delivery_rate`"
)]
pub async fn insert_flow_edge_infos(rb: &mut dyn Executor, tables: &[FlowEdgeInfo]) -> Result<ExecResult, Error> {
    impled!()
}

#[py_sql(
"`select id, flow_definition as flow_def \
from flow_instances \
where status = 'running'`"
)]
pub async fn select_flow_instances_assigned(rb: &mut dyn Executor) -> Result<Vec<FlowInstanceOrigin>, Error> {
    impled!()
}

#[py_sql(
"`select distinct id from flow_instance_deploys`"
)]
pub async fn select_flow_instances_deployed(rb: &mut dyn Executor) -> Result<Vec<HashMap<String, String>>, Error> {
    impled!()
}