use log::debug;
use tokio::runtime::Runtime;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict};
use crate::service::flow_service::FlowService;
use crate::service::topo_service::TopoService;

const PYTHON_ERROR: &str = "python调用异常";
pub(super) fn calc_scheduling(rt: &Runtime) -> Result<(), &'static str> {
    let flow_service = FlowService::new();
    let flow = rt.block_on(flow_service.load())?.state.unwrap();
    let flow = serde_json::to_string(&flow).unwrap();
    let topo_service = TopoService::new();
    let edge_domain_group = rt.block_on(topo_service.load())?.state.unwrap();
    let edge_domain_group = serde_json::to_string(&edge_domain_group).unwrap();
    debug!("flow: {:?}", flow);
    debug!("edge_domain_group: {:?}", edge_domain_group);
    Python::with_gil(|py| -> PyResult<()> {
        let caller = PyModule::import(py, "streamscheduling")?;
        let res: String = caller.getattr("hello")?.call0()?.extract()?;
        debug!("result from python: {}", res);
        let res: String = caller.getattr("schedule")?.call1((flow, edge_domain_group))?.extract()?;
        debug!("result of schedule: {}", res);
        Ok(())
    }).map_err(|e| {
        debug!("{}", e);
        PYTHON_ERROR
    })?;
    Ok(())
}
