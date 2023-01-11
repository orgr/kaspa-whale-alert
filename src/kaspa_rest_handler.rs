use crate::Error;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};

pub struct RestHandler {}

const BLOCKDAG_REQUEST_URL: &str = "https://api.kaspa.org/info/blockdag";
const POLL_INTERVAL_SEC: u64 = 5;

#[derive(Debug, Deserialize)]
struct BlockDagResponse {}

impl RestHandler {
    pub fn new() -> Arc<Self> {
        let price = Mutex::new(0.0);
        Arc::new(Self {})
    }

    pub async fn listen(self: Arc<Self>) {
        tokio::spawn(async move {
            println!("rest handler started");

            // let mut interval = time::interval(Duration::from_secs(5 * 60));
            let mut interval = time::interval(Duration::from_secs(POLL_INTERVAL_SEC));
            loop {
                interval.tick().await;
                let text = reqwest::get(BLOCKDAG_REQUEST_URL)
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap();
                println!("{}", text);
            }
        });
    }
}
