use std::collections::HashMap;
use std::sync::Mutex;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt, TryFutureExt};
use serde_json::{Map, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use tokio::{join, try_join};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use crate::config::profile_config::CONFIG;
use crate::handler::handlers::set_target;
use crate::handler::route::handler_map;
use crate::AUTHENTICATION;
use crate::model::DaemonState;

const CONNECTION_BREAK: &str = "连接中断";
pub const PARSE_FAILED: &str = "解析失败";

pub async fn connect() -> (UnboundedSender<Value>, JoinHandle<Result<(), &'static str>>, JoinHandle<Result<(), &'static str>>) {
    let url = format!("ws://{}/ws/{}", &CONFIG.dispatcher.server_address, &AUTHENTICATION.as_str());
    println!("{}", url);
    let (ws_stream, _) = connect_async(url).await.expect("ws连接失败");
    println!("ws连接成功");
    let (ws_send, ws_recv) = ws_stream.split();
    let (async_send, async_recv) = mpsc::unbounded_channel();
    let handle_read = tokio::spawn(read(ws_recv));
    let handle_write = tokio::spawn(write(ws_send, async_recv));
    (async_send, handle_read, handle_write)
}

async fn read(mut recv: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) -> Result<(), &'static str> {

    let mut handler_map = handler_map();

    while let raw = recv.next().await {
        let raw = raw.ok_or(CONNECTION_BREAK)?.map_err(|_|CONNECTION_BREAK)?;
        let raw = raw.to_text().map_err(|_|PARSE_FAILED)?;
        println!("raw msg: {}", raw);
        let raw: Value = serde_json::from_str(raw).map_err(|_|PARSE_FAILED)?;

        let mut cmd: Map<String, Value>= serde_json::from_value(raw).map_err(|_|PARSE_FAILED)?;
        let cmd_type: String = serde_json::from_value(
            cmd.remove("type").ok_or(PARSE_FAILED)?).map_err(|_|PARSE_FAILED)?;
        println!("cmd_type: {}", cmd_type);
        let handler = handler_map.get(&cmd_type).ok_or(PARSE_FAILED)?;
        let data = cmd.remove("data").ok_or(PARSE_FAILED)?;
        println!("data: {}", data.to_string());
        handler(data)?;
    }
    Ok(())
}

const SHOULD_WRITE_CLOSE: &str = "应写通道关闭";
async fn write(mut send: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>
               , mut should_write: UnboundedReceiver<Value>) -> Result<(), &'static str> {
    while let msg = should_write.recv().await {
        let msg = msg.ok_or(SHOULD_WRITE_CLOSE)?.to_string();
        println!("should write: {}", msg);
        send.send(Message::Text(msg)).map_err(|_|CONNECTION_BREAK).await?;
    }
    Ok(())
}