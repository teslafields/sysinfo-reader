
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use mpsc::{Sender, Receiver, channel};
use actix_web::{dev::Server, middleware, rt, web, App,
                HttpResponse, HttpServer};
use crate::systats::Builder;
use crate::schema::SysinfoSchemaBuilder;


async fn all_info(systats: web::Data<Arc<SysinfoSchemaBuilder>>) -> HttpResponse {
    //if let Ok(sys_sts) = sysinfo_stats.read() {
    //    let data: SysinfoPayload = SysinfoPayload::new();
    //    return HttpResponse::Ok().json(data);
    //}
    //if let Ok(systats) = systatslock.read() {
    //    if let Some(payload) = systats.build_sysinfo_json() {
    //        return HttpResponse::Ok().json(payload);
    //    }
    //}
    //HttpResponse::ServiceUnavailable().body("Internal Error".to_string())
    //return HttpResponse::Ok().json2(systats.get_payload());
    if let Some(payload) = systats.get_payload() {
        return HttpResponse::Ok().json(systats.get_payload());
    }
    HttpResponse::ServiceUnavailable().body("Internal Error".to_string())

}

fn run_app(tx: Sender<Server>, systats: Arc<SysinfoSchemaBuilder>)
    -> std::io::Result<()> {
    let mut sys = rt::System::new("test");
    //let s = sysinfo_stats.clone();
    // srv is server controller type, `dev::Server`
    // let builder_arc = Arc::new(RwLock::new(systats));
    let srv = HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // enable json handling
            .data(web::JsonConfig::default().limit(4096))
            // set application data
            .data(systats.clone())
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

pub fn start_server(systats: Arc<SysinfoSchemaBuilder>) -> Receiver<Server> {
    std::env::set_var("RUST_LOG", "actix_web=info,actix_server=trace");
    env_logger::init();

    let (tx, rx) = channel();

    thread::spawn(move || {
        let _ = run_app(tx, systats);
    });

    rx
}

pub fn stop_server(server_handler: &Receiver<Server>) {
    let srv = server_handler.recv().unwrap();
    rt::System::new("").block_on(srv.stop(true));
}
