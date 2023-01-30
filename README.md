# Kaspa Whale Alert
A Twitter bot that watches the Kaspa network and tweets whenever a "large" transaction is encountered.
- A transaction is considered "large" when its amount relatively to Kaspa's marketcap is over the specified precentage (`WHALE_FACTOR`)

## Usage
### Prerequisites
* A twitter account with write privilages using OAuth1.0a
### Usage
Use the following environment variables or specify them in a `.env` file in project's root dir:
```bash
CONSUMER_KEY="<twitter consumer key>"
CONSUMER_SECRET="<twitter consumer secret>"
ACCESS_TOKEN="<twitter oauth1a access token>"
TOKEN_SECRET="<twitter oauth1a access token>"
WHALE_FACTOR="<precentage of market cap to treat a transaction as large>"
```

for example
```bash
CONSUMER_KEY="xxxxxxxxxxxxxxxxxxxxxxxxx"
CONSUMER_SECRET="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
ACCESS_TOKEN="xxxxxxxxxxxxxxxxxxx-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
TOKEN_SECRET="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
WHALE_FACTOR="0.01"
RUST_LOG="info"
```
### Logs
For logs I used the `env_logger` crate, so the default log level is `error`.
To specify another level use the `RUST_LOG` environment variable.
