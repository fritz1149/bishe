use serde_json::Value;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamDef {
    pub name: String,
    pub bind_stream: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperatorDef {
    pub id: u32,
    pub name: String,
    pub location_id: String,
    pub operator_name: String,
    pub operator_id: String,
    pub operator_version: String,
    pub image_name: String,
    pub input_count: u32,
    pub output_count: u32,
    pub position_x: f32,
    pub position_y: f32,
    pub params: Vec<ParamDef>,
    pub returns: Vec<ParamDef>,
    pub communicate_by_IP: Option<bool>,
    pub node_selector: Option<Value>,
    pub operator_type: Option<String>,
    pub host_constraint: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamDef {
    pub from_operator_id: u32,
    pub from_operator_output_stream: String,
    pub to_operator_id: u32,
    pub to_operator_input_stream: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlowDef {
    pub version: String,
    pub robot_id: String,
    pub edge_device_id: String,
    pub operators: Vec<OperatorDef>,
    pub streams: Vec<StreamDef>
}