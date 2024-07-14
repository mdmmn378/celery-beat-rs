use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::utils;

#[derive(Serialize, Deserialize, Debug)]
pub struct Headers {
    pub lang: String,
    pub task: String,
    pub id: String,
    pub shadow: Option<String>,
    pub eta: Option<String>,
    pub expires: Option<String>,
    pub group: Option<String>,
    pub group_index: Option<u32>,
    pub retries: u32,
    pub timelimit: [Option<u32>; 2],
    pub root_id: String,
    pub parent_id: Option<String>,
    pub argsrepr: String,
    pub kwargsrepr: String,
    pub origin: String,
    pub ignore_result: bool,
    pub replaced_task_nesting: u32,
    pub stamped_headers: Option<String>,
    pub stamps: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeliveryInfo {
    pub exchange: String,
    pub routing_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Properties {
    pub correlation_id: String,
    pub reply_to: String,
    pub delivery_mode: u8,
    pub delivery_info: DeliveryInfo,
    pub priority: u8,
    pub body_encoding: String,
    pub delivery_tag: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskDetails {
    pub callbacks: Option<()>,
    pub errbacks: Option<()>,
    pub chain: Option<()>,
    pub chord: Option<()>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    pub body: String,
    #[serde(rename = "content-encoding")]
    pub content_encoding: String,
    #[serde(rename = "content-type")]
    pub content_type: String,
    pub headers: Headers,
    pub properties: Properties,
}

#[allow(dead_code)]
pub fn create_task(
    task: &str,
    args: Vec<Value>,
    kwargs: serde_json::Map<String, serde_json::Value>,
) -> Payload {
    let task_details = TaskDetails {
        callbacks: None,
        errbacks: None,
        chain: None,
        chord: None,
    };

    let body_content: Vec<Value> = vec![
        Value::Array(args),
        Value::Object(kwargs),
        Value::Object(
            serde_json::to_value(task_details)
                .unwrap()
                .as_object()
                .unwrap()
                .clone(),
        ),
    ];

    let body_serialized = serde_json::to_string(&body_content).unwrap();
    let body_encoded = BASE64_STANDARD.encode(body_serialized.as_bytes());

    Payload {
        body: body_encoded,
        content_encoding: "utf-8".to_string(),
        content_type: "application/json".to_string(),
        headers: Headers {
            lang: "py".to_string(),
            task: task.to_string(),
            id: Uuid::new_v4().to_string(),
            shadow: None,
            eta: None,
            expires: None,
            group: None,
            group_index: None,
            retries: 0,
            timelimit: [None, None],
            root_id: Uuid::new_v4().to_string(),
            parent_id: None,
            argsrepr: format!("{:?}", body_content[0]),
            kwargsrepr: format!("{:?}", body_content[1]),
            origin: utils::get_random_origin(), // TODO: get the origin from the request
            ignore_result: false,
            replaced_task_nesting: 0,
            stamped_headers: None,
            stamps: HashMap::new(),
        },
        properties: Properties {
            correlation_id: Uuid::new_v4().to_string(),
            reply_to: Uuid::new_v4().to_string(),
            delivery_mode: 2,
            delivery_info: DeliveryInfo {
                exchange: "".to_string(),
                routing_key: "celery".to_string(),
            },
            priority: 0,
            body_encoding: "base64".to_string(),
            delivery_tag: Uuid::new_v4().to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task() {
        let args = vec![Value::Number(1.into()), Value::Number(2.into())];
        let kwargs = serde_json::Map::new();
        let task = "src-py.main.add";
        let payload = create_task(task, args, kwargs);
        println!("{:?}", serde_json::to_string(&payload).unwrap());
        assert_eq!(payload.headers.task, task);
        assert_eq!(payload.properties.body_encoding, "base64");
        assert_eq!(payload.properties.delivery_info.routing_key, "celery");
    }
}
