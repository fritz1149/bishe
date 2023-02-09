use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::rc::Rc;
use std::slice::Iter;
use tokio::runtime::{Handle, Runtime};
use super::main::{Signal, Stage};
use super::tasks::*;

pub type Task = fn(&Runtime) -> Result<(), &'static str>;

pub struct TaskChain {
    task_vec: Vec<Task>,
    next_chain: Option<Stage>,
    signal_transfer: HashMap<Signal, Stage>
}

impl TaskChain {
    fn new() -> Self {
        Self {
            task_vec: Vec::new(),
            next_chain: None,
            signal_transfer: HashMap::new()
        }
    }
    fn set_task(mut self, task: Task) -> Self {
        self.task_vec.push(task);
        self
    }
    fn set_next(mut self, next: Stage) -> Self {
        self.next_chain = Some(next);
        self
    }
    fn set_transfer(mut self, signal: Signal, next: Stage) -> Self {
        self.signal_transfer.insert(signal, next);
        self
    }
    pub fn iter(&self) -> Iter<Task> {
        self.task_vec.iter()
    }
    pub fn next_stage(&self) -> &Option<Stage> {
        &self.next_chain
    }
    pub fn transfer_stage(&self, signal: Signal) -> Option<&Stage> {
        self.signal_transfer.get(&signal)
    }
}

pub fn task_map() -> HashMap<Stage, TaskChain> {
    let init = TaskChain::new()
        .set_task(get_topo)
        .set_task(deploy_traffic_monitor)
        .set_next(Stage::Deploy)
        .set_transfer(Signal::Stop, Stage::Stop);
    let deploy = TaskChain::new()
        .set_next(Stage::Run)
        .set_transfer(Signal::Stop, Stage::Stop);
    let run = TaskChain::new()
        .set_transfer(Signal::Redeploy, Stage::Deploy)
        .set_transfer(Signal::Stop, Stage::Stop);
    let stop = TaskChain::new()
        .set_transfer(Signal::Start, Stage::Init);
    let mut map = HashMap::new();
    map.insert(Stage::Init, init);
    map.insert(Stage::Deploy, deploy);
    map.insert(Stage::Run, run);
    map.insert(Stage::Stop, stop);
    map
}