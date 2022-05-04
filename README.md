# Happy tweet 😊

A tool that fetches happy tweets given a search term

> It will only retrieve tweets less than 7 days old, as searching the full archive is only allowed for academic research. More information in the [official documentation](https://developer.twitter.com/en/docs/twitter-api/tweets/search/introduction).

## Installation

TODO


## Advance Search Features

The term accepts a string as follows: `happy-tweet eurovision`. You can filter the results using the advance search features that Twitter offers.

Some examples:

- One hashtag: `happy-tweet "#disney"`
- Multiple hashtags: `happy-tweet "(#dc OR #marvel)"`
- Specific language: `happy-tweet "lang:de"`
- Specific author: `happy-tweet "(from:barackobama)"`
- Min replies: `happy-tweet "min_replies:30"`

And of course you can combine them:

`happy-tweet "paella lang:es min_replies:2"`



Read official docs on [Advance Search](https://help.twitter.com/en/using-twitter/twitter-advanced-search).

## Twitter Bearer Token

TODO

## Output

You can select the output file path using the flag `-o` or `--output`. It will overwrite if it exists. The output has a JSON format. By default it will write the output in `/dev/stdout`.


