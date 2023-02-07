use std::sync::{Arc, Mutex};
use axum::response::IntoResponse;
use axum::{Extension, Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use fast_log::print;
use log::{debug, Level, log};
use serde_json::{Map, Value};
use crate::model::{ComputeNode, EdgeDomain, EdgeDomainGroup};
use crate::service::topo_service::TopoService;

pub(crate) fn api() -> Router {
    Router::new()
        .route("/", post(set_topo))
}

async fn set_topo(Json(payload): Json<Value>) -> impl IntoResponse {
    let action = async { TopoService::new()
        .parse(payload).await?
        .clear().await?
        .save().await
    };
    match action.await {
        Ok(_) => (StatusCode::OK, "设置成功"),
        Err(text) => (StatusCode::BAD_REQUEST, text)
    }
}