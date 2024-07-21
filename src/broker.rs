use std::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::models::Payload;
use redis::{aio::MultiplexedConnection, AsyncCommands, RedisResult};
use serde_json::Value;

pub trait TaskTracker {
    async fn create_task_tracker(&self, payload: &Payload) -> RedisResult<()>;
    async fn get_task_tracker(&self, task_id: &str) -> RedisResult<Payload>;
    async fn delete_task_tracker(&self, task_id: &str) -> RedisResult<()>;
    async fn list_task_trackers(&self) -> RedisResult<Vec<Payload>>;
}

pub struct Broker {
    connection_string: String,
    con: Arc<Mutex<MultiplexedConnection>>,
}

#[allow(dead_code)]
impl Broker {
    pub async fn new(connection_string: &str) -> Arc<Mutex<Broker>> {
        let client = redis::Client::open(connection_string).unwrap();
        let con = client.get_multiplexed_async_connection().await;
        let con = con.unwrap();
        let arc_con = Arc::new(Mutex::new(con));

        let broker = Broker {
            connection_string: connection_string.to_string(),
            con: arc_con,
        };
        Arc::new(Mutex::new(broker))
    }

    pub async fn push_task(&self, payload: &Payload) -> RedisResult<()> {
        // let con = self.get_connection().await?;
        let con = self.con.clone();

        let serialized_payload: Result<String, serde_json::Error> = serde_json::to_string(payload);
        con.lock()
            .unwrap()
            .lpush("celery", serialized_payload.unwrap())
            .await?;
        Ok(())
    }

    async fn list_tasks(&self) -> RedisResult<Vec<Payload>> {
        let con = self.con.clone();
        let tasks: Result<Vec<String>, redis::RedisError> =
            con.lock().unwrap().lrange("celery", 0, -1).await;
        let mut res_tasks: Vec<Payload> = Vec::new();
        match tasks {
            Ok(tasks) => {
                for task in tasks {
                    let payload: Payload = serde_json::from_str(task.as_str()).unwrap();
                    res_tasks.push(payload);
                }
            }
            Err(e) => {
                log::error!("Error: {:?}", e);
            }
        }
        Ok(res_tasks)
    }

    async fn get_task(&self, task_id: &str) -> RedisResult<Payload> {
        let con = self.con.clone();
        let tasks: Result<Vec<String>, redis::RedisError> =
            con.lock().unwrap().lrange("celery", 0, -1).await;

        for task in tasks.unwrap() {
            let payload: Payload = serde_json::from_str(task.as_str()).unwrap();
            if payload.headers.id == task_id {
                return Ok(payload);
            }
        }
        Err(redis::RedisError::from((
            redis::ErrorKind::ResponseError,
            "Task not found",
        )))
    }

    async fn delete_task(&self, task_id: &str) -> RedisResult<()> {
        let con = self.con.clone();
        let tasks: Result<Vec<String>, redis::RedisError> =
            con.lock().unwrap().lrange("celery", 0, -1).await;

        for task in tasks.unwrap() {
            let payload: Payload = serde_json::from_str(task.as_str()).unwrap();
            if payload.headers.id == task_id {
                con.lock().unwrap().lrem("celery", 1, task).await?;
                return Ok(());
            }
        }
        Err(redis::RedisError::from((
            redis::ErrorKind::ResponseError,
            "Task not found",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::create_task;
    use serde_json::Value;

    #[tokio::test]
    async fn test_push_task() {
        let broker = Broker::new("redis://localhost:6379").await;
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let result = broker.lock().unwrap().push_task(&payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let broker = Broker::new("redis://localhost:6379").await;
        let result = broker.lock().unwrap().list_tasks().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_task() {
        let broker = Broker::new("redis://localhost:6379").await;
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let _ = broker.lock().unwrap().push_task(&payload).await;
        let result = broker.lock().unwrap().get_task(&payload.headers.id).await;
        assert!(result.is_ok());
    }
}
