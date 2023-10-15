// #![deny(warnings)]
extern crate reqwest;
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

const API_URL: &'static str = "https://lobste.rs/hottest.json";

fn main() {
    let stories = match fetch_stories(API_URL.to_string()) {
        Ok(stories) => create_finger_response(stories).unwrap(),
        Err(e) => panic!(e),
    };
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
            format!("\n[{}] - {}\n", story.score, deunicode(&story.title))
        } else {
            let re = Regex::new(r"^https").unwrap();
            let story_url = re.replace_all(&story.url, "http");
            format!(
                "\n[{}] - {}\n{}\n",
                story.score,
                deunicode(&story.title),
                story_url
            )
        };

        let meta_line = format!(
            "Submitted {} by {} | {}\n",
            pretty_date(&story.created_at),
            story.submitter_user.username,
            story.tags.join(", ")
        );
        let comments_line = format!(
            "View {} comments {}\n",
            story.comment_count, story.short_id_url
        );

        finger.push_str(&story_line);
        finger.push_str(&meta_line);
        finger.push_str(&comments_line);
    }
    finger
}

fn pretty_date(date_string: &String) -> String {
    let parsed_date = date_string.parse::<DateTime<Utc>>();
    let date = match parsed_date {
        Ok(date) => date,
        Err(_e) => Utc::now(),
    };
    date.format("%a %b %e %T %Y").to_string()
}

fn fetch_stories(url: String) -> Result<Vec<Story>, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let mut response = client.get(&url).send()?;

    let stories: Vec<Story> = response.json()?;
    Ok(stories)
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
