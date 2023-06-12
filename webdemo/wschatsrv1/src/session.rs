use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

use crate::server;

/// 心跳间隔
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// 超时时间
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// 聊天会话
#[derive(Debug)]
pub struct WsChatSession {
    /// 会话唯一ID
    pub id: usize,

    /// 上一次心跳
    pub hb: Instant,

    /// 房间
    pub room: String,

    /// peer name
    pub name: Option<String>,

    /// Chat server
    pub addr: Addr<server::ChatServer>,
}



impl WsChatSession {
    /// 心跳
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // 检查客户端心跳
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // 心跳超时
                println!("Websocket Client heartbeat failed, disconnecting!");

                // 通知聊天服务器断开
                act.addr.do_send(server::Disconnect { id: act.id });

                // 停止 actor
                ctx.stop();

                // 不再发送心跳。
                return;
            }

            // 发送心跳。
            ctx.ping(b"");
        });
    }
}



impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// actor 开始. 注册 ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // 启动心跳。
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // 其他情况关闭聊天服务
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // 通知聊天服务器断开
        self.addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}


/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}



/// WebSocket 消息处理
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        log::debug!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            // 心跳
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }


            ws::Message::Text(text) => {
                let m = text.trim();
                // 处理 / 开头的消息
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/list" => {
                            // 发送房间列表，等待响应。
                            println!("List rooms");
                            self.addr
                                .send(server::ListRooms)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            for room in rooms {
                                                ctx.text(room);
                                            }
                                        }
                                        _ => println!("Something is wrong"),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx)
                            // .wait(ctx) pauses all events in context,
                            // so actor wont receive any new messages until it get list
                            // of rooms back
                        }
                        "/join" => {
                            if v.len() == 2 {
                                self.room = v[1].to_owned();
                                self.addr.do_send(server::Join {
                                    id: self.id,
                                    name: self.room.clone(),
                                });

                                ctx.text("joined");
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/name" => {
                            if v.len() == 2 {
                                self.name = Some(v[1].to_owned());
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }
                        _ => ctx.text(format!("!!! unknown command: {m:?}")),
                    }
                } else {
                    let msg = if let Some(ref name) = self.name {
                        format!("{name}: {m}")
                    } else {
                        m.to_owned()
                    };
                    // 发送消息到 chat server
                    self.addr.do_send(server::ClientMessage {
                        id: self.id,
                        msg,
                        room: self.room.clone(),
                    })
                }
            }

            // 不处理 二进制消息
            ws::Message::Binary(_) => println!("Unexpected binary"),

            // 
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
