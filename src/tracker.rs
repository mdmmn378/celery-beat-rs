use bson;
use futures::StreamExt;
use mongodb::{Client, Collection};

use crate::task_registry::Payload;

#[allow(dead_code)]
pub struct MongoDB {
    client: Client,
}

#[allow(dead_code)]
impl MongoDB {
    pub async fn new(uri: &str) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        MongoDB { client }
    }

    pub async fn push_task(&self, payload: &Payload) -> Result<(), mongodb::error::Error> {
        let db = self.client.database("celery");
        let collection: mongodb::Collection<Payload> = db.collection("tasks");
        collection.insert_one(payload).await?;
        Ok(())
    }

    pub async fn list_tasks(&self) -> Vec<Payload> {
        let db = self.client.database("celery");
        let collection = db.collection("tasks");
        let filter = bson::doc! {};
        let tasks: mongodb::Cursor<Payload> = collection.find(filter).await.unwrap();
        let mut res_tasks: Vec<Payload> = Vec::new();
        tasks
            .for_each(|task| {
                res_tasks.push(task.unwrap());
                futures::future::ready(())
            })
            .await;
        res_tasks
    }

    pub async fn get_task(&self, task_id: &str) -> Option<Payload> {
        let db = self.client.database("celery");
        let collection = db.collection("tasks");
        let filter = bson::doc! { "headers.id": task_id };
        let task = collection.find_one(filter).await.unwrap();
        match task {
            Some(task) => Some(task),
            None => None,
        }
    }

    pub async fn delete_task(&self, task_id: &str) {
        let db = self.client.database("celery");
        let collection: Collection<Payload> = db.collection("tasks");
        let filter = bson::doc! { "headers.id": task_id };
        collection.delete_one(filter).await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_registry::create_task;
    use serde_json::json;

    #[tokio::test]
    async fn test_push_task() {
        let mongo = MongoDB::new("mongodb://localhost:27017").await;
        let payload = create_task("task", vec![json!(1)], Default::default());
        mongo.push_task(&payload).await.unwrap();
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let mongo = MongoDB::new("mongodb://localhost:27017").await;
        let tasks = mongo.list_tasks().await;
        assert!(tasks.len() > 0);
        println!("{:?}", tasks);
    }

    #[tokio::test]
    async fn test_get_task() {
        let mongo = MongoDB::new("mongodb://localhost:27017").await;
        let tasks = mongo.list_tasks().await;
        let task_id = tasks[0].headers.id.clone();
        let task = mongo.get_task(&task_id).await;
        assert!(task.is_some());
        println!("{:?}", task);
    }

    #[tokio::test]
    async fn test_delete_task() {
        let mongo = MongoDB::new("mongodb://localhost:27017").await;
        let tasks = mongo.list_tasks().await;
        let task_id = tasks[0].headers.id.clone();
        mongo.delete_task(&task_id).await;
        let task = mongo.get_task(&task_id).await;
        assert!(task.is_none());
    }
}
