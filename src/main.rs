use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::time::Duration;
use tokio::task;

#[derive(Deserialize)]
struct TaskPayload {
    // sleep_time in milliseconds
    sleep_time: u64,
    message: String,
}

#[post("/sleep")]
async fn sleep_handler(payload: web::Json<TaskPayload>) -> impl Responder {
    // Call the function to start a background task
    start_background_task(payload.sleep_time, payload.message.clone());

    // Respond immediately
    HttpResponse::Ok().body("Job started")
}

fn start_background_task(sleep_time: u64, message: String) {
    let duration = Duration::from_millis(sleep_time);
    // For test purposes, create 1000 tasks
    for i in 0..1000 {
        let message = message.clone();
        let duration = duration;
        task::spawn(async move {
            tokio::time::sleep(duration).await;
            println!("{} {}", message, i);
        });
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(sleep_handler))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
