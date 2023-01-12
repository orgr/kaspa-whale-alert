use crate::Error;
use serde::Deserialize;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub struct CoinGeckoHandler {
    price: Mutex<f64>,
}

const COINGECKO_REQUEST_URL: &str =
    "https://api.coingecko.com/api/v3/simple/price?ids=kaspa&vs_currencies=usd";
const POLL_INTERVAL_SEC: u64 = 5;

#[derive(Debug, Deserialize)]
struct CoingeckoResponse {
    kaspa: CoingeckoResponseCoin,
}

#[derive(Debug, Deserialize)]
struct CoingeckoResponseCoin {
    usd: f64,
}

impl CoinGeckoHandler {
    pub fn new() -> Arc<Self> {
        let price = Mutex::new(0.0);
        Arc::new(Self { price })
    }

    pub fn listen(self: Arc<Self>) {
        let arc = self.clone();
        thread::spawn(move || {
            println!("coinmarketcap started sync");
            loop {
                println!("about to sync!");
                match arc.sync() {
                    Err(e) => println!("{:?}", e),
                    Ok(_) => println!("it went ok"),
                }
                println!("finished sync!");
                thread::sleep(Duration::from_secs(POLL_INTERVAL_SEC));
            }
        });
    }

    fn sync(&self) -> Result<(), Error> {
        let price: f64;

        let response: CoingeckoResponse = reqwest::blocking::get(COINGECKO_REQUEST_URL)?.json()?;
        println!("response {:?}", response);
        self.update(response.kaspa.usd);
        Ok(())
    }

    fn update(&self, price: f64) {
        if let Ok(mut price_unlocked) = self.price.lock() {
            *price_unlocked = price;
        };

        println!("new price: {}", price);
    }

    pub fn get_price(self: Arc<Self>) -> f64 {
        if let Ok(price) = self.price.lock() {
            return *price;
        };
        0.0
    }
}
