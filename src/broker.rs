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
}
