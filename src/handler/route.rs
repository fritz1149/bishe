use std::collections::HashMap;
use std::future::Future;
use std::sync::Mutex;
use serde_json::Value;
use crate::model::DaemonState;
use super::handlers::set_target;

pub fn handler_map() -> HashMap<String, fn(Value) -> Result<(), &'static str>> {
    let mut map: HashMap<String, fn(Value) -> Result<(), &'static str>> = HashMap::new();
    map.insert("SetTarget".to_string(), set_target);
    map
}