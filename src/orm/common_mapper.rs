use rbatis::executor::Executor;
use rbatis::{Error, impl_select, impled};
use rbatis::py_sql;
use rbatis::rbdc::db::ExecResult;
use crate::config::sqlite_config::RB;
use crate::model::{EdgeDomain, ComputeNode, ComputeNodeEdge, Target, NetInfo};

rbatis::crud!(EdgeDomain {}, "edge_domains");
rbatis::crud!(ComputeNode {}, "compute_nodes");
rbatis::crud!(ComputeNodeEdge {}, "compute_node_edges");
rbatis::crud!(NetInfo {}, "net_infos");
#[py_sql(
"`delete from ${table_name} `"
)]
pub async fn delete_all(rb: &mut dyn Executor, table_name: &str) -> Result<ExecResult, Error> {
    impled!()
}

#[py_sql(
"`select ip_addr as hostname from compute_nodes \
where father_hostname = #{hostname}`"
)]
pub async fn select_targets(rb: &mut dyn Executor, hostname: &str) -> Result<Vec<Target>, Error> {
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