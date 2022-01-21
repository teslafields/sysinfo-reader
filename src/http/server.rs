
use std::io;
use std::io::Error;
use std::sync::{mpsc, Arc, RwLock};
use std::{thread, time};
use mpsc::{Sender, Receiver, channel};
use signal_hook::{consts::signal::*, iterator::Signals};
use actix_web::{dev::Server, middleware, rt, web, App, HttpRequest,
                HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use super::super::SysinfoStats;
use crate::schema::SysinfoPayload;


async fn all_info(sysinfo_stats: web::Data<Arc<RwLock<SysinfoStats>>>) -> HttpResponse {
    if let Ok(sys_sts) = sysinfo_stats.read() {
        let data: SysinfoPayload = SysinfoPayload::new();
        return HttpResponse::Ok().json(data);
    }
    HttpResponse::ServiceUnavailable().body("Internal Error".to_string())
}

fn run_app(tx: Sender<Server>, sysinfo_stats: Arc<RwLock<SysinfoStats>>)
    -> std::io::Result<()> {
    let mut sys = rt::System::new("test");
    //let s = sysinfo_stats.clone();
    // srv is server controller type, `dev::Server`
    let srv = HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // enable json handling
            .data(web::JsonConfig::default().limit(4096))
            // set application data
            .data(sysinfo_stats.clone())
            .service(web::resource("/").to(|| async { "Hello world!" }))
            .service(web::resource("/all_info").route(web::get().to(all_info)))
    })
    // Set the number of threads for the server (default is nrcpu)
    .workers(1)
    .bind("127.0.0.1:8080")?
    .run();
    // send server controller to main thread
    let _ = tx.send(srv.clone());
    // run future
    sys.block_on(srv)
}

pub fn start_server(sysinfo_stats: Arc<RwLock<SysinfoStats>>) -> Receiver<Server> {
    std::env::set_var("RUST_LOG", "actix_web=info,actix_server=trace");
    env_logger::init();

    let (tx, rx) = channel();

    thread::spawn(move || {
        let _ = run_app(tx, sysinfo_stats);
    });

    rx
}

pub fn stop_server(server_handler: &Receiver<Server>) {
    let srv = server_handler.recv().unwrap();
    rt::System::new("").block_on(srv.stop(true));
}
