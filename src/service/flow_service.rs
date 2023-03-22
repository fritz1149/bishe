use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use log::debug;
use serde_json::Value;
use crate::model::FlowDef;

pub struct FlowService {
    pub state: Option<Vec<FlowDef>>
}

const FORMAT_ERROR: &str = "格式错误";
const FILE_ERROR: &str = "文件交互错误";
const SAVE_ERROR: &str = "存储错误";
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
}