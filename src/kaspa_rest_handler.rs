use serde::{Deserialize, Deserializer};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Event, Payload, RawClient};

use crate::Error;

const CIRCULATION_REQUEST_URL: &str =
    "https://api.kaspa.org/info/coinsupply/circulating?in_billion=false";
const KASPA_REST_SOCKETIO_URL: &str = "https://api.kaspa.org/ws/socket.io/";
const POLL_INTERVAL_SEC: u64 = 5 * 60;

pub struct RestHandler {
    socket: Client,
    circulation: Mutex<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewBlockPayload {
    transactions: Vec<Tx>,
    verbose_data: BlockVerboseData,
}

#[derive(Debug, Deserialize)]
struct Tx {
    outputs: Vec<TxOutput>,
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
    enum StringOrOther {
        String(String),
    }

    match StringOrOther::deserialize(to_deserialize)? {
        StringOrOther::String(s) => s.parse::<u64>().map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom("Failed to parse as string")),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlockVerboseData {
    is_chain_block: bool,
}

impl RestHandler {
    pub fn handle(tx: SyncSender<Vec<u64>>) -> Arc<Self> {
        let tx_clone = tx.clone();
        let block_handler = move |payload: Payload, socket: RawClient| match payload {
            Payload::String(string_data) => {
                if let Ok(block_payload) = serde_json::from_str::<NewBlockPayload>(&string_data) {
                    let mut amount_vec = Vec::<u64>::new();
                    for tx in block_payload.transactions {
                        amount_vec.push(tx.outputs.iter().map(|op| op.amount).max().unwrap())
                    }

                    if tx_clone.send(amount_vec).is_err() {
                        println!("failed to send tx vector on channel");
                    }
                    return;
                }
                println!("non chain block payload, skipping");
            }
            _ => println!("Unrecognized new-block payload"),
        };

        let socket = ClientBuilder::new(KASPA_REST_SOCKETIO_URL)
            .on(Event::Connect, |_, socket: RawClient| {
                println!("SocketIO connected!");
                while socket.emit("join-room", "blocks").is_err() {}
            })
            .on("new-block", block_handler)
            .connect()
            .expect("websocket connection failed");

        let arc = Arc::new(Self {
            socket,
            circulation: Mutex::new(0.0),
        });

        arc.clone().listen();
        arc
    }

    fn listen(self: Arc<Self>) {
        thread::spawn(move || {
            println!("REST started sync");
            loop {
                println!("REST - about to sync!");
                self.update();
                println!("finished sync!");
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
