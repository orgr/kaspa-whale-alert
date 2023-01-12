use serde::{Deserialize, Deserializer};
use std::sync::{Arc, Mutex};

use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Event, Payload, RawClient};

const BLOCKDAG_REQUEST_URL: &str = "https://api.kaspa.org/info/blockdag";
const KASPA_REST_SOCKETIO_URL: &str = "https://api.kaspa.org/ws/socket.io/";
const POLL_INTERVAL_SEC: u64 = 5;

pub struct RestHandler {
    socket: Client,
    blocks: Arc<Mutex<Vec<u64>>>,
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
    enum StringOrOthfer {
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
    pub fn new() -> Self {
        let blocks = Arc::new(Mutex::new(Vec::<u64>::new()));

        println!("we got here 1");
        let block_handler = |payload: Payload, socket: RawClient| match payload {
            Payload::String(string_data) => {
                let block_payload: NewBlockPayload =
                    serde_json::from_str(&string_data).expect("New block parsing failed");
                println!("{:?}", block_payload);
            }
            _ => println!("Unrecognized new-block payload"),
        };

        let socket = ClientBuilder::new(KASPA_REST_SOCKETIO_URL)
            .on(Event::Connect, |_, socket: RawClient| {
                println!("SocketIO connected!!");
                socket.emit("join-room", "blocks");
            })
            .on("new-block", block_handler)
            .connect()
            .expect("websocket connection failed");
        println!("we got here 2");

        Self {
            socket,
            blocks: Arc::new(Mutex::new(Vec::<u64>::new())),
        }
    }
}
