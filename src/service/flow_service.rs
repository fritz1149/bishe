use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use log::debug;
use serde_json::Value;
use crate::config::sqlite_config::SQLITE;
use crate::model::{FlowDef, Instance};

pub struct FlowService {
    pub state: Option<Vec<FlowDef>>
}

const FORMAT_ERROR: &str = "格式错误";
const FILE_ERROR: &str = "文件交互错误";
const SAVE_ERROR: &str = "存储错误";
const DATABASE_ERROR: &str = "数据库交互错误";
const FLOW_ERROR: &str = "计算图交互错误";
impl FlowService {
    pub fn new() -> Self { Self{state: None} }
    pub async fn parse(mut self, payload: Value) -> Result<Self, &'static str> {
        debug!("parse");
        let data: Vec<FlowDef> = serde_json::from_value(payload).map_err(|e| {
            debug!("{:?}", e);
            FORMAT_ERROR
        })?;
        self.state = Some(data);
        debug!("flow def: {}", serde_json::to_string(&self.state).unwrap());
        Ok(self)
    }
    pub async fn save(self) -> Result<Self, &'static str> {
        let mut f = File::create("sqlite/flow.json").map_err(|_|FILE_ERROR)?;
        let data = self.state.as_ref().ok_or(SAVE_ERROR)?;
        let data = serde_json::to_vec(data).map_err(|_|SAVE_ERROR)?;
        f.write(&*data).map_err(|_|SAVE_ERROR)?;
        Ok(self)
    }
    pub async fn load(mut self) -> Result<Self, &'static str> {
        let mut f = File::open("sqlite/flow.json").map_err(|_|FILE_ERROR)?;
        let mut data = String::new();
        f.read_to_string(&mut data).map_err(|_|FILE_ERROR)?;
        let data: Vec<FlowDef> = serde_json::from_str(&data).map_err(|_|FORMAT_ERROR)?;
        self.state = Some(data);
        Ok(self)
    }
    pub async fn is_sink(&self, auth: &String) -> Result<bool, &'static str> {
        let auth: Vec<&str> = auth.rsplitn(2, "-").collect();
        let op_name = auth[0].to_string();
        let instance_uuid = auth[1];

        let mut rb = SQLITE.lock().await;
        let mut res = Instance::select_by_column(&mut *rb, "id", instance_uuid).await
            .map_err(|_|DATABASE_ERROR)?;
        let flow_id = res.get(0).ok_or(DATABASE_ERROR)?.flow_id;
        let flow = self.state.as_ref().ok_or(FLOW_ERROR)?
            .get(flow_id as usize).ok_or(FLOW_ERROR)?;

        let mut ret = None;
        for op in flow.operators.iter() {
            if op.name == op_name {
                match op.operator_type.as_ref().unwrap().as_str() {
                    "sink" => ret = Some(true),
                    "source"|"operator"  => ret = Some(false),
                    _ => return Err(FLOW_ERROR)
                }
                break;
            }
        }
        match ret {
            None => Err(FLOW_ERROR),
            Some(x) => Ok(x)
        }
    }
}