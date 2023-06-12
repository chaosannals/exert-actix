use std::{
    io,
    thread,
};

use futures_util::{SinkExt as _, StreamExt as _};
use actix_web::web::Bytes;
use awc::ws;
use tokio::{select, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;

#[actix_web::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting echo WebSocket client");

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<String>();
    let mut cmd_rx = UnboundedReceiverStream::new(cmd_rx);


    // 启动阻塞线程读取终端输入
    let input_thread = thread::spawn(move || loop {
        let mut cmd = String::with_capacity(32);

        if io::stdin().read_line(&mut cmd).is_err() {
            log::error!("error reading line");
            return;
        }

        cmd_tx.send(cmd).unwrap();
    });

    let (res, mut ws) = awc::Client::new()
        .ws("ws://127.0.0.1:48080/ws")
        .connect()
        .await
        .unwrap();

    log::debug!("response: {res:?}");
    log::info!("connected; server will echo messages sent");

    loop {
        //  stream 必须实现 Unpin 才能被 select! 
        select! {
            Some(msg) = ws.next() => {
                match msg {
                    Ok(ws::Frame::Text(txt)) => {
                        // 打印接收数据
                        log::info!("Server: {txt:?}")
                    }

                    Ok(ws::Frame::Ping(_)) => {
                        // 响应 PING
                        //log::info!("Server PING");
                        ws.send(ws::Message::Pong(Bytes::new())).await.unwrap();
                    }

                    _ => {}
                }
            }

            Some(cmd) = cmd_rx.next() => {
                if cmd.is_empty() {
                    continue;
                }
                // 发送 终端 输入信息
                ws.send(ws::Message::Text(cmd.into())).await.unwrap();
            }

            else => break
        }
    }

    input_thread.join().unwrap();
}
