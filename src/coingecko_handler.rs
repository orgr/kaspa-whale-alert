use crate::Error;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};

pub struct CoinGeckoHandler {
    status: Arc<Mutex<MarketStatus>>,
    precent_threshold: u32,
}

struct MarketStatus {
    threshold: u32,
    priced_threshold: u64,
    init: bool,
}

impl MarketStatus {
    pub fn new() -> Self {
        Self {
            init: false,
            threshold: 0,
            priced_threshold: 0,
        }
    }
}

const coingecko_request_url: &str = "https://api.coingecko.com/api/v3/simple/price?ids=kaspa&vs_currencies=usd&include_market_cap=true";

// {\"kaspa\":{\"usd\":0.00499719,\"usd_market_cap\":78044992.34680438}}
#[derive(Debug, Deserialize)]
struct CoingeckoResponse {
    kaspa: CoingeckoResponseCoin,
}

#[derive(Debug, Deserialize)]
struct CoingeckoResponseCoin {
    usd: f64,
    usd_market_cap: f64,
}

impl CoinGeckoHandler {
    pub fn new() -> Self {
        let status = Arc::new(Mutex::new(MarketStatus::new()));
        Self {
            status,
            precent_threshold: 0,
        }
    }

    pub async fn listen(&mut self, precent_threshold: u8) {
        println!("coinmarketcap started sync");

        self.precent_threshold = precent_threshold.into();
        // let mut interval = time::interval(Duration::from_secs(10 * 60));
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            println!("about to sync!");
            match self.sync().await {
                Err(e) => println!("{:?}", e),
                Ok(_) => println!("it went ok"),
            }
            println!("finished sync!");
        }
    }

    async fn sync(&self) -> Result<(), Error> {
        let status_locked = self.status.clone();
        let mut marketcap: f64;
        let mut price: f64;

        let response = reqwest::get(coingecko_request_url).await?.text().await?;
        println!("response {}", response);
        let parsed_response: CoingeckoResponse = serde_json::from_str(&response)?;
        (price, marketcap) = (
            parsed_response.kaspa.usd,
            parsed_response.kaspa.usd_market_cap,
        );

        self.update(marketcap, price);
        Ok(())
    }

    fn update(&self, marketcap: f64, price: f64) {
        let threshold = (marketcap * (self.precent_threshold as f64 * 0.01));
        let priced_threshold = (threshold / price) as u64;
        let status_locked = self.status.clone();
        if let Ok(mut status) = status_locked.lock() {
            status.threshold = threshold as u32;
            status.priced_threshold = priced_threshold;
            status.init = true;
        };

        println!(
            "new threshold: {}, new priced thresh: {}, precent_thresh: {},",
            threshold, priced_threshold, self.precent_threshold
        );
    }

    pub async fn is_amount_greater_than_threshold(&self, amount: u64) -> Result<bool, Error> {
        let mut priced_threshold: u64 = 0;
        let status = self.status.clone();
        if let Ok(status_unlocked) = status.lock() {
            if !status_unlocked.init {
                return Err("not synced with coingecko".into());
            }
            priced_threshold = status_unlocked.priced_threshold;
        }
        Ok(amount >= priced_threshold)
    }
}
