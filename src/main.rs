mod apis;
mod beat;
mod broker;
mod models;
mod task_registry;
mod tracker;
mod utils;
use std::sync::Arc;

use actix_web::middleware::NormalizePath;
use actix_web::middleware::TrailingSlash;
use actix_web::{web, App, HttpServer};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = "0.0.0.0";
    let port = 8080;
    let bind_addr = format!("{}:{}", addr, port);
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    println!("Starting server at: {}", bind_addr);
    let broker = broker::Broker::new("redis://localhost:6379");
    let app_data = Arc::new(models::AppData {
        broker: Arc::new(broker),
    });
    HttpServer::new({
        let app_data = app_data.clone();
        move || {
            App::new()
                .wrap(NormalizePath::new(TrailingSlash::Trim))
                .service(web::scope("/api/v1").configure(apis::init_routes))
                .app_data(web::Data::new(app_data.clone()))
        }
    })
    .bind(bind_addr)?
    .run()
    .await
}
