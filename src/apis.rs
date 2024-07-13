use crate::models::{AppData, SubmissionStatus, TaskSubmitRequest, TaskSubmitResponse};
use crate::task_registry::create_task;
use actix_web::{get, post, web, HttpResponse, Responder};
// use log::debug;
use std::sync::Arc;

#[post("/submit-task")]
async fn submit_task_api_view(
    task: web::Json<TaskSubmitRequest>,
    app_data: web::Data<Arc<AppData>>,
) -> impl Responder {
    let serialized_task = create_task(&task.task_name, task.args.clone(), task.kwargs.clone());
    // debug!("Serialized task: {:?}", serialized_task);

    app_data.broker.push_task(&serialized_task).await.unwrap();
    HttpResponse::Ok()
        .content_type("application/json")
        .append_header(("X-Task-Id", serialized_task.headers.id))
        .json(TaskSubmitResponse {
            status: SubmissionStatus::Received,
        })
}

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"message": "Hello, world!"}"#)
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"status": "UP"}"#)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(submit_task_api_view);
    cfg.service(root);
    cfg.service(health);
}
