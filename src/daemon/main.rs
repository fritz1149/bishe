use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;
use std::sync::mpsc::{RecvError, RecvTimeoutError, Sender};
use std::thread;
use std::thread::ThreadId;
use std::time::Duration;
use lazy_static::lazy_static;
use log::debug;
use serde_json::Value;
use tokio::runtime::Handle;
use super::task_chain::*;
use std::sync::Mutex;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum Stage {
    Init,
    Deploy,
    Run,
    Rest,
    Stop
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub enum Signal {
    Stop,
    Start,
    Redeploy,
    Rest,
    Quit,
    NoSignal
}

lazy_static!{
    pub static ref STAGE: Mutex<Stage> = Mutex::new(Stage::Init);
}

pub fn set_stage(stage: Stage) {
    let mut stage_ = STAGE.lock().unwrap();
    *stage_ = stage;
}

pub fn get_stage() -> Stage {
    let stage = STAGE.lock().unwrap();
    stage.clone()
}

const MAX_FAIL_TIME: u32 = 10;
pub fn daemon_main() -> Sender<Signal> {
    let (send, recv) = mpsc::channel();
    let daemon = move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build().unwrap();
        let task_map = task_map();
        let mut task_chain = task_map.get(&Stage::Init).unwrap();
        let mut task_iter = task_chain.iter();
        let mut failed_task: Option<&Task> = None;
        let mut failed_time = 0;
        let mut no_more_task = false;
        let mut state: HashMap<&str, Value> = HashMap::new();
        loop {
            if let Some(task) = failed_task.or(task_iter.next()) {
                match task(&rt, &mut state) {
                    Err(text) => {
                        failed_task = Some(task);
                        failed_time += 1;
                        debug!("{}", text);
                        if failed_time > MAX_FAIL_TIME {
                            panic!("已失败{}次，超过最大重试次数，守护进程退出", failed_time);
                        }
                        debug!("即将睡眠{}秒后重试", 10 * failed_time);
                        thread::sleep(Duration::from_secs(10 * failed_time as u64));
                    }
                    Ok(text) => {
                        failed_time = 0;
                        failed_task = None;
                    }
                }
            }
            else if let Some(stage) = task_chain.next_stage() {
                        debug!("状态切换到：{:?}", stage);
                        task_chain = task_map.get(stage).unwrap();
                        task_iter = task_chain.iter();
                        no_more_task = false;
                        set_stage(stage.clone());
                    }
            else {
                no_more_task = true;
                debug!("已经没有更多任务");
            }
            debug!("准备接收信号");
            let signal;
            if no_more_task {
                signal = recv.recv().unwrap_or(Signal::Quit);
                debug!("⭐收到信号：{:?}", signal);
            } else {
                signal = match recv.recv_timeout(Duration::from_secs(3)) {
                    Ok(x) => x,
                    Err(RecvTimeoutError::Timeout) => Signal::NoSignal,
                    Err(RecvTimeoutError::Disconnected) => Signal::Quit
                }
            }
            debug!("收到信号：{:?}", signal);
            match signal {
                Signal::Quit => {
                    debug!("守护线程意外退出");
                    panic!();
                }
                Signal::NoSignal => {}
                _ => {
                    if let Some(stage) = task_chain.transfer_stage(signal) {
                        debug!("状态切换到：{:?}", stage);
                        task_chain = task_map.get(stage).unwrap();
                        task_iter = task_chain.iter();
                        no_more_task = false;
                        failed_task = None;
                        set_stage(stage.clone());
                    };
                }
            }
        }
    };
    thread::spawn(daemon);
    send
}