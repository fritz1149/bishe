use rbatis::executor::Executor;
use rbatis::{Error, impl_select, impled};
use rbatis::py_sql;
use rbatis::rbdc::db::ExecResult;
use crate::config::sqlite_config::RB;
use crate::model::{EdgeDomain, ComputeNode, ComputeNodeEdge, Target};

rbatis::crud!(EdgeDomain {}, "edge_domains");
rbatis::crud!(ComputeNode {}, "compute_nodes");
rbatis::crud!(ComputeNodeEdge {}, "compute_node_edges");

#[py_sql(
"`delete from ${table_name} `"
)]
pub async fn delete_all(rb: &mut dyn Executor, table_name: &str) -> Result<ExecResult, Error> {
    impled!()
}

#[py_sql(
"`select nodes2.ip_addr as hostname from \
compute_node_edges as edges join compute_nodes as nodes1 on edges.compute_node_id1 = nodes1.id \
join compute_nodes as nodes2 on edges.compute_node_id2 = nodes2.id \
where nodes1.ip_addr = #{node1}`"
)]
pub async fn select_node2(rb: &mut dyn Executor, node1: &str) -> Result<Vec<Target>, Error> {
    impled!()
}
