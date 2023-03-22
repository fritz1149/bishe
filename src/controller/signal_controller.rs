use std::collections::HashMap;
use axum::{Json, Router};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use log::debug;
use serde_json::Value;
use crate::daemon::main::Signal;
use crate::TELL_DAEMON;

pub fn api() -> Router {
    Router::new()
        .route("/", post(get_signal))
}

const PARSE_ERROR: &str = "解码错误";
const DAEMON_ERROR: &str = "守护线程处理信号错误";
async fn get_signal(Query(mut params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let mut parse = || {
        let signal_type = params.remove("type").ok_or(PARSE_ERROR)?;
        let signal = match signal_type.as_str() {
            "stop" => Signal::Stop,
            "start" => Signal::Start,
            "redeploy" => Signal::Redeploy,
            _ => return Err(PARSE_ERROR)
        };
        let tell_daemon = &*TELL_DAEMON.lock().unwrap();
        tell_daemon.send(signal).map_err(|e| {
            debug!("{}", e);
            DAEMON_ERROR
        })?;
        Ok(())
    };
    match parse() {
        Ok(_) => (StatusCode::OK, "接收信号成功"),
        Err(text) => (StatusCode::BAD_REQUEST, text)
    }
}