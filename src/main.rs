mod coingecko_handler;
mod kaspa_rest_handler;
mod twitter;

use coingecko_handler::CoinGeckoHandler;
use dotenv::dotenv;
use kaspa_rest_handler::RestHandler;
use twitter::TwitterKeys;

use std::{error::Error as StdError, sync::mpsc};

pub type Error = Box<dyn StdError + 'static>;

fn main() -> Result<(), Error> {
    dotenv().ok();
    let consumer_key = parse_env_var("CONSUMER_KEY");
    let consumer_secret = parse_env_var("CONSUMER_SECRET");
    let access_token = parse_env_var("ACCESS_TOKEN");
    let token_secret = parse_env_var("TOKEN_SECRET");
    let whale_factor: u8 = parse_env_var("WHALE_FACTOR").parse()?;

    // let twitter_keys = TwitterKeys::new(consumer_key, consumer_secret, access_token, token_secret);

    // twitter_keys.tweet(message).await;

    let (tx, rx) = mpsc::sync_channel::<Vec<u64>>(10);
    let coingecko_handler = CoinGeckoHandler::handle();

    let kaspa_rest_handler = RestHandler::handle(tx);

    loop {
        let amount_vec = rx.recv().unwrap();
        for amount in amount_vec {
            let kas_amount = (amount / 100000000) as f64;
            let usd_amount = kas_amount * coingecko_handler.get_price();
            println!(
                "amount received: {}, amount in KAS: {}, in USD: {}",
                amount, kas_amount, usd_amount
            );

            let circulation = kaspa_rest_handler.get_circulation();
            let threshold = (whale_factor as f64) / 100.0 * circulation;
            println!("CIRCULATION {}, THRESHOLD {}", circulation, threshold);
            if kas_amount > threshold {
                let message = format!(
                    "Whale Alert!!! a transaction of {} KAS has been detected (more than {}% of circulation!)",
                    kas_amount,
                    whale_factor);
                println!("{}", message);
            }
        }
    }
}

fn parse_env_var(var_name: &str) -> String {
    let err_message = format!("{} must be set.", var_name);
    std::env::var(var_name).expect(&err_message)
}
