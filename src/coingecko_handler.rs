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
const POLL_INTERVAL_SEC: u64 = 5 * 60;

#[derive(Debug, Deserialize)]
struct CoingeckoResponse {
    kaspa: CoingeckoResponseCoin,
}

#[derive(Debug, Deserialize)]
struct CoingeckoResponseCoin {
    usd: f64,
}

impl CoinGeckoHandler {
    pub fn handle() -> Arc<Self> {
        let price = Mutex::new(0.0);

        let arc = Arc::new(Self { price });
        arc.clone().listen();
        arc
    }

    fn listen(self: Arc<Self>) {
        thread::spawn(move || {
            println!("COINMARKETCAP started sync");
            loop {
                println!("COINMARKETCAP - about to sync!");
                match self.update() {
                    Err(e) => println!("{:?}", e),
                    Ok(_) => println!("it went ok"),
                }
                println!("finished sync!");
                thread::sleep(Duration::from_secs(POLL_INTERVAL_SEC));
            }
        });
    }

    fn update(&self) -> Result<(), Error> {
        let response: CoingeckoResponse = reqwest::blocking::get(COINGECKO_REQUEST_URL)?.json()?;
        println!("response {:?}", response);
        let mut price = self.price.lock().unwrap();
        *price = response.kaspa.usd;
        Ok(())
    }

    pub fn get_price(&self) -> f64 {
        *self.price.lock().unwrap()
    }
}
