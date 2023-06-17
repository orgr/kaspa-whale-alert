mod coingecko_handler;
mod kaspa_rest_handler;
mod twitter;

use coingecko_handler::CoinGeckoHandler;
use dotenv::dotenv;
use kaspa_rest_handler::{RestHandler, TxInfo};
use log::{debug, error, info};
use std::{error::Error as StdError, sync::mpsc};
use twitter::TwitterKeys;

pub type Error = Box<dyn StdError + 'static>;

fn main() -> Result<(), Error> {
    env_logger::init();
    dotenv().ok();
    let consumer_key = parse_env_var("CONSUMER_KEY");
    let consumer_secret = parse_env_var("CONSUMER_SECRET");
    let access_token = parse_env_var("ACCESS_TOKEN");
    let token_secret = parse_env_var("TOKEN_SECRET");
    let socket_server_url = parse_env_var("WEBSOCKET_URL");
    let whale_factor: f64 = parse_env_var("WHALE_FACTOR").parse()?;

    let twitter_keys = TwitterKeys::new(consumer_key, consumer_secret, access_token, token_secret);

    let (tx_send, tx_recv) = mpsc::sync_channel::<Vec<TxInfo>>(10);
    let (supply_ready_send, supply_ready_recv) = mpsc::sync_channel::<()>(1);
    let coingecko_handler = CoinGeckoHandler::handle();
    let kaspa_rest_handler = RestHandler::handle(tx_send, supply_ready_send, socket_server_url);

    supply_ready_recv.recv().unwrap();
    let mut supply = kaspa_rest_handler.get_circulation();
    assert!(supply != 0.0);
    let mut threshold = get_threshold(whale_factor, supply);
    let startup_message = format!(
        "*beep boop beep*\nwhale watcher is up and running...\n\
                           alerting on transactions larger than {} kas",
        threshold
    );
    info!("{}", startup_message);

    // startup tweet commented
    // twitter_keys.tweet(startup_message.into());

    let mut max_amount = 0.0;
    loop {
        info!("Loop iteration, waiting for tx vector");
        let tx_info_vec = tx_recv.recv().unwrap();

        for tx_info in tx_info_vec {
            let kas_amount = explicit_amount_to_kas_amount(tx_info.amount);
            let usd_amount = kas_amount * coingecko_handler.get_price();

            supply = kaspa_rest_handler.get_circulation();
            threshold = get_threshold(whale_factor, supply);
            if kas_amount > max_amount {
                max_amount = kas_amount;
            }
            debug!(
                "amount received: {}\tamount in KAS: {}\tin USD: {}\t\
                 supply: {}\tthreshold: {}\tmax amount: {}",
                tx_info.amount, kas_amount, usd_amount, supply, threshold, max_amount
            );

            if kas_amount >= threshold {
                max_amount = 0.0;
                let percent_of_supply = (kas_amount / supply) * 100.0;
                let message = gen_message(kas_amount, usd_amount, percent_of_supply, &tx_info.id);
                info!("{}", message);
                twitter_keys.tweet(message);
            }
        }
    }
}

fn get_threshold(whale_factor: f64, supply: f64) -> f64 {
    whale_factor / 100.0 * supply
}

const BLOCK_EXPLORER_TX_ID_URL: &str = "https://explorer.kaspa.org/txs/";
fn get_tx_id_link(tx_id: &str) -> String {
    format!("{}{}", BLOCK_EXPLORER_TX_ID_URL, tx_id)
}

fn parse_env_var(var_name: &str) -> String {
    let err_message = format!("{} must be set.", var_name);
    std::env::var(var_name).expect(&err_message)
}

const EXPLICIT_AMOUNT_IN_KAS_AMOUNT: u64 = 100000000;
fn explicit_amount_to_kas_amount(explicit: u64) -> f64 {
    (explicit / EXPLICIT_AMOUNT_IN_KAS_AMOUNT) as f64
}

const TWEET_CHAR_LIMIT: usize = 280;
use num_format::{Locale, ToFormattedString};
fn gen_message(kas_amount: f64, usd_amount: f64, percent_of_supply: f64, tx_id: &str) -> String {
    let usd_amount_str = (usd_amount.floor() as i64).to_formatted_string(&Locale::en);
    let sponsor_msg = std::env::var("SPONSOR_MESSAGE").unwrap_or("".to_string());
    let message = format!(
        "Whale Alert!!! a tx of {:.2}M KAS (${}) has been detected\n\
                     {:.4}% of current supply\n\
                     {}\n\
                     {}",
        kas_amount / 1000000.0,
        usd_amount_str,
        percent_of_supply,
        get_tx_id_link(tx_id),
        sponsor_msg
    );

    if message.len() > TWEET_CHAR_LIMIT {
        error!("generated tweet is too long {}", message.len());
    }
    return message;
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_tweet_length() {
        dotenv().ok();
        let message = gen_message(
            999999.9,
            2.0,
            50.0,
            "c6c5e3661d97d4b576dbb0a413dc66461dd508118ed612d6c0627d604e4758b7",
        );
        assert_eq!(message.len() < TWEET_CHAR_LIMIT, true);
        println!("{}\n", message);
        println!(
            "{} chars left to reach limit",
            TWEET_CHAR_LIMIT - message.len()
        )
    }
}
