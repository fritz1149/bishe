use std::collections::HashMap;
use axum::extract::{Path, Query, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::response::{IntoResponse, Response};
use axum::{headers, Json, Router, TypedHeader};
use axum::http::StatusCode;
use axum::routing::{get, post};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt, TryFutureExt};
use log::debug;
use serde::Serialize;
use serde_json::{json, Value};
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::oneshot::Receiver;
use crate::config::sqlite_config::SQLITE;
use crate::handler::handler_map;
use crate::model::{ComputeNodeEdge, EdgeDomain, MonitorConfig};
use crate::orm::common_mapper;
use crate::service::flow_service::FlowService;

const CONNECTION_BREAK: &str = "连接中断";
const DATABASE_ERROR: &str = "数据库交互错误";

pub(crate) fn api() -> Router {
    Router::new()
        .route("/:hostname", get(handler))
        .route("/config", get(get_config))
}

const PARSE_ERROR: &str = "解码错误";
async fn get_config(Query(mut params): Query<HashMap<String, String>>) -> Response {
    let action = async {
        let hostname = params.remove("authentication").ok_or(PARSE_ERROR)?;
        let mut rb = SQLITE.lock().await;
        let targets = common_mapper::select_targets(&mut *rb, &hostname).await.map_err(|_|DATABASE_ERROR)?;
        let self_host = common_mapper::select_by_hostname(&mut *rb, &hostname).await.map_err(|_|DATABASE_ERROR)?;
        let domain = EdgeDomain::select_by_column(&mut *rb, "id", &self_host.domain_id).await.map_err(|_|DATABASE_ERROR)?
            .remove(0);
        let cross_domain = domain.is_cloud;
        debug!("monitor config {:?} {:?} {:?}", targets, self_host, cross_domain);
        Ok::<MonitorConfig, &'static str>(MonitorConfig{ targets, self_host, cross_domain })
    };
    match action.await {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(text) => (StatusCode::BAD_REQUEST, text).into_response()
    }
}

async fn handler(ws: WebSocketUpgrade,
                 Path(hostname): Path<String>) -> impl IntoResponse {
    ws.on_upgrade(|socket|handle_socket(socket, hostname))
}

async fn handle_socket(mut socket: WebSocket, hostname: String) {
    debug!("client connected: {}", &hostname);
    let (mut sender, mut receiver) = socket.split();
    let (async_send, async_recv) = mpsc::unbounded_channel();
    // let select_targets = ||async {
    //     let mut rb = SQLITE.lock().await;
    //     common_mapper::select_targets(&mut *rb, &hostname).await
    // };
    // match select_targets().await {
    //     Ok(targets) => {
    //         let msg = json!({
    //             "type": "SetTarget",
    //             "data": targets
    //         });
    //         async_send.send(msg).expect("传输网络访问目标失败");
    //     },
    //     Err(_) => {
    //         debug!("SetTargets: {}", DATABASE_ERROR);
    //         return;
    //     }
    // }
    tokio::spawn(write(sender, hostname.clone(), async_recv));
    tokio::spawn(read(receiver, hostname.clone()));
}

async fn read(mut recv: SplitStream<WebSocket>, hostname: String) -> Result<(), &'static str> {
    let mut handler_map = handler_map();
    while let raw = recv.next().await {
        let msg = raw.ok_or(CONNECTION_BREAK)?.map_err(|_|CONNECTION_BREAK)?;
        let msg = msg.to_text().unwrap();
        let mut msg: Value = serde_json::from_str(msg).unwrap();
        let msg_type: String = serde_json::from_value(msg["type"].take()).unwrap();
        let data = msg["data"].take();
        // debug!("from {}: msg_type: {}, data:{}", &hostname, msg_type, data.to_string());
        let handler = handler_map.get(&msg_type).unwrap();
        handler(data, &hostname)?;
    }
    Ok(())
}


const SHOULD_WRITE_CLOSE: &str = "应写通道关闭";
const WRITE_CHANNEL_CLOSE: &str = "写端连接断开";
async fn write(mut send: SplitSink<WebSocket, Message>, hostname: String, mut should_write: UnboundedReceiver<Value>) -> Result<(), &'static str> {
    while let msg = should_write.recv().await {
        let msg = msg.ok_or(SHOULD_WRITE_CLOSE)?.to_string();
        send.send(Message::Text(msg)).map_err(|_|WRITE_CHANNEL_CLOSE).await?;
    }
    Ok(())
}