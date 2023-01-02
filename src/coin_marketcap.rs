use crate::Error;
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};

pub struct CoinMarketcapHandler {
    status: Arc<Mutex<MarketcapStatus>>,
    precent_threshold: u32,
}

struct MarketcapStatus {
    current_price: u64,
    threshold: u64,
    init: bool,
}

impl MarketcapStatus {
    pub fn new() -> Self {
        Self {
            init: false,
            current_price: 0,
            threshold: 0,
        }
    }
}

const coinmarketcap_request_url: &str = "https://api.coingecko.com/api/v3/simple/price?ids=kaspa&vs_currencies=usd&include_market_cap=true";
impl CoinMarketcapHandler {
    pub fn new() -> Self {
        let status = Arc::new(Mutex::new(MarketcapStatus::new()));
        Self {
            status,
            precent_threshold: 0,
        }
    }

    pub async fn listen(&self, precent_threshold: u32) {
        println!("coinmarketcap started sync");

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
        let marketcap: u64;
        let price: u64;

        let response = reqwest::get(coinmarketcap_request_url)
            .await?
            .text()
            .await?;

        println!("==============================={:?}", response);
        // if let Ok(mut status) = status_locked.lock() {
        //     status.current_price = price;
        //     status.threshold = (marketcap / 100 as u64) * self.precent_threshold as u64;
        //     status.init = true;
        // };
        Ok(())
    }

    pub async fn is_amount_greater_than_precent_of_marketcap(
        &self,
        amount: u64,
    ) -> Result<bool, Error> {
        let (mut price, mut threshold) = (0, 0);
        let status = self.status.clone();
        if let Ok(status_unlocked) = status.lock() {
            (price, threshold) = (status_unlocked.current_price, status_unlocked.threshold);
        }
        let usd_val = amount * price;
        Ok(usd_val >= threshold)
    }
}
