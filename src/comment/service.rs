use crate::errors::CustomError;
use crate::models::{Comment};
use chrono::Utc;
use futures::stream::TryStreamExt;
use mongodb::{
    Collection,
    bson::{doc, oid::ObjectId},
};

pub async fn add_comment(
    collection: &Collection<Comment>,
    post_id: ObjectId,
    author_id: ObjectId,
    content: String,
) -> Result<Comment, CustomError> {
    let comment = Comment {
        id: ObjectId::new(),
        post_id,
        author_id,
        content,
        created_at: Utc::now(),
    };

    collection
        .insert_one(&comment, None)
        .await
        .map_err(|_| CustomError::InternalServerError("Failed to add comment".into()))?;

    Ok(comment)
}

pub async fn get_comments_for_post(
    collection: &Collection<Comment>,
    post_id: &ObjectId,
) -> Result<Vec<Comment>, CustomError> {
    let mut cursor = collection
        .find(doc! { "post_id": post_id }, None)
        .await
        .map_err(|_| CustomError::InternalServerError("Failed to fetch comments".into()))?;

    let mut comments = Vec::new();
    while let Some(comment) = cursor.try_next().await.unwrap_or(None) {
        comments.push(comment);
    }
    Ok(comments)
}
