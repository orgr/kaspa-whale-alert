mod coingecko_handler;
mod kaspa_rest_handler;
mod twitter;

use coingecko_handler::CoinGeckoHandler;
use dotenv::dotenv;
use kaspa_rest_handler::RestHandler;
use twitter::TwitterKeys;

use std::{error::Error as StdError, net::IpAddr, str::FromStr};

pub type Error = Box<dyn StdError + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let consumer_key = parse_env_var("CONSUMER_KEY");
    let consumer_secret = parse_env_var("CONSUMER_SECRET");
    let access_token = parse_env_var("ACCESS_TOKEN");
    let token_secret = parse_env_var("TOKEN_SECRET");
    let whale_factor: u8 = parse_env_var("WHALE_FACTOR").parse()?;

    // let twitter_keys = TwitterKeys::new(consumer_key, consumer_secret, access_token, token_secret);
    let message = "Whale alert".to_string();
    // twitter_keys.tweet(message).await;

    let coingecko_handler = CoinGeckoHandler::new();
    coingecko_handler.clone().listen().await;

    let kaspa_rest_handler = RestHandler::new();
    kaspa_rest_handler.clone().listen().await;
    // let rest_handler = R
    // let market_status = coingecko_handler.clone().get_price();
    loop {}
    Ok(())
}

fn parse_env_var(var_name: &str) -> String {
    let err_message = format!("{} must be set.", var_name);
    std::env::var(var_name).expect(&err_message)
}

// TODO
// 1. You have coingecko price, updated every 5 minutes.
// 2. You are requesting for blockdag, you don't need that, there is a websocket room that gives the latest blocks
// -- see it here: https://github.com/lAmeR1/kaspa-rest-server/blob/main/sockets/blocks.py
