use actix::prelude::*;

// 定义消息
#[derive(Message)]
#[rtype(result = "usize")]
struct Sum(usize, usize);

// 此示例没有进入系统消息

// Actor 定义
struct Calculator;

impl Actor for Calculator {
    type Context = Context<Self>;
}

// 实现 `Handler` on `Calculator` for the `Sum` message.
impl Handler<Sum> for Calculator {
    type Result = usize; // 响应结果

    fn handle(&mut self, msg: Sum, _ctx: &mut Context<Self>) -> Self::Result {
        msg.0 + msg.1
    }
}

#[actix::main] // 异步标签
async fn main() {
    let addr = Calculator.start();
    let res = addr.send(Sum(10, 5)).await; // <- 发送消息获取结果

    match res {
        Ok(result) => println!("SUM: {}", result),
        _ => println!("Communication to the actor has failed"),
    }
}