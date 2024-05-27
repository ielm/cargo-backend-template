use crate::entity::{post, post::Entity as Post};
use sea_orm::*;
use serde::{Deserialize, Serialize};

pub struct Mutation;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewPost {
    pub title: String,
    pub text: String,
}

impl NewPost {
    pub fn into_active_model(self) -> post::ActiveModel {
        post::ActiveModel {
            title: sea_orm::ActiveValue::Set(self.title),
            text: sea_orm::ActiveValue::Set(self.text),
            ..Default::default()
        }
    }

    pub fn into_active_model_with_id(self, id: i32) -> post::ActiveModel {
        post::ActiveModel {
            id: sea_orm::ActiveValue::Set(id),
            title: sea_orm::ActiveValue::Set(self.title),
            text: sea_orm::ActiveValue::Set(self.text),
        }
    }
}

impl Mutation {
    pub async fn create_post(db: &DbConn, form_data: NewPost) -> Result<post::ActiveModel, DbErr> {
        form_data.into_active_model().save(db).await
    }

    pub async fn update_post_by_id(
        db: &DbConn,
        id: i32,
        form_data: NewPost,
    ) -> Result<post::Model, DbErr> {
        let post: post::ActiveModel = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        post::ActiveModel {
            id: post.id,
            title: Set(form_data.title.to_owned()),
            text: Set(form_data.text.to_owned()),
        }
        .update(db)
        .await
    }

    pub async fn delete_post(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let post: post::ActiveModel = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        post.delete(db).await
    }

    pub async fn delete_all_posts(db: &DbConn) -> Result<DeleteResult, DbErr> {
        Post::delete_many().exec(db).await
    }
}
