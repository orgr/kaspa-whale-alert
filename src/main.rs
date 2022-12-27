mod twitter;

use dotenv::dotenv;
use std::error::Error;
use twitter::TwitterKeys;

pub mod proto {
    tonic::include_proto!("protowire");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let consumer_key = parse_env_var("CONSUMER_KEY")?;
    let consumer_secret = parse_env_var("CONSUMER_SECRET")?;
    let access_token = parse_env_var("ACCESS_TOKEN")?;
    let token_secret = parse_env_var("TOKEN_SECRET")?;
    let kaspad_address = parse_env_var("KASPAD_ADDRESS")?;

    let twitter_keys = TwitterKeys::new(consumer_key, consumer_secret, access_token, token_secret);
    let message = "Whale alert".to_string();
    twitter_keys.tweet(message).await
}

fn parse_env_var(var_name: &str) -> Result<String, Box<dyn Error>> {
    let err_message = format!("{} must be set.", var_name);
    let result = std::env::var(var_name).expect(&err_message);
    Ok(result)
}
