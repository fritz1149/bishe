use axum::extract::{Path, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::response::{IntoResponse, Response};
use axum::{headers, Router, TypedHeader};
use axum::routing::get;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt, TryFutureExt};
use log::debug;
use serde_json::{json, Value};
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use crate::config::sqlite_config::RB;
use crate::model::ComputeNodeEdge;
use crate::orm::common_mapper;

const CONNECTION_BREAK: &str = "连接中断";
const DATABASE_ERROR: &str = "数据库交互错误";

pub(crate) fn api() -> Router {
    Router::new()
        .route("/:ip_addr", get(handler))
}

async fn handler(ws: WebSocketUpgrade,
                 Path(ip_addr): Path<String>) -> impl IntoResponse {
    ws.on_upgrade(|socket|handle_socket(socket, ip_addr))
}

async fn handle_socket(mut socket: WebSocket, ip_addr: String) {
    debug!("client connected: {}", &ip_addr);
    let select_targets = ||async {
        let mut rb = RB.lock().await;
        common_mapper::select_node2(&mut *rb, &ip_addr).await
    };
    let (mut sender, mut receiver) = socket.split();
    let (async_send, async_recv) = oneshot::channel::<String>();
    match select_targets().await {
        Ok(targets) => {
            let msg = json!({
                "type": "SetTarget",
                "data": targets
            }).to_string();
            async_send.send(msg).expect("传输网络访问目标失败");
        },
        Err(_) => {
            debug!("SetTargets: {}", DATABASE_ERROR);
            return;
        }
    }
    tokio::spawn(write(sender, ip_addr.clone(), async_recv));
    tokio::spawn(read(receiver, ip_addr.clone()));
}

async fn read(mut recv: SplitStream<WebSocket>, ip_addr: String) -> Result<(), &'static str> {
    while let raw = recv.next().await {
        let msg = raw.ok_or(CONNECTION_BREAK)?.map_err(|_|CONNECTION_BREAK)?;
        let msg = msg.to_text().unwrap();
        debug!("{} say: {}", &ip_addr, msg);
    }
    Ok(())
}

const SHOULD_WRITE_CLOSE: &str = "应写通道关闭";
const WRITE_CHANNEL_CLOSE: &str = "写端连接断开";
async fn write(mut send: SplitSink<WebSocket, Message>, ip_addr: String, mut should_write: Receiver<String>) -> Result<(), &'static str> {
    while let msg = (&mut should_write).await.map_err(|_|"应写通道关闭")? {
        send.send(Message::Text(msg)).map_err(|_|WRITE_CHANNEL_CLOSE).await?;
    }
    Ok(())
}