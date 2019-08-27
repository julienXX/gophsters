
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
