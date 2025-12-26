use crate::comment::model::Comment;
use crate::utils::error::CustomError;
use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::{Client, Collection};

pub struct CommentService {
    collection: Collection<Comment>,
}

impl CommentService {
    pub fn new(client: &Client) -> Self {
        let collection = client
            .database("rust_blogdb")
            .collection::<Comment>("comments");
        CommentService { collection }
    }

    /// Add a new comment to a post
    pub async fn add_comment(
        &self,
        post_id: ObjectId,
        author_id: ObjectId,
        author_username: Option<String>,
        content: String,
    ) -> Result<ObjectId, CustomError> {
        let comment = Comment {
            id: None,
            post_id,
            author_id,
            author_username,
            content,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = self.collection.insert_one(comment).await.map_err(|e| {
            CustomError::InternalServerError(format!("Failed to add comment: {}", e))
        })?;

        result.inserted_id.as_object_id().ok_or_else(|| {
            CustomError::InternalServerError("Failed to get inserted comment ID".to_string())
        })
    }

    /// Get all comments for a specific post
    pub async fn get_comments_for_post(
        &self,
        post_id: &ObjectId,
    ) -> Result<Vec<Comment>, CustomError> {
        let cursor = self
            .collection
            .find(doc! { "post_id": post_id })
            .await
            .map_err(|e| {
                CustomError::InternalServerError(format!("Failed to fetch comments: {}", e))
            })?;

        let comments: Vec<Comment> = cursor.try_collect().await.map_err(|e| {
            CustomError::InternalServerError(format!("Failed to collect comments: {}", e))
        })?;

        Ok(comments)
    }

    /// Get a single comment by ID
    pub async fn get_comment_by_id(
        &self,
        comment_id: &ObjectId,
    ) -> Result<Option<Comment>, CustomError> {
        self.collection
            .find_one(doc! { "_id": comment_id })
            .await
            .map_err(|e| {
                CustomError::InternalServerError(format!("Failed to fetch comment: {}", e))
            })
    }

    /// Update a comment (only author can update)
    pub async fn update_comment(
        &self,
        comment_id: &ObjectId,
        author_id: &ObjectId,
        content: String,
    ) -> Result<bool, CustomError> {
        let result = self
            .collection
            .update_one(
                doc! { "_id": comment_id, "author_id": author_id },
                doc! {
                    "$set": {
                        "content": content,
                        "updated_at": Utc::now().to_rfc3339()
                    }
                },
            )
            .await
            .map_err(|e| {
                CustomError::InternalServerError(format!("Failed to update comment: {}", e))
            })?;

        if result.matched_count == 0 {
            return Err(CustomError::NotFoundError(
                "Comment not found or not authorized".to_string(),
            ));
        }

        Ok(result.modified_count > 0)
    }

    /// Delete a comment (only author can delete)
    pub async fn delete_comment(
        &self,
        comment_id: &ObjectId,
        author_id: &ObjectId,
    ) -> Result<bool, CustomError> {
        let result = self
            .collection
            .delete_one(doc! { "_id": comment_id, "author_id": author_id })
            .await
            .map_err(|e| {
                CustomError::InternalServerError(format!("Failed to delete comment: {}", e))
            })?;

        if result.deleted_count == 0 {
            return Err(CustomError::NotFoundError(
                "Comment not found or not authorized".to_string(),
            ));
        }

        Ok(true)
    }

    /// Get comment count for a post
    pub async fn get_comment_count(&self, post_id: &ObjectId) -> Result<u64, CustomError> {
        self.collection
            .count_documents(doc! { "post_id": post_id })
            .await
            .map_err(|e| {
                CustomError::InternalServerError(format!("Failed to count comments: {}", e))
            })
    }

    /// Get all comments by a user
    pub async fn get_comments_by_user(
        &self,
        author_id: &ObjectId,
    ) -> Result<Vec<Comment>, CustomError> {
        let cursor = self
            .collection
            .find(doc! { "author_id": author_id })
            .await
            .map_err(|e| {
                CustomError::InternalServerError(format!("Failed to fetch comments: {}", e))
            })?;

        let comments: Vec<Comment> = cursor.try_collect().await.map_err(|e| {
            CustomError::InternalServerError(format!("Failed to collect comments: {}", e))
        })?;

        Ok(comments)
    }
}
