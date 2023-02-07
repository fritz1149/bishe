use rbatis::executor::Executor;
use rbatis::{Error, impled};
use rbatis::py_sql;
use rbatis::rbdc::db::ExecResult;
use crate::config::sqlite_config::RB;

#[py_sql(
"`delete from ${table_name} `"
)]
pub async fn delete_all_raw(rb: &mut dyn Executor, table_name: &str) -> Result<ExecResult, Error> {
    impled!()
}

pub async fn delete_all(table_name: &str) -> Result<ExecResult, Error> {
    let mut rb = RB.lock().await;
    delete_all_raw(&mut *rb, table_name).await
}