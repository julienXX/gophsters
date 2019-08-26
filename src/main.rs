use chrono::prelude::*;
use regex::Regex;
use textwrap::{fill, indent};
use deunicode::deunicode;
use serde::Deserialize;
use structopt::StructOpt;

use std::fs::File;
use std::io::prelude::*;

use hyper::Client;
use hyper::rt::{self, Future, Stream};
use hyper_tls::HttpsConnector;
use url::Url;

#[derive(Debug, StructOpt)]
#[structopt(name = "gophsters", about = "Generate a gophermap from lobste.rs recent stories")]
struct Cli {
    /// The host to fetch Lobsters articles from
    #[structopt(short = "h", long = "host", default_value = "lobste.rs")]
    host: String,
    // The folder 
}

fn main() {
    let cli = Cli::from_args();

    let host = if cli.host.starts_with("http") { cli.host } else { format!("https://{}", cli.host) };

    let base_url = Url::parse(&host).expect("Could not parse hostname");
    // join() doesn't care about a trailing slash passed as host
    let url = base_url.join("hottest.json").unwrap().as_str().parse().unwrap();

    let fut = fetch_stories(url)
        .map(|stories| {
            create_gophermap(stories).unwrap();
        })
        .map_err(|e| {
            match e {
                FetchError::Http(e) => eprintln!("http error: {}", e),
                FetchError::Json(e) => eprintln!("json parsing error: {}", e),
            }
        });

    rt::run(fut);

    println!("Done.")
}

fn create_gophermap(stories: Vec<Story>) -> std::io::Result<()> {
    let mut f = File::create("gophermap")?;
    let gophermap = stories_to_gophermap(stories);
    f.write_all(&gophermap.as_bytes())?;
    Ok(())
}

fn termination_line() -> String {
    "\r\n.".to_owned()
}

fn stories_to_gophermap(stories: Vec<Story>) -> String {
    let mut gophermap = String::new();
    gophermap.push_str(&main_title());
    for story in stories {
        println!("Building story: {}", story.title);

        let story_has_url = story.url.is_empty();
        let story_line = if story_has_url {
            format!("h[{}] - {}\tURL:{}\n", story.score, deunicode(&story.title), story.short_id_url)
        } else {
            let story_url = if story.url.starts_with("https") { story.url.replacen("https", "http", 1) } else { story.url.clone() };
            format!("h[{}] - {}\tURL:{}\n", story.score, deunicode(&story.title), story_url)
        };

        let meta_line = format!("Submitted {} by {} | {}\n", pretty_date(&story.created_at), story.submitter_user.username, story.tags.join(", "));
        let comment_line = format!("0View comments ({})\t{}\n\n", &story.comment_count, format!("{}.txt", &story.short_id));
        build_comments_for(story);

        gophermap.push_str(&story_line);
        gophermap.push_str(&meta_line);
        gophermap.push_str(&comment_line);
    }
    gophermap.push_str(&termination_line());
    gophermap
}

fn build_comments_for(story: Story) {
    let url = format!("{}.json", &story.short_id_url).parse().unwrap();
    let fut = fetch_comments(url)
        .map(|(comments, short_id)| {
            let mut f = File::create(format!("{}.txt", short_id)).unwrap();
            let coms = build_comments_page(comments, story);
            f.write_all(&coms.as_bytes()).expect("could not write file");
        })
        .map_err(|e| {
            match e {
                FetchError::Http(e) => eprintln!("http error: {}", e),
                FetchError::Json(e) => eprintln!("json parsing error: {}", e),
            }
        });

    rt::run(fut);
}

fn build_comments_page(comments: Vec<Comment>, story: Story) -> String {
    let mut c = String::new();
    c.push_str(&comment_title(story));
    for comment in comments {
        let meta_line = indent_comment(format!("> {} commented [{}]:\n", comment.commenting_user.username, comment.score), comment.indent_level);
        let comment_line = format!("{}\n", indent_comment(cleanup(comment.comment), comment.indent_level));
        c.push_str(&meta_line);
        c.push_str(&comment_line);
    }
    c.push_str(&termination_line());
    c
}

fn indent_comment(string: String, level: u8) -> String {
    match level {
        1 => indent(&fill(&string, 60), ""),
        2 => indent(&fill(&string, 60), "\t"),
        _ => indent(&fill(&string, 60), "\t\t"),
    }
}

fn cleanup(comment: String) -> String {
    let re = Regex::new(r"<.*?>").unwrap();
    let cleaned: String = deunicode(&comment);
    let result = re.replace_all(&cleaned, "");
    result.to_string()
}

fn main_title() -> String {
    let utc = Utc::now().format("%a %b %e %T %Y").to_string();
    format!("
 .----------------.
| .--------------. |
| |   _____      | |
| |  |_   _|     | |
| |    | |       | |
| |    | |   _   | |
| |   _| |__/ |  | |
| |  |________|  | |
| |              | |
| '--------------' |
 '----------------'

This is an unofficial Lobste.rs mirror on gopher.
You can find the 25 hottest stories and their comments.
Sync happens every 10 minutes or so.

Last updated {}

", utc)
}

fn comment_title(story: Story) -> String {
    format!("
 .----------------.
| .--------------. |
| |   _____      | |
| |  |_   _|     | |
| |    | |       | |
| |    | |   _   | |
| |   _| |__/ |  | |
| |  |________|  | |
| |              | |
| '--------------' |
 '----------------'


Viewing comments for \"{}\"
---

", deunicode(&story.title))
}

fn pretty_date(date_string: &str) -> String {
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

fn fetch_comments(url: hyper::Uri) -> impl Future<Item=(Vec<Comment>, String), Error=FetchError> {
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
            let body_string = std::str::from_utf8(&body).unwrap();
            let json_body: CommentRoot = serde_json::from_str(&body_string)?;
            let comments = json_body.comments;
            Ok((comments, json_body.short_id))
        })
        .from_err()
}

#[derive(Deserialize, Debug)]
struct Story {
    title: String,
    created_at: String,
    score: i8,
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

#[derive(Deserialize, Debug)]
struct CommentRoot {
    short_id: String,
    comments: Vec<Comment>,
}

#[derive(Deserialize, Debug)]
struct Comment {
    comment: String,
    created_at: String,
    score: i8,
    indent_level: u8,
    commenting_user: User,
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
