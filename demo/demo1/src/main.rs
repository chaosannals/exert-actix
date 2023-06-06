use actix::{Actor, Context, System};

struct MyActor;

impl Actor for MyActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("I am alive!");
        System::current().stop(); // 这里让总消息循环退出，一般服务器不会出现这个代码。
    }
}


fn main() {
    println!("Hello, world!");
    let system = actix::System::new();

    let _addr = system.block_on(async { MyActor.start() });

    system.run(); // 这里是个消息循环
}
