use actix_web::{web, middleware, App, HttpServer};

mod app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let hello_config = app::hello::make_config();
    let port = 44444;
    log::info!("starting HTTP server at http://localhost:{:?}", port);

    HttpServer::new(move || {
        // 初始的时候会生成多个线程(默认好像是CPU线程数)相互独立，所以这里面的变量也是多份的且在内存常驻。
        let hello_config_arc = hello_config.clone(); // 异步引用，即使这里 clone 多分也只引用了外部的 hello_config
        
        log::info!("on new.");
        App::new()
            .wrap(middleware::Logger::default())
            // configure 不支持 arc ，类型又解不出来，只能直接多套一层闭包（只是为了让类型对上。。）了。再调一次。
            .service(web::scope("/hello").configure(|cfg| {hello_config_arc(cfg)}))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}