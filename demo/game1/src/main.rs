use actix::prelude::*;
use std::time::Duration;

#[derive(Message)]
#[rtype(result = "()")]
struct Ping {
    pub id: usize,
}

// Actor 定义
struct Game {
    counter: usize,
    name: String,
    recipient: Recipient<Ping>,
}

impl Actor for Game {
    type Context = Context<Game>;
}

// 消息处理
impl Handler<Ping> for Game {
    type Result = ();

    fn handle(&mut self, msg: Ping, ctx: &mut Context<Self>) {
        self.counter += 1;

        if self.counter > 10 {
            System::current().stop();
        } else {
            println!("[{0}] Ping received {1}", self.name, msg.id);

            // 等待 100 纳秒
            ctx.run_later(Duration::new(0, 100), move |act, _| {
                act.recipient.do_send(Ping { id: msg.id + 1 });
            });
        }
    }
}

fn main() {
    let system = System::new();

    // 使用 Game::create 创建接收对象，可以推迟 actor 创建。（提供上下文）
    let _addr = system.block_on(async {
        // 创建方法提供一个上下文，最后产出与该上下文相关的 actor
        Game::create(|ctx| {
            // 通过上下文创建一个地址 属于第一个 actor （最后函数返回的 actor）
            // 使用 Game::create 的好处，在没有创建 actor 前，先拿到了地址。
            let addr = ctx.address();

            // 创建第二个 actor 并开始
            let addr2 = Game {
                counter: 0,
                name: String::from("Game 2"),
                recipient: addr.recipient(),
            }
            .start();

            // 第二个 actor 发送 ping
            addr2.do_send(Ping { id: 10 });

            // 创建第一个 actor
            Game {
                counter: 0,
                name: String::from("Game 1"),
                recipient: addr2.recipient(),
            }
        });
    });

    system.run().unwrap();
}