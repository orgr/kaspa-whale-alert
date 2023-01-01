# Kaspa Whale Alert
A Twitter bot that watches the Kaspa network and tweets whenever a "large" transaction is encountered.
- A transaction is considered "large" when its amount relatively to Kaspa's marketcap is over the specified precentage (`WHALE_FACOTR`)

## Usage
### Prerequisites
* Kaspad access
* A twitter account with write privilages using OAuth1.0a
### Usage
Add a .env file to project's root dir with the following parameters:
```
CONSUMER_KEY="<twitter consumer key>"
CONSUMER_SECRET="<twitter consumer secret>"
ACCESS_TOKEN="<twitter oauth1a access token>"
TOKEN_SECRET="<twitter oauth1a access token>"
KASPAD_ADDRESS="<kaspad address, for localhost use 127.0.0.1>"
KASPAD_PORT="<kaspad port>"
WHALE_FACTOR="<precentage of market cap to treat a transaction as large>"
```

for example
```
CONSUMER_KEY="xxxxxxxxxxxxxxxxxxxxxxxxx"
CONSUMER_SECRET="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
ACCESS_TOKEN="xxxxxxxxxxxxxxxxxxx-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
TOKEN_SECRET="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
KASPAD_ADDRESS="127.0.0.1"
KASPAD_PORT="16110"
WHALE_FACTOR="10"
```
