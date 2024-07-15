use crate::broker;
use crate::tracker;
use crate::utils;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use chrono::prelude::*;

#[allow(dead_code)]
pub async fn beat_box(broker: Arc<Mutex<broker::Broker>>) {
    let tracker = tracker::MongoDB::new("mongodb://localhost:27017").await;
    let active_tasks = tracker.list_tasks().await;
    let now = Utc::now();
    let time_tolerance_in_seconds = 60;
    for mut task in active_tasks {
        let next_run = &task.next_call.unwrap();
        let next_run_converted: DateTime<Utc> = next_run.to_system_time().into();

        let time_diff = now.signed_duration_since(next_run_converted).num_seconds();

        if time_diff.abs() <= time_tolerance_in_seconds {
            let now_in_system_time: SystemTime = SystemTime::now();
            let bson_now = bson::DateTime::from(now_in_system_time);
            task.last_called_at = Some(bson_now);
            let cron = task.cron.clone().unwrap();
            let next_call = utils::next_call(next_run_converted, cron);
            let next_call_system_time: SystemTime = next_call.into();
            task.next_call = Some(bson::DateTime::from(next_call_system_time));

            let broker = broker.lock().unwrap();
            broker.push_task(&task).await.unwrap();
        }
    }
}
