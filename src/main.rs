// #![deny(warnings)]
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate chrono;
use chrono::prelude::*;

extern crate regex;
use regex::Regex;

extern crate deunicode;
use deunicode::deunicode;

use hyper::Client;
use hyper::rt::{self, Future, Stream};
use hyper_tls::HttpsConnector;

const API_URL: &'static str = "https://lobste.rs/hottest.json";

fn main() {
    let url = API_URL.parse().unwrap();

    let fut = fetch_stories(url)
        .map(|stories| {
            create_finger_response(stories).unwrap();
        })
        .map_err(|e| {
            match e {
                FetchError::Http(e) => eprintln!("http error: {}", e),
                FetchError::Json(e) => eprintln!("json parsing error: {}", e),
            }
        });

    rt::run(fut);
}

fn create_finger_response(stories: Vec<Story>) -> std::io::Result<()> {
    let finger_response = stories_to_finger(stories);
    println!("{}", finger_response);
    Ok(())
}

fn stories_to_finger(stories: Vec<Story>) -> String {
    let mut finger = String::new();
    for story in stories {
        let story_has_url = story.url.is_empty();
        let story_line = if story_has_url {
            format!("\n[{}] - {}\tURL:{}\n", story.score, deunicode(&story.title), story.short_id_url)
        } else {
            let re = Regex::new(r"^https").unwrap();
            let story_url = re.replace_all(&story.url, "http");
            format!("\n[{}] - {}\n{}\n", story.score, deunicode(&story.title), story_url)
        };

        let meta_line = format!("Submitted {} by {} | {}\n", pretty_date(&story.created_at), story.submitter_user.username, story.tags.join(", "));

        finger.push_str(&story_line);
        finger.push_str(&meta_line);
    }
    finger
}

fn pretty_date(date_string: &String) -> String {
    let parsed_date = date_string.parse::<DateTime<Utc>>();
    let date = match parsed_date {
        Ok(date) => date,
        Err(_e)  => Utc::now(),
    };
    date.format("%a %b %e %T %Y").to_string()
}

fn fetch_stories(url: hyper::Uri) -> impl Future<Item=Vec<Story>, Error=FetchError> {
    let https = HttpsConnector::new(4).expect("TLS initialization failed");
    let client = Client::builder()
        .build::<_, hyper::Body>(https);

    client
        .get(url)
        .and_then(|res| {
            res.into_body().concat2()
        })
        .from_err::<FetchError>()
        .and_then(|body| {
            let stories = serde_json::from_slice(&body)?;

            Ok(stories)
        })
        .from_err()
}

#[derive(Deserialize, Debug)]
struct Story {
    title: String,
    created_at: String,
    score: u8,
    comment_count: u8,
    short_id: String,
    short_id_url: String,
    url: String,
    tags: Vec<String>,
    submitter_user: User,
}

#[derive(Deserialize, Debug)]
struct User {
    username: String,
}

enum FetchError {
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
