use actix_web::{get, web, Responder};

use exert_actix_common::fs::*;
use exert_actix_common::ip::*;

use chrono::prelude::*;
use rand::distributions::Uniform;
use rand::prelude::*;
use std::path::Path;
use std::time::SystemTime;

fn make_set(length: i32) -> Vec<i32> {
    let mut r = vec![];
    let u = Uniform::new(0, 10000000);
    for _ in 0..length {
        r.push(u.sample(&mut rand::thread_rng()));
    }
    return r;
}

#[get("/{id}/{name}/index.html")]
async fn user_index(info: web::Path<(u32, String)>) -> impl Responder {
    let a: Vec<i32> = make_set(10000);
    format!("Hello {0}! => {1:?}", info.1, a)
}

#[get("/fs.html")]
async fn fs_index() -> impl Responder {
    let p = get_path_buf(Path::new("."), &vec!["rs"]);
    return format!("path: {0:?}", p);
}

#[get("/ip.html")]
async fn ip_index() -> impl Responder {
    let ip = get_wan_ip().await;
    format!("Ip {0}", ip.unwrap())
}

#[get("/now.html")]
async fn now_index() -> impl Responder {
    let today = Local::now().format("%Y-%m-%d").to_string();
    if let Ok(now) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        return format!("today: {0}, now {1}", today, now.as_nanos());
    }
    return format!("today {0} , get time failed.", today);
}

#[get("/index.html")]
async fn index() -> impl Responder {
    let today = Local::now().format("%Y-%m-%d").to_string();
    if let Ok(now) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        return format!("today: {0}, now {1}", today, now.as_nanos());
    }
    return format!("today {0} , get time failed.", today);
}
