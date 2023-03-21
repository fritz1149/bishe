use axum::{Extension, Json, Router};
use axum::response::IntoResponse;
use serde_json::{Map, Value};
use axum::http::StatusCode;
use axum::routing::post;
use crate::model::{FlowDef, OperatorDef, StreamDef, ParamDef};
use crate::service::flow_service::FlowService;

pub(crate) fn api() -> Router {
    Router::new()
        .route("/", post(set_flow))
}

async fn set_flow(Json(payload): Json<Value>) -> impl IntoResponse {
    let action = async { FlowService::new()
        .parse(payload).await?
        .save().await
    };
    match action.await {
        Ok(_) => (StatusCode::OK, "设置成功"),
        Err(text) => (StatusCode::BAD_REQUEST, text)
    }
}