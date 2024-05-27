use crate::AppState;

use axum::{
    debug_handler,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use sea_orm::TryIntoModel;
use serde::{Deserialize, Serialize};

use backend::entity::post::Model as Post;

use backend::service::{Mutation as MutationCore, NewPost, Query as QueryCore};

#[derive(Deserialize)]
pub struct Params {
    page: Option<u64>,
    posts_per_page: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct Posts {
    posts: Vec<Post>,
    page: u64,
    num_pages: u64,
}

pub async fn list_posts_by_page(
    state: State<AppState>,
    Query(params): Query<Params>,
) -> (StatusCode, Json<Posts>) {
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(10);
    let (posts, num_pages) = QueryCore::find_posts_in_page(&state.conn, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    let out = Posts {
        posts,
        page,
        num_pages,
    };

    (StatusCode::OK, Json(out))
}

pub async fn list_posts(
    state: State<AppState>,
    req: axum::extract::Request,
) -> (StatusCode, Json<Vec<Post>>) {
    println!("Request: {:?}", req);

    let posts = QueryCore::find_all_posts(&state.conn)
        .await
        .expect("Cannot find all posts");

    (StatusCode::OK, Json(posts))
}

#[debug_handler]
// async fn new_post(state: State<AppState>, Json(post): Json<NewPost>) -> (StatusCode, Json<Post>) {
pub async fn new_post(state: State<AppState>, Json(post): Json<NewPost>) -> impl IntoResponse {
    let new_post = MutationCore::create_post(&state.conn, post)
        .await
        .expect("could not insert post");

    (
        StatusCode::CREATED,
        [
            (header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type"),
            (header::ACCESS_CONTROL_ALLOW_HEADERS, "Accept"),
        ],
        Json(
            new_post
                .try_into_model()
                .expect("Could not convert Post ActiveModel to Model"),
        ),
    )
}

#[debug_handler]
pub async fn edit_post(
    state: State<AppState>,
    Path(id): Path<i32>,
    Json(new_post): Json<NewPost>,
) -> (StatusCode, Json<Post>) {
    // let
    let post = MutationCore::update_post_by_id(&state.conn, id, new_post)
        .await
        .expect("could not edit post");

    (StatusCode::OK, Json(post))
}

#[debug_handler]
pub async fn delete_post(state: State<AppState>, Path(id): Path<i32>) -> (StatusCode, Json<()>) {
    MutationCore::delete_post(&state.conn, id)
        .await
        .expect("could not delete post");

    (StatusCode::OK, Json(()))
}
