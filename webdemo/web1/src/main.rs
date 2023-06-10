use actix_web::{
    get,
    web,
    middleware,
    App,
    HttpServer,
    HttpRequest,
    Responder,
};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[get("/")]
async fn index(req: HttpRequest) -> String {
    format!("index: {:?}", req)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let port = 48080;
    log::info!("starting HTTP server at http://localhost:{:?}", port);

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(index)
            .service(greet)
    })
    .bind(("0.0.0.0", port))?
    .workers(4)
    .run()
    .await
}