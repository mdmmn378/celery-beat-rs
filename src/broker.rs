use crate::task_registry::Payload;
use redis::{Commands, RedisResult};

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

    fn connect(&self) -> RedisResult<redis::Connection> {
        let client = redis::Client::open(self.connection_string.as_str())?;
        client.get_connection()
    }

    pub fn push_task(&self, payload: &Payload) -> RedisResult<()> {
        let mut con = self.connect()?;
        let serialized_payload = serde_json::to_string(payload);
        con.lpush("celery", serialized_payload.unwrap())
    }

    pub fn list_tasks(&self) -> RedisResult<Vec<Payload>> {
        let mut con = self.connect()?;
        let tasks: Result<Vec<String>, redis::RedisError> = con.lrange("celery", 0, -1);
        let mut res_tasks: Vec<Payload> = Vec::new();
        match tasks {
            Ok(tasks) => {
                for task in tasks {
                    let payload: Payload = serde_json::from_str(task.as_str()).unwrap();
                    res_tasks.push(payload);
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
        Ok(res_tasks)
    }

    pub fn get_task(&self, task_id: &str) -> RedisResult<Payload> {
        let mut con = self.connect()?;
        let tasks: Result<Vec<String>, redis::RedisError> = con.lrange("celery", 0, -1);

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

    pub fn delete_task(&self, task_id: &str) -> RedisResult<()> {
        let mut con = self.connect()?;
        let tasks: Result<Vec<String>, redis::RedisError> = con.lrange("celery", 0, -1);

        for task in tasks.unwrap() {
            let payload: Payload = serde_json::from_str(task.as_str()).unwrap();
            if payload.headers.id == task_id {
                let _: () = con.lrem("celery", 1, task)?;
                return Ok(());
            }
        }
        Err(redis::RedisError::from((
            redis::ErrorKind::ResponseError,
            "Task not found",
        )))
    }

    pub fn create_task_tracker(&self, payload: &Payload) -> RedisResult<()> {
        let mut con = self.connect()?;
        // there is a hset called trackers, and the key is the task_id, and the value is the payload
        let serialized_payload = serde_json::to_string(payload);
        con.hset(
            "celery_trackers",
            payload.headers.id.as_str(),
            serialized_payload.unwrap(),
        )
    }

    pub fn get_task_tracker(&self, task_id: &str) -> RedisResult<Payload> {
        let mut con = self.connect()?;
        let payload: String = con.hget("celery_trackers", task_id)?;
        let payload: Payload = serde_json::from_str(payload.as_str()).unwrap();
        Ok(payload)
    }

    pub fn delete_task_tracker(&self, task_id: &str) -> RedisResult<()> {
        let mut con = self.connect()?;
        let _: () = con.hdel("celery_trackers", task_id)?;
        Ok(())
    }

    pub fn list_task_trackers(&self) -> RedisResult<Vec<Payload>> {
        let mut con = self.connect()?;
        let trackers: Result<Vec<String>, redis::RedisError> = con.hvals("celery_trackers");
        let mut res_trackers: Vec<Payload> = Vec::new();
        match trackers {
            Ok(trackers) => {
                for tracker in trackers {
                    let payload: Payload = serde_json::from_str(tracker.as_str()).unwrap();
                    res_trackers.push(payload);
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
        Ok(res_trackers)
    }
}

#[cfg(test)]
mod tests {
    use crate::task_registry::create_task;
    use serde_json::Value;

    use super::*;

    #[test]
    fn test_push_task() {
        let broker = Broker::new("redis://localhost:6379");
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let result = broker.push_task(&payload);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_tasks() {
        let broker = Broker::new("redis://localhost:6379");
        let result = broker.list_tasks();
        assert!(result.is_ok());
        // println!("{:?}", result.unwrap());
    }

    #[test]
    fn test_get_task() {
        let broker = Broker::new("redis://localhost:6379");
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let _ = broker.push_task(&payload);
        let result = broker.get_task(&payload.headers.id);
        assert!(result.is_ok());
        // println!("{:?}", result.unwrap());
    }

    #[test]
    fn test_delete_task() {
        let broker = Broker::new("redis://localhost:6379");
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let _ = broker.push_task(&payload);
        let result = broker.delete_task(&payload.headers.id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_task_tracker() {
        let broker = Broker::new("redis://localhost:6379");
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let result = broker.create_task_tracker(&payload);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_task_tracker() {
        let broker = Broker::new("redis://localhost:6379");
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let _ = broker.create_task_tracker(&payload);
        let result = broker.get_task_tracker(&payload.headers.id);
        assert!(result.is_ok());
        // println!("{:?}", result.unwrap());
    }

    #[test]
    fn test_delete_task_tracker() {
        let broker = Broker::new("redis://localhost:6379");
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        let _ = broker.create_task_tracker(&payload);
        let result = broker.delete_task_tracker(&payload.headers.id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_task_trackers() {
        let broker = Broker::new("redis://localhost:6379");
        let result = broker.list_task_trackers();
        assert!(result.is_ok());
        // println!("{:?}", result.unwrap());
    }
}
