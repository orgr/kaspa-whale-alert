mod coingecko_handler;
mod kaspa_rest_handler;
mod twitter;

use coingecko_handler::CoinGeckoHandler;
use dotenv::dotenv;
use kaspa_rest_handler::{RestHandler, TxInfo};
use log::{debug, info, warn};
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
    let whale_factor: f64 = parse_env_var("WHALE_FACTOR").parse()?;

    let twitter_keys = TwitterKeys::new(consumer_key, consumer_secret, access_token, token_secret);

    let (tx_send, tx_recv) = mpsc::sync_channel::<Vec<TxInfo>>(10);
    let (supply_ready_send, supply_ready_recv) = mpsc::sync_channel::<()>(1);
    let coingecko_handler = CoinGeckoHandler::handle();
    let kaspa_rest_handler = RestHandler::handle(tx_send, supply_ready_send);

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
        info!("waiting for tx vector");
        let tx_info_vec = tx_recv.recv().unwrap();
        if tx_info_vec.len() == 0 {
            warn!("received empty tx vector from kaspa handler");
        }

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
                let message = format!(
                    "Whale Alert!!! a transaction of {:.2}M KAS ({:.2}$) has been detected \n\
                     {:.4}% of current supply \n\
                     {}",
                    kas_amount / 1000000.0,
                    usd_amount,
                    percent_of_supply,
                    get_tx_id_link(&tx_info.id)
                );
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
