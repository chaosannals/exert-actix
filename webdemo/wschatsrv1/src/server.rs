use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

/// 服务器指令消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);


/// 会话连接消息
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// 会话断开消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// 加入房间消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// 用户 ID
    pub id: usize,

    /// Room name
    pub name: String,
}

/// 用户消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// 用户会话 ID
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
}

/// 列举活跃房间
pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

/// 聊天服务器，管理房间和会话
#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
    visitor_count: Arc<AtomicUsize>,
}

impl ChatServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChatServer {
        // 创建默认房间
        let mut rooms = HashMap::new();
        rooms.insert("main".to_owned(), HashSet::new());

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
            visitor_count,
        }
    }
}

impl ChatServer {
    /// 发送房间（类似群）消息
    fn send_message(&self, room: &str, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }
}


/// Make actor from `ChatServer`
impl Actor for ChatServer {
    /// 实现基于 ChatServer 上下文
    type Context = Context<Self>;
}

/// 处理连接消息
///
/// 注册新会话并赋予唯一ID
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // 通知同一房间的用户。
        self.send_message("main", "Someone joined", 0);

        // 注册会话赋予ID
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // 自动加入主房间
        self.rooms
            .entry("main".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        self.send_message("main", &format!("Total visitors {count}"), 0);

        // 返回 ID
        id
    }
}


/// 处理断开消息
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

        // 从会话列表里删除会话
        if self.sessions.remove(&msg.id).is_some() {
            // 从所有房间里面删除会话
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }

        // 发送消息通知相关房间用户。
        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }
    }
}


/// 处理用户消息
impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        // 排除本身会话，向指定房间群发消息
        self.send_message(&msg.room, msg.msg.as_str(), msg.id);
    }
}



/// 处理房间列表消息
impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}



/// 处理加入消息。断开老房间，加入新房间。
impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { id, name } = msg;
        let mut rooms = Vec::new();

        // 把会话从所有房间里删除。
        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }
        // 发送消息给其他用户
        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }

        // 加入新房间。
        self.rooms
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        // 发送消息给房间里的其他用户
        self.send_message(&name, "Someone connected", id);
    }
}
