# find-nearby-twitterer

Find twitter friends living nearby.

## Usage

1. Get Twitter API keys
2. Write following info to `.envrc`

```bash
export TWITTER_API_KEY=XXXXXXXX
export TWITTER_API_KEY_SECRET=XXXXXXXX
export TWITTER_ACCESS_TOKEN=XXXXXXXX
export TWITTER_ACCESS_TOKEN_SECRET=XXXXXXXX
```

3. Run `cargo run followers YOUR_TWITTER_ACCOUNT_NAME` to fetch and save your followers to `followers.json`
4. Run `cargo run following YOUR_TWITTER_ACCOUNT_NAME` to fetch and save your followees to `following.json`
5. Run `cargo run list followers.json following.json -p kyoto` to find twitter friends whose location info include "kyoto"


## License

MIT


## Thanks

- https://qiita.com/hppRC/items/05a81b56d12d663d03e0
- https://zenn.dev/bin_zsh/books/83ef64248646f18d246a
