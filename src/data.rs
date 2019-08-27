
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Story {
    pub title: String,
    pub created_at: String,
    pub score: i8,
    pub comment_count: u8,
    pub short_id: String,
    pub short_id_url: String,
    pub url: String,
    pub tags: Vec<String>,
    pub submitter_user: User,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub username: String,
}

#[derive(Deserialize, Debug)]
pub struct CommentRoot {
    pub short_id: String,
    pub comments: Vec<Comment>,
}

#[derive(Deserialize, Debug)]
pub struct Comment {
    pub comment: String,
    pub created_at: String,
    pub score: i8,
    pub indent_level: u8,
    pub commenting_user: User,
}

pub enum FetchError {
    Http(hyper::Error),
    Json(serde_json::Error),
}

impl From<hyper::Error> for FetchError {
    fn from(err: hyper::Error) -> FetchError {
        FetchError::Http(err)
    }
}

impl From<serde_json::Error> for FetchError {
    fn from(err: serde_json::Error) -> FetchError {
        FetchError::Json(err)
    }
}
