use clap::Parser;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use url::Url;

static BEARER_ENV_TOKEN_NAME: &str = "HAPPY_TWEET_BEARER_TOKEN";

#[derive(Parser)]
#[clap(
    author = "onmax",
    version,
    about = "A cli tool for fetching happy tweets given a term"
)]
struct Arguments {
    #[clap(forbid_empty_values = true, validator = validate_term_search)]
    /// The term to search for. You can use Twitter's search features like: '@', 'from', 'to', geography locations, etc. More info: https://github.com/onmax/happy-tweet#advance-search-features
    term: String,
    #[clap(short, long, default_value = "/dev/stdout", forbid_empty_values = true, validator = validate_output_path)]
    /// The output file path. By default it's stdout.
    output: std::path::PathBuf,
    #[clap(short, long)]
    /// Bearer token for the twitter api. Read the docs for more info: https://github.com/onmax/happy-tweet#twitter-bearer-token. You can also set an env variable named `HAPPY_TWEET_BEARER_TOKEN`
    token: Option<String>,
}

#[derive(Debug, Serialize)]
struct User {
    username: String,
    profile_image_url: String,
}

#[derive(Debug, Serialize)]
struct Tweet {
    text: String,
    url: String,
    user: User,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TwitterApiResponseData {
    text: String,
    created_at: String,
    author_id: String,
    id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TwitterApiResponseUser {
    id: String,
    username: String,
    name: String,
    profile_image_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TwitterApiResponseIncludes {
    users: Vec<TwitterApiResponseUser>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TwitterApiResponseMeta {
    newest_id: String,
    oldest_id: String,
    result_count: u16,
    next_token: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TwitterApiResponse {
    data: Vec<TwitterApiResponseData>,
    includes: TwitterApiResponseIncludes,
    meta: TwitterApiResponseMeta,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();
    let url = Url::parse_with_params(
        "https://api.twitter.com/2/tweets/search/recent",
        &[
            ("max_results", "100"),
            ("query", &args.term),
            ("tweet.fields", "created_at"),
            ("expansions", "author_id"),
            ("user.fields", "profile_image_url"),
        ],
    )?;

    let bearer = env::var(BEARER_ENV_TOKEN_NAME).unwrap_or_else(|_| args.token.unwrap_or_else(|| {
        panic!("You need to provide a bearer token as an argument or set an env variable named `{}`. Read more: https://github.com/onmax/happy-tweet", BEARER_ENV_TOKEN_NAME);
    }));

    let bearer = if !bearer.starts_with("Bearer ") {
        format!("Bearer {}", bearer)
    } else {
        bearer
    };

    let client = reqwest::Client::builder().build()?;
    let res = client.get(url).header(AUTHORIZATION, bearer).send().await?;
    let data = res.json::<TwitterApiResponse>().await?;

    // convert data to a vector of Tweets
    let mut tweets: Vec<Tweet> = Vec::new();
    for tweet in data.data {
        let user = data
            .includes
            .users
            .iter()
            .find(|u| u.id == tweet.author_id)
            .unwrap();
        let tweet = Tweet {
            text: tweet.text,
            url: format!("https://twitter.com/{}/status/{}", user.username, tweet.id),
            user: User {
                username: user.username.to_string(),
                profile_image_url: user.profile_image_url.to_string(),
            },
        };
        tweets.push(tweet);
    }

    // write results
    let mut file = File::create(args.output)?;
    let json = serde_json::to_string_pretty(&tweets)?;
    file.write_all(json.as_bytes())?;

    println!("\n\n✅  Finish!");

    Ok(())
}

fn validate_term_search(name: &str) -> Result<(), String> {
    if name.is_empty() {
        Err(String::from("The term cannot be empty"))
    } else if name.trim().len() != name.len() {
        Err(String::from(
            "search term name cannot have leading and trailing space",
        ))
    } else {
        Ok(())
    }
}

fn validate_output_path(output: &str) -> Result<(), String> {
    if output.trim().len() != output.len() {
        Err(String::from(
            "output cannot have leading and trailing space",
        ))
    } else {
        Ok(())
    }
}
