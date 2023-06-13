use actix_web::{
    get,
    web,
    guard,
    middleware,
    App,
    HttpServer,
    HttpRequest,
    Responder,
};

mod api;
use api::ApiError;

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[get("/")]
async fn index(req: HttpRequest) -> String {
    format!("index: {:?}", req)
}

#[get("/error")]
async fn error() -> Result<String, ApiError> {
    Err(ApiError { message: "an".to_owned() })
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let port = 48080;
    log::info!("starting HTTP server at http://localhost:{:?}", port);

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api")
                    // .guard(guard::Header("content-type", "application/json"))
                    // .guard(guard::Get())
                    // .guard(guard::Post())
                    .service(api::apiparam::index)
                    .service(api::apiparam::index1)
            )
            .external_resource("/baidu", "https://baidu.com")
            .service(error)
            .service(index)
            .service(greet)
    })
    .bind(("0.0.0.0", port))?
    .workers(4)
    .run()
    .await
}