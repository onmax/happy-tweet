FROM rust:1.59


COPY ../src/main.rs ./src/
COPY ../Cargo.toml .
RUN cargo build

ENV HAPPY_TWEET_BEARER_TOKEN "YOUR_TOKEN_HERE"

RUN mkdir out
RUN cargo run -- "nimiq -is:retweet" -o ./output.json
