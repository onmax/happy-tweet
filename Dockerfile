FROM rust:1.59

COPY ./src/main.rs ./src/
COPY ./Cargo.toml .

ENV HAPPY_TWEET_BEARER_TOKEN "YOUR_TOKEN_HERE"

RUN cargo run -- "banana" -o ./output.json
