use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

/// 心跳间隔
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// 超时时间
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct MyWebSocket {
    // 最近一次客户端 PING 时刻
    last_heartbeat: Instant,
}

impl MyWebSocket {
    pub fn new() -> Self {
        Self { last_heartbeat: Instant::now() }
    }

    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, | act, ctx | {
            // 检查客户端心跳
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                // 心跳超时
                println!("Websocket Client heartbeat failed, disconnecting!");

                // 停止 actor
                ctx.stop();

                // 不再发送 PING
                return;
            }

            // 向客户端发送 PING
            ctx.ping(b"");
        });
    }
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// actor 开始周期事件，启动心跳.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
    }
}

/// 实现 `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // 处理 websocket 消息
        println!("WS: {msg:?}");
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = Instant::now(); // 接收 PING 改写心跳记录。
                ctx.pong(&msg); // 向客户端发PONG
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now(); // 接收 PONG 改写心跳记录。
            }
            Ok(ws::Message::Text(text)) => ctx.text(text), // 文本数据
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin), // 二进制数据
            Ok(ws::Message::Close(reason)) => {
                // websocket 关闭事件
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}
