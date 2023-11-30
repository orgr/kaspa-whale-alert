mod coingecko_handler;
mod kaspa_rest_handler;
mod twitter;

use coingecko_handler::CoinGeckoHandler;
use dotenv::dotenv;
use kaspa_rest_handler::{OutputCandidate, RestHandler};
use log::{debug, error, info, warn};
use std::{collections::HashMap, error::Error as StdError, fmt::Write, sync::mpsc};
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

    let (tx_send, tx_recv) = mpsc::sync_channel::<Vec<OutputCandidate>>(10);
    let coingecko_handler = CoinGeckoHandler::handle();
    let kaspa_rest_handler = RestHandler::handle(tx_send, socket_server_url, whale_factor);

    let startup_message = format!("*beep boop beep*\nwhale watcher is up and running...");
    info!("{}", startup_message);

    // startup tweet commented
    // twitter_keys.tweet(startup_message.into());

    let exchange_addrs = HashMap::from([
        (
            "kaspa:qqetp7ct8kqss99fxmymyz5t3fezppxp0t58wl6pawp27elqd46uudme00cl0",
            "MEXC",
        ),
        (
            "kaspa:qq3k4du6wf2g26j7ds6fqmgtgavgm3zy676wntp2e52nsuns2n4s6xkndmx0y",
            "KuCoin",
        ),
        (
            "kaspa:qrelgny7sr3vahq69yykxx36m65gvmhryxrlwngfzgu8xkdslum2yxjp3ap8m",
            "Gate.io",
        ),
        (
            "kaspa:qzkz8vgqqzjq2uc75hcw7rnj8c76mu2c39lg9ca0t3tcv282fm94jyadynj6j",
            "CoinEx",
        ),
        (
            "kaspa:qp482t64mnfy6rs7hdk5rp9ezrcewangv3hfcsm9qye7rv37rrwkss0mxrev3",
            "Txbit",
        ),
        (
            "kaspa:qqywx2wszmnrsu0mzgav85rdwvzangfpdj9j3ady9jpr7hu4u8c2wl9wqgd6j",
            "Bitget",
        ),
        (
            "kaspa:pqtm55d2a456qws90g096cxnecc7msjmxr8n2ernwues8zfdamkl2kfmxr8gr",
            "Rust Fund",
        ),
        (
            "kaspa:ppk66xua7nmq8elv3eglfet0xxcfuks835xdgsm5jlymjhazyu6h5ac62l4ey",
            "DAGKnight Protocol Fund",
        ),
    ]);

    loop {
        info!("Loop iteration, waiting for whale output candidates vector");
        let output_candidate_vec = tx_recv.recv().unwrap();

        for opc in output_candidate_vec {
            let kas_amount = opc.amount;
            let usd_amount = kas_amount * coingecko_handler.get_price();

            debug!(
                "amount received: {} amount in KAS: {}\tin USD: {}",
                opc.amount, kas_amount, usd_amount
            );

            check_whale_tx(
                kas_amount,
                usd_amount,
                opc.idx,
                &opc.id,
                &exchange_addrs,
                &twitter_keys,
                &kaspa_rest_handler,
            );
        }
    }
}

fn check_whale_tx(
    kas_amount: f64,
    usd_amount: f64,
    output_idx: usize,
    tx_id: &str,
    exchange_addrs: &HashMap<&str, &str>,
    twitter_keys: &TwitterKeys,
    kaspa_rest_handler: &RestHandler,
) {
    let tx_extra_info: TxExtraInfo;
    match kaspa_rest_handler.get_tx_extra_info(tx_id) {
        Ok(extra) => tx_extra_info = extra,
        Err(e) => {
            warn!("{}", e.to_string());
            return;
        }
    }
    if !tx_extra_info.is_accepted {
        info!("tx not accepted yet, {}", tx_id);
        // TODO uncomment next line
        // return;
    }
    let output = &tx_extra_info.outputs[output_idx];

    let mut from_exchange_opt = None;
    for input in tx_extra_info.inputs {
        if input.previous_outpoint_address == output.script_public_key_address {
            info!(
                "self transfer found in tx: {}, from {} to itself",
                tx_id, output.script_public_key_address
            );
            return;
        }

        if exchange_addrs.contains_key(input.previous_outpoint_address.as_str()) {
            let addr = input.previous_outpoint_address.as_str();
            from_exchange_opt = Some(exchange_addrs[addr]);
        }
    }

    let mut to_exchange_opt = None;
    let output_addr = output.script_public_key_address.as_str();

    if exchange_addrs.contains_key(output_addr) {
        to_exchange_opt = Some(exchange_addrs[output_addr]);
    }

    let supply = kaspa_rest_handler.get_circulation();
    let percent_of_supply = (kas_amount / supply) * 100.0;

    let message = gen_message(
        kas_amount,
        usd_amount,
        percent_of_supply,
        tx_id,
        from_exchange_opt,
        to_exchange_opt,
    );
    info!("{}", message);
    twitter_keys.tweet(message);
}

const BLOCK_EXPLORER_TX_ID_URL: &str = "https://explorer.kaspa.org/txs/";
fn get_tx_id_link(tx_id: &str) -> String {
    format!("{}{}", BLOCK_EXPLORER_TX_ID_URL, tx_id)
}

fn parse_env_var(var_name: &str) -> String {
    let err_message = format!("{} must be set.", var_name);
    std::env::var(var_name).expect(&err_message)
}

const TWEET_CHAR_LIMIT: usize = 280;
use num_format::{Locale, ToFormattedString};

use crate::kaspa_rest_handler::TxExtraInfo;
fn gen_message(
    kas_amount: f64,
    usd_amount: f64,
    percent_of_supply: f64,
    tx_id: &str,
    from_exchange_opt: Option<&str>,
    to_exchange_opt: Option<&str>,
) -> String {
    let mut message = String::new();
    let usd_amount_str = (usd_amount.floor() as i64).to_formatted_string(&Locale::en);
    write!(
        &mut message,
        "Whale Alert!!! a tx of {:.2}M $kas (${}) has been detected\n\
         {:.4}% of current supply\n{}",
        kas_amount / 1000000.0,
        usd_amount_str,
        percent_of_supply,
        get_tx_id_link(tx_id)
    )
    .unwrap();
    if let Some(from_exchange) = from_exchange_opt {
        write!(&mut message, "\nfrom {}", from_exchange).unwrap();
    }
    if let Some(to_exchange) = to_exchange_opt {
        write!(&mut message, "\nto {}", to_exchange).unwrap();
    }
    if let Ok(sponsor_msg) = std::env::var("SPONSOR_MESSAGE") {
        write!(&mut message, "\n{}", sponsor_msg).unwrap();
    }

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
            Some("MEXC"),
            Some("KuCoin"),
        );
        assert_eq!(message.len() < TWEET_CHAR_LIMIT, true);
        println!("{}\n", message);
        println!(
            "{} chars left to reach limit",
            TWEET_CHAR_LIMIT - message.len()
        )
    }
}
