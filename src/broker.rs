use crate::task_registry::Payload;
use redis::{AsyncCommands, RedisResult};

pub trait TaskTracker {
    async fn create_task_tracker(&self, payload: &Payload) -> RedisResult<()>;
    async fn get_task_tracker(&self, task_id: &str) -> RedisResult<Payload>;
    async fn delete_task_tracker(&self, task_id: &str) -> RedisResult<()>;
    async fn list_task_trackers(&self) -> RedisResult<Vec<Payload>>;
}

pub struct Broker {
    connection_string: String,
}

#[allow(dead_code)]
impl Broker {
    pub fn new(connection_string: &str) -> Broker {
        Broker {
            connection_string: connection_string.to_string(),
        }
    }

    async fn connect(&self) -> RedisResult<redis::aio::MultiplexedConnection> {
        let client = redis::Client::open(self.connection_string.as_str())?;
        let con = client.get_multiplexed_async_connection().await?;
        Ok(con)
    }

    pub async fn push_task(&self, payload: &Payload) -> RedisResult<()> {
        let mut con = self.connect().await?;
        let serialized_payload = serde_json::to_string(payload);
        con.lpush("celery", serialized_payload.unwrap()).await?;
        Ok(())
    }

    pub async fn list_tasks(&self) -> RedisResult<Vec<Payload>> {
        let mut con = self.connect().await?;
        let tasks: Result<Vec<String>, redis::RedisError> = con.lrange("celery", 0, -1).await;
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

    pub async fn get_task(&self, task_id: &str) -> RedisResult<Payload> {
        let mut con = self.connect().await?;
        let tasks: Result<Vec<String>, redis::RedisError> = con.lrange("celery", 0, -1).await;

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

    pub async fn delete_task(&self, task_id: &str) -> RedisResult<()> {
        let mut con = self.connect().await?;
        let tasks: Result<Vec<String>, redis::RedisError> = con.lrange("celery", 0, -1).await;

        for task in tasks.unwrap() {
            let payload: Payload = serde_json::from_str(task.as_str()).unwrap();
            if payload.headers.id == task_id {
                con.lrem("celery", 1, task).await?;
                return Ok(());
            }
        }
        Err(redis::RedisError::from((
            redis::ErrorKind::ResponseError,
            "Task not found",
        )))
    }
}

impl TaskTracker for Broker {
    async fn create_task_tracker(&self, payload: &Payload) -> RedisResult<()> {
        let mut con = self.connect().await?;
        let serialized_payload = serde_json::to_string(payload);
        con.hset(
            "celery_trackers",
            payload.headers.id.as_str(),
            serialized_payload.unwrap(),
        )
        .await?;
        Ok(())
    }

    async fn get_task_tracker(&self, task_id: &str) -> RedisResult<Payload> {
        let mut con = self.connect().await?;
        let payload: String = con.hget("celery_trackers", task_id).await?;
        let payload: Payload = serde_json::from_str(payload.as_str()).unwrap();
        Ok(payload)
    }

    async fn delete_task_tracker(&self, task_id: &str) -> RedisResult<()> {
        let mut con = self.connect().await?;
        con.hdel("celery_trackers", task_id).await?;
        Ok(())
    }

    async fn list_task_trackers(&self) -> RedisResult<Vec<Payload>> {
        let mut con = self.connect().await?;
        let trackers: Result<Vec<String>, redis::RedisError> = con.hvals("celery_trackers").await;
        let mut res_trackers: Vec<Payload> = Vec::new();
        match trackers {
            Ok(trackers) => {
                for tracker in trackers {
                    let payload: Payload = serde_json::from_str(tracker.as_str()).unwrap();
                    res_trackers.push(payload);
                }
            }
            Err(e) => {
                log::error!("Error: {:?}", e);
            }
        }
        Ok(res_trackers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_registry::create_task;
    use serde_json::Value;

    #[tokio::test]
    async fn test_push_task() {
        let broker = Broker::new("redis://localhost:6379");
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let result = broker.push_task(&payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let broker = Broker::new("redis://localhost:6379");
        let result = broker.list_tasks().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_task() {
        let broker = Broker::new("redis://localhost:6379");
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let _ = broker.push_task(&payload).await;
        let result = broker.get_task(&payload.headers.id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_task_tracker() {
        let broker = Broker::new("redis://localhost:6379");
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let result = broker.create_task_tracker(&payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_task_tracker() {
        let broker = Broker::new("redis://localhost:6379");
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let _ = broker.create_task_tracker(&payload).await;
        let result = broker.get_task_tracker(&payload.headers.id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_task_tracker() {
        let broker = Broker::new("redis://localhost:6379");
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let _ = broker.create_task_tracker(&payload).await;
        let result = broker.delete_task_tracker(&payload.headers.id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_task_trackers() {
        let broker = Broker::new("redis://localhost:6379");
        let result = broker.list_task_trackers().await;
        assert!(result.is_ok());
    }
}
