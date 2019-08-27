use reqwest::get;
use crate::Result;
use crate::data::{Story,Comment,CommentRoot};

fn download(url: &str) -> Result<String> {
    Ok(get(url)?.text()?)
}

pub fn stories(url: &str) -> Result<Vec<Story>> {
    let body = download(&url)?;
    let stories: Vec<Story> = serde_json::from_str(&body)?;
    Ok(stories)
}

pub fn comments(permalink: &str) -> Result<Vec<Comment>> {
    let url = format!("{}.json", permalink);
    let body = download(&url)?;
    let comment_root: CommentRoot = serde_json::from_str(&body)?;
    Ok(comment_root.comments)
}
