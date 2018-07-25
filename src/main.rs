// #![deny(warnings)]
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::fs::File;
use std::io::prelude::*;

use hyper::Client;
use hyper::rt::{self, Future, Stream};
use hyper_tls::HttpsConnector;

const API_URL: &'static str = "https://lobste.rs/newest.json";

fn main() {
    let url = API_URL.parse().unwrap();

    let fut = fetch_json(url)
        // use the parsed vector
        .map(|stories| {
            // print stories
            println!("stories: {:#?}", stories);
            create_gophermap(stories).unwrap();
        })
        // if there was an error print it
        .map_err(|e| {
            match e {
                FetchError::Http(e) => eprintln!("http error: {}", e),
                FetchError::Json(e) => eprintln!("json parsing error: {}", e),
            }
        });

    // Run the runtime with the future trying to fetch, parse and print json.
    //
    // Note that in more complicated use cases, the runtime should probably
    // run on its own, and futures should just be spawned into it.
    rt::run(fut);
}

fn create_gophermap(stories: Vec<Story>) -> std::io::Result<()> {
    let mut f = File::create("gophsters.gophermap")?;
    let gophermap = stories_to_gophermap(stories);
    f.write_all(&gophermap.as_bytes())?;
    Ok(())
}

fn stories_to_gophermap(stories: Vec<Story>) -> String {
    let mut s = String::new();
    for story in stories {
        let story_line = format!("h{}\tURL:{}", story.title, story.short_id_url);
        let comment_line = format!("h[{}] Comments\tURL:{}", story.comment_count, story.comments_url);
        s.push_str(&story_line);
        s.push_str("\n");
        s.push_str(&comment_line);
        s.push_str("\n");
    }
    s
}

fn fetch_json(url: hyper::Uri) -> impl Future<Item=Vec<Story>, Error=FetchError> {
    let https = HttpsConnector::new(4).expect("TLS initialization failed");
    let client = Client::builder()
        .build::<_, hyper::Body>(https);

    client
        // Fetch the url...
        .get(url)
        // And then, if we get a response back...
        .and_then(|res| {
            // asynchronously concatenate chunks of the body
            res.into_body().concat2()
        })
        .from_err::<FetchError>()
        // use the body after concatenation
        .and_then(|body| {
            // try to parse as json with serde_json
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
    short_id_url: String,
    comments_url: String,
}

// Define a type so we can return multiple types of errors
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
