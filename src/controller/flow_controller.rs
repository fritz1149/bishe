use axum::{Extension, Json, Router};
use axum::response::IntoResponse;
use serde_json::{Map, Value};
use axum::http::StatusCode;
use axum::routing::post;
use log::debug;
use crate::daemon::main::Signal;
use crate::model::{FlowDef, OperatorDef, StreamDef, ParamDef};
use crate::service::flow_service::FlowService;
use crate::TELL_DAEMON;

pub(crate) fn api() -> Router {
    Router::new()
        .route("/", post(set_flow))
}

const DAEMON_ERROR: &str = "守护线程处理信号错误";
async fn set_flow(Json(payload): Json<Value>) -> impl IntoResponse {
    let async_action = async { FlowService::new()
        .parse(payload).await?
        .save().await
    };
    let action = async {
        async_action.await?;
        let tell_daemon = &*TELL_DAEMON.lock().unwrap();
        tell_daemon.send(Signal::Redeploy).map_err(|e| {
            debug!("{}", e);
            DAEMON_ERROR
        })
    };
    match action.await {
        Ok(_) => (StatusCode::OK, "设置成功"),
        Err(text) => (StatusCode::BAD_REQUEST, text)
    }
}