use serde_json::{json, Value};
use tokio::runtime::Runtime;
use crate::model::Target;
use super::MonitorStrategy;

pub struct FlowEdgeStrategy;

impl MonitorStrategy for FlowEdgeStrategy {
    fn exec(&self, rt: &Runtime, targets: &Vec<Target>) -> Value {
        json!({})
    }
}