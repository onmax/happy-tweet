FROM rust:1.59

COPY ./src/main.rs ./src/
COPY ./Cargo.toml .
# COPY ./output.json . # This is optional. If you want to append new tweets to an existing file, uncomment this line.

ENV HAPPY_TWEET_BEARER_TOKEN "YOUR_TOKEN_HERE"

RUN cargo run -- "YOUR_SEARCH_TERM_HERE" -o ./output.json
