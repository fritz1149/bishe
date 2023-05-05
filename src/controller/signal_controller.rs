use std::collections::HashMap;
use axum::{Json, Router};
use axum::extract::Query;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{post, get};
use log::debug;
use serde_json::Value;
use crate::daemon::main::{get_stage, Signal, Stage};
use crate::TELL_DAEMON;

pub fn api() -> Router {
    Router::new()
        .route("/", post(set_signal))
        .route("/current_stage", get(get_current_stage))
}

const PARSE_ERROR: &str = "解码错误";
const DAEMON_ERROR: &str = "守护线程处理信号错误";
async fn set_signal(Query(mut params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let mut parse = || {
        let signal_type = params.remove("type").ok_or(PARSE_ERROR)?;
        let signal = match signal_type.as_str() {
            "stop" => Signal::Stop,
            "start" => Signal::Start,
            "redeploy" => Signal::Redeploy,
            "rest" => Signal::Rest,
            _ => return Err(PARSE_ERROR)
        };
        let tell_daemon = &*TELL_DAEMON.lock().unwrap();
        tell_daemon.send(signal).map_err(|e| {
            debug!("{}", e);
            DAEMON_ERROR
        })?;
        Ok(())
    };
    let mut headers = HeaderMap::new();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    match parse() {
        Ok(_) => (StatusCode::OK, headers, "接收信号成功"),
        Err(text) => (StatusCode::BAD_REQUEST, headers, text)
    }
}

async fn get_current_stage() -> impl IntoResponse {
    let stage = format!("{:?}", get_stage());
    let mut headers = HeaderMap::new();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    (StatusCode::OK, headers, stage)
}