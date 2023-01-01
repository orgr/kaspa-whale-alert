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

        let mut interval = time::interval(Duration::from_secs(10 * 60));
        loop {
            interval.tick().await;
            self.sync();
        }
    }

    async fn sync(&self) {
        let status_locked = self.status.clone();
        let marketcap: u64;
        let price: u64;

        if let Ok(mut status) = status_locked.lock() {
            status.current_price = price;
            status.threshold = (marketcap / 100 as u64) * self.precent_threshold as u64;
            status.init = true;
        };
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
