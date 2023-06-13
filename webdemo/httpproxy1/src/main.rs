use std::net::ToSocketAddrs;

use actix_web::{
    dev::PeerAddr, error, http::Method, middleware, web, App, Error, HttpRequest, HttpResponse,
    HttpServer,
};
use awc::Client;
use futures_util::StreamExt as _;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use url::Url;

// 第二种代理方式加了前缀
const REQWEST_PREFIX: &str = "/using-reqwest";


/// 使用 `awc` 实现第一种代理
/// 配置 rustls 不起效， openssl 需要 c 编译，配置又麻烦。
async fn forward(
    req: HttpRequest,
    payload: web::Payload,
    peer_addr: Option<PeerAddr>,
    url: web::Data<Url>,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    log::info!("forwarding to 0");

    let mut new_url = (**url).clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    log::info!("forwarding to 1");

    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();

    // TODO: 此示例只实现一个标准的代理头
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = match peer_addr {
        Some(PeerAddr(addr)) => {
            forwarded_req.insert_header(("x-forwarded-for", addr.ip().to_string()))
        }
        None => forwarded_req,
    };

    log::info!("forwarding to 2");

    let res = forwarded_req
        .send_stream(payload)
        .await
        .map_err(error::ErrorInternalServerError)?;


    log::info!("forwarding to 3");

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.insert_header((header_name.clone(), header_value.clone()));
    }

    log::info!("forwarding to 4");

    Ok(client_resp.streaming(res))
}


/// 使用 `reqwest` 实现第二种代理。不支持 HTTP2
/// https 使用 rustls 配置起效。
async fn forward_reqwest(
    req: HttpRequest,
    mut payload: web::Payload,
    method: Method,
    peer_addr: Option<PeerAddr>,
    url: web::Data<Url>,
    client: web::Data<reqwest::Client>,
) -> Result<HttpResponse, Error> {
    log::info!("reqwest to 0");

    let path = req
        .uri()
        .path()
        .strip_prefix(REQWEST_PREFIX)
        .unwrap_or(req.uri().path());

    let mut new_url = (**url).clone();
    new_url.set_path(path);
    new_url.set_query(req.uri().query());

    let (tx, rx) = mpsc::unbounded_channel();

    log::info!("reqwest to 1");

    actix_web::rt::spawn(async move {
        while let Some(chunk) = payload.next().await {
            tx.send(chunk).unwrap();
        }
    });

    log::info!("reqwest to 2");
    let forwarded_req = client
        .request(method, new_url)
        .body(reqwest::Body::wrap_stream(UnboundedReceiverStream::new(rx)));

    // TODO: 此示例只实现一个标准的代理头
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = match peer_addr {
        Some(PeerAddr(addr)) => forwarded_req.header("x-forwarded-for", addr.ip().to_string()),
        None => forwarded_req,
    };

    log::info!("reqwest to 3");

    //let forwarded_req = forwarded_req.header(":Authority:", "developer.mozilla.org");

    let res = forwarded_req
        .send()
        .await
        .map_err(error::ErrorInternalServerError)?;


    log::info!("reqwest to 4");

    let mut client_resp = HttpResponse::build(res.status());
    // HTTP2 冒号开头的头部，reqwest 不支持 HTTP2 时添加会报500。
    // client_resp.insert_header((":Authority", "developer.mozilla.org"));
    // client_resp.insert_header((":Method", "GET"));
    // client_resp.insert_header((":Path", path));
    // client_resp.insert_header((":Scheme", "https"));

    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.insert_header((header_name.clone(), header_value.clone()));
        let hn = header_name.clone();
        log::info!("reqwest header {hn}");
    }

    log::info!("reqwest to 5");
    Ok(client_resp.streaming(res.bytes_stream()))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // let forward_url = Url::parse("https://developer.mozilla.org/").unwrap();
    let forward_url = Url::parse("https://baidu.com/").unwrap();
    let port = 48080;
    log::info!("starting HTTP server at http://127.0.0.1:{}", port);
    log::info!("forwarding to {forward_url}");

    let reqwest_client = reqwest::Client::default();

    HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(Client::default()))
                .app_data(web::Data::new(reqwest_client.clone()))
                .app_data(web::Data::new(forward_url.clone()))
                .wrap(middleware::Logger::default())
                .service(web::scope(REQWEST_PREFIX).default_service(web::to(forward_reqwest)))
                // .default_service(web::to(forward))
                .default_service(web::to(forward_reqwest))
        })
        .bind(("0.0.0.0", port))?
        .workers(2)
        .run()
        .await
}
