use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use actix::*;
use actix_files::{Files, NamedFile};
use actix_web::{
    middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;

mod server;
mod session;

// 前端页面
async fn index() -> impl Responder {
    NamedFile::open_async("./wschatsrv1/static/index.html").await.unwrap()
}

/// 聊天 websocket 入口点路由
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        session::WsChatSession {
            id: 0,
            hb: Instant::now(),
            room: "main".to_owned(),
            name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}


/// 显示状态
async fn get_count(count: web::Data<AtomicUsize>) -> impl Responder {
    let current_count = count.load(Ordering::SeqCst);
    format!("Visitors: {current_count}")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // 持有访问者个数
    let app_state = Arc::new(AtomicUsize::new(0));

    // 启动服务器
    let server = server::ChatServer::new(app_state.clone()).start();

    let port = 48080;
    log::info!("starting HTTP server at http://localhost:{:?}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(app_state.clone()))
            .app_data(web::Data::new(server.clone()))
            .service(web::resource("/").to(index))
            .route("/count", web::get().to(get_count))
            .route("/ws", web::get().to(chat_route))
            .service(Files::new("/static", "./wschatsrv1/static"))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
