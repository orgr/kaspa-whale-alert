use log::{debug, error, info};
use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Event, Payload, RawClient};
use serde::{Deserialize, Deserializer};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::Error;

const CIRCULATION_REQUEST_URL: &str =
    "https://api.kaspa.org/info/coinsupply/circulating?in_billion=false";
const KASPA_REST_SOCKETIO_URL: &str = "http://kaspa.ddnss.de:8001/ws/socket.io/";
const POLL_INTERVAL_SEC: u64 = 5 * 60;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewBlockPayload {
    txs: Vec<Tx>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Tx {
    tx_id: String,
    outputs: Vec<(String, String)>,
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
        websocket_url: String,
    ) -> Arc<Self> {
        let arc = Arc::new(Self {
            circulation: Mutex::new(0.0),
        });

        thread::spawn(move || {
            info!("listening for ws_client errors...");
            loop {
                let (ws_client_err_send, ws_client_err_recv): (SyncSender<()>, Receiver<()>) =
                    sync_channel(1);
                let ws_client =
                    Self::new_ws_client(tx_send.clone(), ws_client_err_send, websocket_url.clone())
                        .unwrap();
                ws_client_err_recv.recv().unwrap();
                drop(ws_client_err_recv);
                ws_client.disconnect().unwrap();
            }
        });
        arc.clone().listen(circulation_ready_send);
        arc
    }

    fn new_ws_client(
        tx_send: SyncSender<Vec<TxInfo>>,
        err_send: SyncSender<()>,
        websocket_url: String,
    ) -> Result<Client, Error> {
        let tx_clone = tx_send.clone();
        let block_handler = move |payload: Payload, _| match payload {
            Payload::String(string_data) => {
                if let Ok(block_payload) = serde_json::from_str::<NewBlockPayload>(&string_data) {
                    let mut amount_vec = Vec::<TxInfo>::new();
                    let txs = block_payload.txs;
                    for tx in &txs[1..txs.len()] {
                        // skip coinbase tx
                        amount_vec.push(TxInfo {
                            amount: tx
                                .outputs
                                .iter()
                                .map(|op| op.1.parse::<u64>().unwrap())
                                .max()
                                .unwrap(),
                            id: tx.tx_id.clone(),
                        });
                    }
                    if amount_vec.len() > 0 {
                        if tx_clone.send(amount_vec).is_err() {
                            debug!("failed to send tx vector on channel");
                        }
                    }
                    return;
                }
                debug!("block data--> {}", string_data);
                debug!("non chain block payload, skipping");
            }
            _ => error!("Unrecognized new-block payload"),
        };

        let error_handler = move |payload: Payload, _| {
            error!("SocketIO Error {:?}, forcing reconnect", payload);
            if let Err(_) = err_send.send(()) {
                info!("error channel double send avoided :)")
            }
        };

        let ws_client = ClientBuilder::new(websocket_url)
            .on(Event::Connect, |_, socket: RawClient| {
                info!("SocketIO connected!");
                while socket.emit("join-room", "blocks").is_err() {}
            })
            .on(Event::Error, error_handler)
            .on("new-block", block_handler)
            .connect()?;
        info!("connect func finished");
        Ok(ws_client)
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
                        debug!("update successful");
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
