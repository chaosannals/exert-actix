use actix_web::{App, HttpServer};

mod api;

use api::varia::*;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(user_index)
            .service(ip_index)
            .service(fs_index)
            .service(now_index)
    })
    .bind("0.0.0.0:20080")?
    .run()
    .await
}
