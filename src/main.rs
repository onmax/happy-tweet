use anyhow::Result;
use clap::Parser;
use reqwest::header::AUTHORIZATION;
use rust_bert::pipelines::sentiment::{
    Sentiment, SentimentConfig, SentimentModel, SentimentPolarity,
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::File,
    io::prelude::*,
    sync::mpsc,
    thread::{self, JoinHandle},
};
use tokio::{sync::oneshot, task};
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
    /// The output file path. It will be append the results if it exists avoiding duplicates. Output will have a JSON format.
    output: std::path::PathBuf,

    #[clap(short, long)]
    /// Bearer token for the twitter api. Read the docs for more info: https://github.com/onmax/happy-tweet#twitter-bearer-token. You can also set an env variable named `HAPPY_TWEET_BEARER_TOKEN`
    token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    profile_image_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tweet {
    url: String,
    content: String,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HappyTweet {
    user: User,
    tweet: Tweet,
    #[serde(skip_serializing, skip_deserializing)]
    sentiment: Option<Sentiment>,
}

impl PartialEq for HappyTweet {
    fn eq(&self, other: &Self) -> bool {
        self.tweet.url == other.tweet.url
    }
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
    println!("Starting...");
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
    if !res.status().is_success() {
        Err(String::from(
            "🙅 No response when requesting tweets. Check your term.",
        ))?;
    }
    let data = res.json::<TwitterApiResponse>().await?;

    // TODO remove duplicates

    let tweets_string = data
        .data
        .iter()
        .map(|tweet| tweet.text.to_owned())
        .collect::<Vec<String>>();
    let (_handle, classifier) = SentimentClassifier::spawn();
    let sentiments = classifier.predict(tweets_string).await?;

    // convert data to a vector of Tweets
    let mut tweets: Vec<HappyTweet> = Vec::new();
    for (tweet, sentiment) in data.data.iter().zip(sentiments) {
        let user = data
            .includes
            .users
            .iter()
            .find(|u| u.id == tweet.author_id)
            .unwrap();
        let tweet = HappyTweet {
            tweet: Tweet {
                content: tweet.text.clone(),
                url: format!("https://twitter.com/{}/status/{}", user.username, tweet.id),
                created_at: tweet.created_at.clone(),
            },
            user: User {
                username: user.username.to_string(),
                profile_image_url: user.profile_image_url.to_string(),
            },
            sentiment: Some(sentiment),
        };
        tweets.push(tweet);
    }

    // Filter tweets to only keep the "Positive" ones
    let mut tweets = tweets
        .into_iter()
        .filter(|tweet| {
            if let Some(sentiment) = &tweet.sentiment {
                sentiment.polarity == SentimentPolarity::Positive
            } else {
                false
            }
        })
        .collect::<Vec<HappyTweet>>();

    // check if files exists and appends to the array tweets
    let output_path = args.output.as_path();
    if output_path.exists() {
        let mut file = File::open(output_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let existing_tweets: Vec<HappyTweet> = serde_json::from_str(&contents)?;
        for existing_tweet in existing_tweets {
            if !tweets.contains(&existing_tweet) {
                tweets.push(existing_tweet);
            }
        }
    }

    // write results
    let mut file = File::create(output_path)?;
    let json = serde_json::to_string_pretty(&tweets)?;
    file.write_all(json.as_bytes())?;

    println!(
        "\n\n✅  Finish! Retrieved {} tweets. Check {}",
        tweets.len(),
        output_path.display()
    );

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

/// Message type for internal channel, passing around texts and return value
/// senders
type Message = (Vec<String>, oneshot::Sender<Vec<Sentiment>>);

/// Runner for sentiment classification
#[derive(Debug, Clone)]
pub struct SentimentClassifier {
    sender: mpsc::SyncSender<Message>,
}

impl SentimentClassifier {
    /// Spawn a classifier on a separate thread and return a classifier instance
    /// to interact with it
    pub fn spawn() -> (JoinHandle<Result<()>>, SentimentClassifier) {
        let (sender, receiver) = mpsc::sync_channel(100);
        let handle = thread::spawn(move || Self::runner(receiver));
        (handle, SentimentClassifier { sender })
    }

    /// The classification runner itself
    fn runner(receiver: mpsc::Receiver<Message>) -> Result<()> {
        // Needs to be in sync runtime, async doesn't work
        let model = SentimentModel::new(SentimentConfig::default())?;

        while let Ok((texts, sender)) = receiver.recv() {
            let texts: Vec<&str> = texts.iter().map(String::as_str).collect();
            let sentiments = model.predict(texts);
            sender.send(sentiments).expect("sending results");
        }

        Ok(())
    }

    /// Make the runner predict a sample and return the result
    pub async fn predict(&self, texts: Vec<String>) -> Result<Vec<Sentiment>> {
        let (sender, receiver) = oneshot::channel();
        task::block_in_place(|| self.sender.send((texts, sender)))?;
        Ok(receiver.await?)
    }
}
