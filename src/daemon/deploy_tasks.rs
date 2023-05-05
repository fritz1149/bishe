use std::collections::HashMap;
use std::time::Duration;
use log::debug;
use tokio::runtime::Runtime;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict};
use serde_json::{json, Map, Value};
use crate::config::profile_config::CONFIG;
use crate::service::flow_service::FlowService;
use crate::service::topo_service::TopoService;
use std::iter::zip;
use crate::config::pg_config::PG;
use crate::config::sqlite_config::SQLITE;
use crate::model::{EdgeDomainGroup, FlowInstance, FlowDefOrigin, FlowInstanceDeploy, Instance, StreamDef};
use crate::orm::common_mapper;
use crate::orm::common_mapper::{insert_flow_edge_infos};

const DATABASE_ERROR: &str = "数据库交互错误";
pub(super) fn fetch_flows(rt: &Runtime, state: &mut HashMap<&str, Value>) -> Result<(), &'static str> {
    let fetch = async {
        let mut rb = PG.lock().await;
        return common_mapper::select_flow_instances_assigned(&mut *rb).await;
    };
    let instances = rt.block_on(fetch).map_err(|e| {
        debug!("pgsql数据获取错误: {:?}", e);
        DATABASE_ERROR
    })?;
    let instances: Vec<FlowInstance> = instances.into_iter().map(|instance| {
        let mut flow_def_origin: FlowDefOrigin = serde_json::from_str(instance.flow_def.as_str()).unwrap();
        let streams = flow_def_origin.links.into_iter().map(|link| {
            let from_op = link.from_role_id;
            let from_arg = link.from_role_output_port;
            let from_stream = flow_def_origin.roles.get(from_op as usize - 1).unwrap()
                .returns.get(from_arg as usize).unwrap()
                .bind_stream.clone();
            let to_op = link.to_role_id;
            let to_arg = link.to_role_input_port;
            let to_stream = flow_def_origin.roles.get(to_op as usize - 1).unwrap()
                .params.get(to_arg as usize).unwrap()
                .bind_stream.clone();
            StreamDef {
                from_operator_id: from_op,
                from_operator_output_stream: from_stream,
                to_operator_id: to_op,
                to_operator_input_stream: to_stream
            }
        }).collect();
        for op in flow_def_origin.roles.iter_mut() {
            op.operator_type = op.operator_type.to_lowercase();
            if op.operator_type == "transform" {
                op.operator_type = String::from("operator");
            }
        }
        FlowInstance {
            instance_id: instance.id,
            version: flow_def_origin.version,
            robot_id: flow_def_origin.robot_id,
            edge_device_id: flow_def_origin.edge_device_id,
            operators: flow_def_origin.roles,
            streams,
        }
    }).collect();
    state.insert("instances", serde_json::to_value(instances).unwrap());
    Ok(())
}

const PYTHON_ERROR: &str = "python调用异常";
pub(super) fn calc_scheduling(rt: &Runtime, state: &mut HashMap<&str, Value>) -> Result<(), &'static str> {
    let mut instances: Vec<FlowInstance> = serde_json::from_value(state.get("instances").unwrap().clone()).unwrap();
    if instances.len() == 0 {
        state.insert("scheduled_instances", serde_json::to_value(instances).unwrap());
        return Ok(());
    }
    let flow = serde_json::to_string(&instances).unwrap();

    let topo_service = TopoService::new();
    let edge_domain_group = rt.block_on(topo_service.load())?.state.unwrap();
    let edg = serde_json::to_string(&edge_domain_group).unwrap();
    debug!("flow: {:?}", flow);
    debug!("edge_domain_group: {:?}", edg);
    let schedule_res = Python::with_gil(|py| -> PyResult<String> {
        let caller = PyModule::import(py, "streamscheduling")?;
        let res: String = caller.getattr("hello")?.call0()?.extract()?;
        debug!("result from python: {}", res);
        let res: String = caller.getattr("schedule")?.call1((flow, edg))?.extract()?;
        debug!("result of schedule: {}", res);
        Ok(res)
    }).map_err(|e| {
        debug!("{}", e);
        PYTHON_ERROR
    })?;

    let node_map = edge_domain_group.node_map();
    let schedule_map: Vec<HashMap<String, String>> = serde_json::from_str(&schedule_res).unwrap();
    for (i, map) in schedule_map.into_iter().enumerate() {
        let instance = instances.get_mut(i).unwrap();
        for (id, host)  in map {
            debug!("调度：{}->{}", id, node_map.get(&host).unwrap());
            let id = id.parse::<usize>().unwrap();
            instance.operators.get_mut(id-1).unwrap().host_constraint = Some(host);
        }
    }

    let mut instance_deploys = Vec::new();
    for instance in instances.iter_mut() {
        for op in instance.operators.iter_mut() {
            instance_deploys.push(FlowInstanceDeploy {
                id: instance.instance_id.clone(),
                operator_index: op.id,
                operator_id: op.operator_id.clone(),
                compute_node_id: op.host_constraint.clone().unwrap(),
            });
            let host = node_map.get(op.host_constraint.as_ref().unwrap()).unwrap();
            op.node_selector = Some(json!({
                "kubernetes.io/hostname": host
            }));
            op.communicate_by_IP = Some(true)
        }
    }
    debug!("实例: {:?}", instances);
    rt.block_on(async {
        let mut rb = PG.lock().await;
        FlowInstanceDeploy::insert_batch(&mut *rb, &instance_deploys, 20).await.unwrap();
    });
    FlowService::new()
        .set_state(instances.clone())?
        .save()?;
    state.insert("scheduled_instances", serde_json::to_value(instances).unwrap());
    Ok(())
}

const REQWEST_FAILED: &str = "网络请求失败";
const REQWEST_ERROR: &str = "网络请求状态异常";
pub(super) fn deploy_scheduling(rt: &Runtime, state: &mut HashMap<&str, Value>) -> Result<(), &'static str> {
    let instances: Vec<FlowInstance> = serde_json::from_value(state.get("scheduled_instances").unwrap().clone()).unwrap();
    if instances.len() == 0 {
        return Ok(())
    }
    let url = format!("http://{}/deploy", &CONFIG.flowdeploy_backend_address);
    let client = reqwest::Client::builder()
        .no_proxy()
        .build().unwrap();
    for (i, instance) in instances.into_iter().enumerate() {
        // debug!("flow_def: {}", serde_json::to_string(&flow_def).unwrap());
        let request = async {
            let response = client.post(url.clone())
                .json(&instance)
                .timeout(Duration::from_secs(20))
                .send()
                .await.map_err(|e| { debug!("{:?}", e); REQWEST_FAILED })?;
            // 解析过程
            if response.status().as_u16() != 200 {
                debug!("状态码: {}", response.status().as_u16());
                debug!("文本信息： {}", response.text().await.unwrap());
                return Err(REQWEST_ERROR);
            }
            let text = response
                .text()
                .await.map_err(|e| { debug!("{:?}", e); REQWEST_FAILED })?;
            debug!("deploy_scheduling_result: {}", text);
            Ok(())
        };
        rt.block_on(request)?;
    }
    Ok(())
}
