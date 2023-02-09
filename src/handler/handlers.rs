use std::sync::Mutex;
use serde_json::Value;
use crate::DAEMON_STATE;
use crate::model::{DaemonState, Target};
use crate::ws::PARSE_FAILED;

pub fn set_target(data: Value) -> Result<(), &'static str> {
    let mut data: Vec<Target> = serde_json::from_value(data).map_err(|_|PARSE_FAILED)?;
    println!("SetTarget: {:?}", data);
    let mut daemon_state = DAEMON_STATE.lock().unwrap();
    daemon_state.targets.append(&mut data);
    Ok(())
}