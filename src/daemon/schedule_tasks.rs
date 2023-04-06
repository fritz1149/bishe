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
use crate::config::sqlite_config::RB;
use crate::model::Instance;
use crate::orm::common_mapper::insert_flow_edge_infos;

const PYTHON_ERROR: &str = "python调用异常";
pub(super) fn calc_scheduling(rt: &Runtime, state: &mut Value) -> Result<(), &'static str> {
    let flow_service = FlowService::new();
    let flow = rt.block_on(flow_service.load())?.state.unwrap();
    let flow = serde_json::to_string(&flow).unwrap();
    let topo_service = TopoService::new();
    let edge_domain_group = rt.block_on(topo_service.load())?.state.unwrap();
    let edge_domain_group = serde_json::to_string(&edge_domain_group).unwrap();
    debug!("flow: {:?}", flow);
    debug!("edge_domain_group: {:?}", edge_domain_group);
    let schedule_res = Python::with_gil(|py| -> PyResult<String> {
        let caller = PyModule::import(py, "streamscheduling")?;
        let res: String = caller.getattr("hello")?.call0()?.extract()?;
        debug!("result from python: {}", res);
        let res: String = caller.getattr("schedule")?.call1((flow, edge_domain_group))?.extract()?;
        debug!("result of schedule: {}", res);
        Ok(res)
    }).map_err(|e| {
        debug!("{}", e);
        PYTHON_ERROR
    })?;
    *state = serde_json::from_str(schedule_res.as_str()).unwrap();
    Ok(())
}

const REQWEST_FAILED: &str = "网络请求失败";
const REQWEST_ERROR: &str = "网络请求状态异常";
const DATABASE_ERROR: &str = "数据库交互错误";
pub(super) fn deploy_scheduling(rt: &Runtime, state: &mut Value) -> Result<(), &'static str> {
    let schedule_plans = state.as_array().unwrap();
    let flow_service = FlowService::new();
    let mut flow_def_list = rt.block_on(flow_service.load())?.state.unwrap();
    for i in 0..flow_def_list.len() {
        let flow_def = flow_def_list.get_mut(i).unwrap();
        let schedule_plan = schedule_plans.get(i).unwrap().as_object().unwrap();
        for operator_def in flow_def.operators.iter_mut() {
            let operator_id = operator_def.id.to_string();
            let host = schedule_plan.get(&operator_id).unwrap().as_str().unwrap().to_string();
            (*operator_def).node_selector = Some(json!({
               "kubernetes.io/hostname": host
            }));
            (*operator_def).communicate_by_IP = Some(true);
        }
    }
    let url = format!("http://{}/deploy", &CONFIG.flowdeploy_backend_address);
    // debug!("url: {}", url);
    let mut instances = Vec::with_capacity(flow_def_list.len());
    for (i, flow_def) in flow_def_list.into_iter().enumerate() {
        let client = reqwest::Client::builder()
            .no_proxy()
            .build().unwrap();
        // debug!("flow_def: {}", serde_json::to_string(&flow_def).unwrap());
        let request = async {
            let response = client.post(url.clone())
                .json(&flow_def)
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
            let mut res: Value = serde_json::from_str(&*text).map_err(|e| { debug!("{:?}", e); REQWEST_FAILED })?;
            debug!("deploy_scheduling_result: {}", res);
            let instance_uuid = res["instanceUUID"].take().as_str().unwrap().to_string();
            instances.push(Instance{
                id: instance_uuid,
                flow_id: i as u32
            });
            Ok(())
        };
        rt.block_on(request)?;
    }
    rt.block_on(async {
        let mut rb = RB.lock().await;
        Instance::insert_batch(&mut *rb, &instances, 20).await
            .map_err(|e| {
                debug!("流式计算实例信息存储错误: {}", e.to_string());
                DATABASE_ERROR
            })?;
        Ok(())
    })
}
