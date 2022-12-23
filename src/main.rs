mod twitter;

use dotenv::dotenv;
use std::error::Error;
use twitter::TwitterKeys;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let consumer_key = std::env::var("CONSUMER_KEY").expect("CONSUMER_KEY must be set.");
    let consumer_secret = std::env::var("CONSUMER_SECRET").expect("CONSUMER_SECRET must be set.");
    let access_token = std::env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN must be set.");
    let token_secret = std::env::var("TOKEN_SECRET").expect("TOKEN_SECRET must be set.");

    let twitter_keys = TwitterKeys::new(consumer_key, consumer_secret, access_token, token_secret);
    let message = "Whale alert".to_string();
    twitter_keys.tweet(message).await
}
