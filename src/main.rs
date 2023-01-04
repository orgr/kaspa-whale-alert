mod coingecko_handler;
mod kaspa_handler;
mod twitter;

use coingecko_handler::CoinGeckoHandler;
use dotenv::dotenv;
use kaspa_handler::KaspaHandler;
use twitter::TwitterKeys;

pub mod proto {
    tonic::include_proto!("protowire");
}

use std::{error::Error as StdError, net::IpAddr, str::FromStr};

pub type Error = Box<dyn StdError + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let consumer_key = parse_env_var("CONSUMER_KEY");
    let consumer_secret = parse_env_var("CONSUMER_SECRET");
    let access_token = parse_env_var("ACCESS_TOKEN");
    let token_secret = parse_env_var("TOKEN_SECRET");
    let mut kaspad_address = parse_env_var("KASPAD_ADDRESS");
    let port = parse_env_var("KASPAD_PORT");
    if !kaspad_address.starts_with("grpc://") {
        IpAddr::from_str(&kaspad_address)?;
        kaspad_address = format!("grpc://{}:{}", kaspad_address, port);
    }
    let whale_factor: u8 = parse_env_var("WHALE_FACTOR").parse()?;

    // let twitter_keys = TwitterKeys::new(consumer_key, consumer_secret, access_token, token_secret);
    // let message = "Whale alert".to_string();
    // twitter_keys.tweet(message).await
    loop {
        let mut coingecko_handler = CoinGeckoHandler::new();
        tokio::spawn(async move { coingecko_handler.listen(whale_factor).await });
        let mut kaspa_handler = KaspaHandler::connect(kaspad_address.clone()).await?;
        kaspa_handler.listen().await?;
    }
    Ok(())
}

fn parse_env_var(var_name: &str) -> String {
    let err_message = format!("{} must be set.", var_name);
    std::env::var(var_name).expect(&err_message)
}
