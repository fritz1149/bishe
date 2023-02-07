use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::mpsc::{RecvError, RecvTimeoutError, Sender};
use std::thread;
use std::thread::ThreadId;
use std::time::Duration;
use log::debug;
use tokio::runtime::Handle;
use super::task_chain::*;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum Stage {
    Init,
    Deploy,
    Run,
    Stop
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub enum Signal {
    Stop,
    Start,
    Redeploy,
    Quit,
    NoSignal
}

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
        let mut no_more_task = false;
        loop {
            if let Some(task) = failed_task.or(task_iter.next()) {
                match task(&rt) {
                    Err(text) => {
                        failed_task = Some(task);
                        debug!("{}", text);
                    }
                    Ok(text) => {
                        failed_task = None;
                        debug!("{}", &text);
                    }
                }
            }
            else if let Some(stage) = task_chain.next_stage() {
                        debug!("状态切换到：{:?}", stage);
                        task_chain = task_map.get(stage).unwrap();
                        task_iter = task_chain.iter();
                        no_more_task = false;
                    }
            else {
                no_more_task = true;
                debug!("已经没有更多任务");
            }
            debug!("准备接收信号");
            let signal;
            if no_more_task {
                signal = recv.recv().unwrap_or(Signal::Quit);
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
                    };
                }
            }
        }
    };
    thread::spawn(daemon);
    send
}