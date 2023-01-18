use log::{debug, error, info};
use rust_socketio::{ClientBuilder, Event, Payload, RawClient};
use serde::{Deserialize, Deserializer};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::Error;

const CIRCULATION_REQUEST_URL: &str =
    "https://api.kaspa.org/info/coinsupply/circulating?in_billion=false";
const KASPA_REST_SOCKETIO_URL: &str = "https://api.kaspa.org/ws/socket.io/";
const POLL_INTERVAL_SEC: u64 = 5 * 60;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewBlockPayload {
    transactions: Vec<Tx>,
    verbose_data: BlockVerboseData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Tx {
    verbose_data: TxVerboseData,
    outputs: Vec<TxOutput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TxVerboseData {
    transaction_id: String,
}

#[derive(Debug, Deserialize)]
struct TxOutput {
    #[serde(deserialize_with = "deserialize_str_to_u64")]
    amount: u64,
}

fn deserialize_str_to_u64<'de, D>(to_deserialize: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringToDeserialize {
        String(String),
    }

    let StringToDeserialize::String(s) = StringToDeserialize::deserialize(to_deserialize)?;
    s.parse::<u64>().map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlockVerboseData {
    is_chain_block: bool,
}

pub struct TxInfo {
    pub amount: u64,
    pub id: String,
}

pub struct RestHandler {
    circulation: Mutex<f64>,
}

impl RestHandler {
    pub fn handle(
        tx_send: SyncSender<Vec<TxInfo>>,
        circulation_ready_send: SyncSender<()>,
    ) -> Arc<Self> {
        let tx_clone = tx_send.clone();
        let block_handler = move |payload: Payload, _| match payload {
            Payload::String(string_data) => {
                if let Ok(block_payload) = serde_json::from_str::<NewBlockPayload>(&string_data) {
                    let mut amount_vec = Vec::<TxInfo>::new();
                    assert!(block_payload.verbose_data.is_chain_block);
                    let txs = block_payload.transactions;
                    for tx in &txs[1..txs.len()] {
                        // skip coinbase tx
                        amount_vec.push(TxInfo {
                            amount: tx.outputs.iter().map(|op| op.amount).max().unwrap(),
                            id: tx.verbose_data.transaction_id.clone(),
                        });
                    }

                    if tx_clone.send(amount_vec).is_err() {
                        debug!("failed to send tx vector on channel");
                    }
                    return;
                }
                debug!("non chain block payload, skipping");
            }
            _ => debug!("Unrecognized new-block payload"),
        };

        ClientBuilder::new(KASPA_REST_SOCKETIO_URL)
            .on(Event::Connect, |_, socket: RawClient| {
                debug!("SocketIO connected!");
                while socket.emit("join-room", "blocks").is_err() {}
            })
            .on("new-block", block_handler)
            .connect()
            .expect("websocket connection failed");

        let arc = Arc::new(Self {
            circulation: Mutex::new(0.0),
        });

        arc.clone().listen(circulation_ready_send);
        arc
    }

    fn listen(self: Arc<Self>, ready_send: SyncSender<()>) {
        thread::spawn(move || {
            info!("sync started");
            let mut ready = false;
            loop {
                match self.update() {
                    Err(e) => error!("{:?}", e),
                    Ok(_) => {
                        if !ready {
                            ready = true;
                            ready_send.send(()).unwrap();
                        }
                        info!("update successful");
                    }
                }
                thread::sleep(Duration::from_secs(POLL_INTERVAL_SEC));
            }
        });
    }

    fn update(&self) -> Result<(), Error> {
        let response: f64 = reqwest::blocking::get(CIRCULATION_REQUEST_URL)?
            .text()?
            .parse()?;
        let mut circulation = self.circulation.lock().unwrap();
        *circulation = response;
        Ok(())
    }

    pub fn get_circulation(&self) -> f64 {
        *self.circulation.lock().unwrap()
    }
}
