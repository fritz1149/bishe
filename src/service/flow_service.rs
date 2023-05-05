use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use log::debug;
use serde_json::Value;
use crate::config::sqlite_config::SQLITE;
use crate::model::{FlowInstance, Instance};

pub struct FlowService {
    pub state: Option<Vec<FlowInstance>>
}

const FORMAT_ERROR: &str = "格式错误";
const FILE_ERROR: &str = "文件交互错误";
const SAVE_ERROR: &str = "存储错误";
const DATABASE_ERROR: &str = "数据库交互错误";
const FLOW_ERROR: &str = "计算图交互错误";
impl FlowService {
    pub fn new() -> Self { Self{state: None} }
    pub fn set_state(mut self, instances: Vec<FlowInstance>) -> Result<Self, &'static str> {
        self.state = Some(instances);
        Ok(self)
    }
    pub fn save(self) -> Result<Self, &'static str> {
        let mut f = File::create("sqlite/flow.json").map_err(|_|FILE_ERROR)?;
        let data = self.state.as_ref().ok_or(SAVE_ERROR)?;
        let data = serde_json::to_vec(data).map_err(|_|SAVE_ERROR)?;
        f.write(&*data).map_err(|_|SAVE_ERROR)?;
        Ok(self)
    }
    pub fn load(mut self) -> Result<Self, &'static str> {
        let mut f = File::open("sqlite/flow.json").map_err(|_|FILE_ERROR)?;
        let mut data = String::new();
        f.read_to_string(&mut data).map_err(|_|FILE_ERROR)?;
        let data: Vec<FlowInstance> = serde_json::from_str(&data).map_err(|e| {
            debug!("load flow error: {}", e);
            FORMAT_ERROR })?;
        self.state = Some(data);
        Ok(self)
    }
    pub fn is_sink(&self, auth: &String) -> Result<bool, &'static str> {
        let auth: Vec<&str> = auth.rsplitn(3, "-").collect();
        let op_id = auth[0].parse::<u32>().unwrap();
        let instance_id = auth[2];

        if let None = self.state {
            return Err(FLOW_ERROR);
        }
        let mut ins_this: Option<&FlowInstance> = None;
        for instance in self.state.as_ref().unwrap() {
            if instance.instance_id == instance_id {
                ins_this = Some(instance);
                break;
            }
        }
        if let None = ins_this {
            return Err(FLOW_ERROR);
        }

        let mut ret = None;
        for op in ins_this.unwrap().operators.iter() {
            if op.id == op_id {
                match op.operator_type.as_str() {
                    "sink" => ret = Some(true),
                    "source"|"operator" => ret = Some(false),
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