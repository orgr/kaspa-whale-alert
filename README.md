# Kaspa Whale Alert
A twitter bot that watches the kaspa network and tweets whenever a large transaction is encountered

# Usage
* kaspad acces is required
Add a .env file to project's root dir with the following parameters:
CONSUMER_KEY="<twitter consumer key>"
CONSUMER_SECRET="<twitter consumer secret>"
ACCESS_TOKEN="<twitter oauth1a access token>"
TOKEN_SECRET="<twitter oauth1a access token>"
KASPAD_ADDRESS="<kaspad address, for localhost use 127.0.0.1>"
KASPAD_PORT="<kaspad port>"
WHALE_FACTOR="<precentage of market cap to treat a transaction as large>"
