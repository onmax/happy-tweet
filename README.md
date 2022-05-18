# Happy tweet 😊

A tool that fetches happy tweets given a search term

> It will only retrieve tweets less than 7 days old, as searching the full archive is only allowed for academic research. More information in the [official documentation](https://developer.twitter.com/en/docs/twitter-api/tweets/search/introduction).

## Installation

**Remember that you will need a [Twitter Bearer Token](#twitter-bearer-token)**

### Cargo run

`cargo run -- "#banana" -o output.json -t YOUR_TOKEN_HERE`

_`#banana` is the search term, replace it with your search_

### Dockerfile

Modify the Dockerfile and replace:

- [Twitter Bearer Token](#twitter-bearer-token)
- Your search term

```bash
make build #you are suppose to run only once
make run
```

## Advance Search Features

The term accepts a string as follows: `happy-tweet eurovision`. You can filter the results using the advance search features that Twitter offers.

Some examples:

- One hashtag: `happy-tweet "#disney"`
- Ignore retweet `happy-tweet -is:retweet`
- Multiple hashtags: `happy-tweet "(#dc OR #marvel)"`
- Specific language: `happy-tweet "lang:de"`
- Specific author: `happy-tweet "(from:barackobama)"`

And of course you can combine them:

`happy-tweet "paella lang:es"`

`happy-tweet "(happy OR happiness) lang:en -birthday -is:retweet"`

Read official docs on [Advance Search](https://developer.twitter.com/en/docs/twitter-api/tweets/search/integrate/build-a-query).

## Twitter Bearer Token

[How to generate a Bearer Token](https://developer.twitter.com/en/docs/authentication/oauth-2-0/bearer-tokens#:~:text=Login%20to%20your%20Twitter%20account,Bearer%20Token%20on%20this%20page.)

## Output

You can select the output file path using the flag `-o` or `--output`. It will append the new results to the existing file, otherwise it will create the file. The output has a JSON format. By default it will write the output in `/dev/stdout`.
