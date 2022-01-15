
use std::io;
use std::io::Error;
use std::sync::mpsc;
use std::{thread, time};
use mpsc::{Sender, Receiver, channel};
use signal_hook::{consts::signal::*, iterator::Signals};
use actix_web::{dev::Server, middleware, rt, web, App, HttpRequest,
                HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Info {
    username: String,
}

async fn index(item: web::Json<Info>) -> HttpResponse {
    println!("model: {:?}", &item);
    HttpResponse::Ok().json(item.0) // <- send response
}

fn run_app(tx: Sender<Server>) -> std::io::Result<()> {
    let mut sys = rt::System::new("test");
    // srv is server controller type, `dev::Server`
    let srv = HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(4096))
            .service(web::resource("/index.html").to(|| async { "Hello world!" }))
            .service(web::resource("/").route(web::post().to(index)))
    })
    .bind("127.0.0.1:8080")?
    .run();
    // send server controller to main thread
    let _ = tx.send(srv.clone());
    // run future
    sys.block_on(srv)
}

pub fn start_server() -> Receiver<Server> {
    std::env::set_var("RUST_LOG", "actix_web=info,actix_server=trace");
    env_logger::init();

    let (tx, rx) = channel();

    thread::spawn(move || {
        let _ = run_app(tx);
    });

    rx
}

pub fn stop_server(server_handler: &Receiver<Server>) {
    let srv = server_handler.recv().unwrap();
    rt::System::new("").block_on(srv.stop(true));
}
