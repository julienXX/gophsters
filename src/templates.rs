use chrono::prelude::*;
use regex::Regex;
use textwrap::{fill, indent};
use deunicode::deunicode;

use crate::data::{Comment, Story};

pub fn stories_to_geminimap(stories: &Vec<Story>) -> String {
    let mut geminimap = String::new();
    geminimap.push_str(&main_title());
    for story in stories {
        let story_has_url = story.url.is_empty();
        let story_line = if story_has_url {
            format!("=> {} [{}] - {}\n", story.short_id_url, story.score, deunicode(&story.title))
        } else {
            let story_url = if story.url.starts_with("https") { story.url.replacen("https", "http", 1) } else { story.url.clone() };
            format!("=> {} [{}] - {}\n", story_url, story.score, deunicode(&story.title))
        };

        let meta_line = format!("> Submitted {} by {} | {}\n", pretty_date(&story.created_at), story.submitter_user.username, story.tags.join(", "));
        let comment_line = format!("=> {} View comments ({})\n\n", format!("{}.gmi", &story.short_id), &story.comment_count);

        geminimap.push_str(&story_line);
        geminimap.push_str(&meta_line);
        geminimap.push_str(&comment_line);
    }
    geminimap
}

pub fn build_comments_page(comments: &Vec<Comment>, story: &Story) -> String {
    let mut c = String::new();
    c.push_str(&comment_title(story));
    for comment in comments {
        let meta_line = indent_comment(format!("> {} commented [{}]:\n", comment.commenting_user.username, comment.score), comment.indent_level);
        let comment_line = format!("{}\n", indent_comment(cleanup(&comment.comment), comment.indent_level));
        c.push_str(&meta_line);
        c.push_str(&comment_line);
    }
    c
}

fn indent_comment(string: String, level: u8) -> String {
    match level {
        1 => indent(&fill(&string, 60), ""),
        2 => indent(&fill(&string, 60), "\t"),
        _ => indent(&fill(&string, 60), "\t\t"),
    }
}

fn cleanup(comment: &str) -> String {
    let re = Regex::new(r"<.*?>").unwrap();
    let cleaned: String = deunicode(&comment);
    let result = re.replace_all(&cleaned, "");
    result.to_string()
}

fn main_title() -> String {
    let utc = Utc::now().format("%a %b %e %T %Y").to_string();
    format!("
```
 .-----------------.
| .---------------. |
| |   _________   | |
| |  |___   ___|  | |
| |      | |      | |
| |      | |      | |
| |     _| |_     | |
| |    |_____|    | |
| |               | |
| '---------------' |
 '-----------------'
```
This is an unofficial Tilde.news mirror on gopher.
You can find the 25 hottest stories and their comments.
Sync happens every 10 minutes or so.

Last updated {}

", utc)
}

fn comment_title(story: &Story) -> String {
    format!("
```
 .-----------------.
| .---------------. |
| |   _________   | |
| |  |___   ___|  | |
| |      | |      | |
| |      | |      | |
| |     _| |_     | |
| |    |_____|    | |
| |               | |
| '---------------' |
 '-----------------'
```

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
