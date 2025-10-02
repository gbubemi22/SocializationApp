use crate::post::post_model::Post;
use crate::utils::error::CustomError;
use chrono::Utc;
use mongodb::{
    Client, Collection,
    bson::{doc, oid::ObjectId},
};

pub struct PostService {
    collection: Collection<Post>,
}

impl PostService {
    pub fn new(client: &Client) -> Self {
        let collection = client.database("rust_blogdb").collection::<Post>("posts");
        PostService { collection }
    }

    // ✅ Add &self parameter and use self.collection
    pub async fn create_post(&self, post: Post) -> Result<Post, CustomError> {
        self.collection
            .insert_one(&post)
            .await
            .map_err(|_| CustomError::InternalServerError("Failed to create post".into()))?;

        Ok(post)
    }

    // ✅ Add &self parameter
    pub async fn get_post(&self, id: &str) -> Result<Option<Post>, CustomError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| CustomError::BadRequestError("Invalid post ID".into()))?;

        self.collection
            .find_one(doc! { "_id": object_id })
            .await
            .map_err(|_| CustomError::InternalServerError("Failed to fetch post".into()))
    }

    // ✅ Add &self parameter
    pub async fn delete_post(&self, id: &str) -> Result<bool, CustomError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| CustomError::BadRequestError("Invalid post ID".into()))?;

        let result = self
            .collection
            .delete_one(doc! { "_id": object_id })
            .await
            .map_err(|_| CustomError::InternalServerError("Failed to delete post".into()))?;

        Ok(result.deleted_count > 0)
    }

    // ✅ Add &self parameter
    pub async fn update_post(
        &self,
        id: &str,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<Option<Post>, CustomError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| CustomError::BadRequestError("Invalid post ID".into()))?;

        let mut update_doc = doc! {
            "$set": {
                "updated_at": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis())
            }
        };

        if let Some(t) = title {
            update_doc
                .get_document_mut("$set")
                .unwrap()
                .insert("title", t);
        }
        if let Some(c) = content {
            update_doc
                .get_document_mut("$set")
                .unwrap()
                .insert("content", c);
        }

        let updated_post = self
            .collection
            .find_one_and_update(doc! { "_id": object_id }, update_doc)
            .await
            .map_err(|_| CustomError::InternalServerError("Failed to update post".into()))?;

        Ok(updated_post)
    }
}
