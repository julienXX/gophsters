use structopt::StructOpt;

use std::fs::File;
use std::io::prelude::*;

use url::Url;

// Used for asynchronous iteration over stories
// i.e. parallelized blocking network IO via rayon
use rayon::prelude::*;

mod templates;
mod fetch;
mod data;
use data::Story;

// For simple automagic error handling
use error_chain::error_chain;

error_chain!{
    foreign_links {
        Http(reqwest::Error);
        Json(serde_json::Error);
        Io(std::io::Error);
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "gophsters", about = "Generate a gemini file from lobste.rs recent stories")]
struct Cli {
    /// The host to fetch Lobsters articles from
    #[structopt(short = "h", long = "host", default_value = "lobste.rs")]
    host: String,
}

fn main() -> Result<()> {
    let cli = Cli::from_args();

    let host = if cli.host.starts_with("http") { cli.host } else { format!("https://{}", cli.host) };

    let base_url = Url::parse(&host).expect("Could not parse hostname");
    // join() doesn't care about a trailing slash passed as host
    let url = base_url.join("hottest.json").unwrap();

    let stories = fetch::stories(&url.as_str())?;

    match create_geminimap(&stories) {
        Ok(_) => {
            println!("Built geminimap for server {}.", &host);
        },
        Err(e) => {
            eprintln!("Failed to build geminimap for server {} because of error:", &host);
            return Err(e);
        }
    }

    // Configure rayon to use maximum 4 threads (so we don't get blocked by the lobsters API)
    rayon::ThreadPoolBuilder::new().num_threads(4).build_global().unwrap();

    // Sweet, sweet rayon for parellel processing
    stories.par_iter().for_each(|story| {
        match build_comments_for(&story) {
            Ok(_) => {
                println!("Built comments for page {}", &story.title);
            },
            Err(e) => {
                eprintln!("Failed to build comments for page {} because of error:\n{}", &story.title, e);
            }
        }
    });

    println!("Done.");
    Ok(())
}

fn create_geminimap(stories: &Vec<Story>) -> Result<()> {
    let mut f = File::create("index.gmi")?;
    let geminimap = templates::stories_to_geminimap(&stories);
    f.write_all(&geminimap.as_bytes())?;
    Ok(())
}

fn build_comments_for(story: &Story) -> Result<()> {
    let comments = fetch::comments(&story.short_id_url)?;
    let mut f = File::create(format!("{}.gmi", story.short_id))?;
    let coms = templates::build_comments_page(&comments, story);
    f.write_all(&coms.as_bytes())?;
    Ok(())
}
