use actix_web::{get, web, Result, Responder};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Url1Param {
    controller: String,
    action: i32,
}

#[derive(Serialize)]
struct IndexResponse {
    code: i32,
    message: String,
}

#[get("/index.json")]
async fn index() -> impl Responder {
    web::Json(IndexResponse { code: 0, message: "Ok".to_owned() })
}

#[get("/{controller}/{action}.json")]
async fn index1(param: web::Path<Url1Param>) -> impl Responder {
    web::Json(param.into_inner())
}
