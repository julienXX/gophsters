use structopt::StructOpt;

use std::fs::File;
use std::io::prelude::*;

use hyper::Client;
use hyper::rt::{self, Future, Stream};
use hyper_tls::HttpsConnector;
use url::Url;

mod templates;
//mod fetch;
mod data;
use data::{Comment, CommentRoot, Story, FetchError};

#[derive(Debug, StructOpt)]
#[structopt(name = "gophsters", about = "Generate a gophermap from lobste.rs recent stories")]
struct Cli {
    /// The host to fetch Lobsters articles from
    #[structopt(short = "h", long = "host", default_value = "lobste.rs")]
    host: String,
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
    let gophermap = templates::stories_to_gophermap(&stories);
    f.write_all(&gophermap.as_bytes())?;
    for story in stories {
        build_comments_for(story);
    }
    Ok(())
}

fn build_comments_for(story: Story) {
    let url = format!("{}.json", &story.short_id_url).parse().unwrap();
    let fut = fetch_comments(url)
        .map(|(comments, short_id)| {
            let mut f = File::create(format!("{}.txt", short_id)).unwrap();
            let coms = templates::build_comments_page(&comments, story);
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
