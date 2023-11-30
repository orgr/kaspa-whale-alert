use log::{debug, error, info};
use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Event, Payload, RawClient};
use serde::Deserialize;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::Error;

const CIRCULATION_REQUEST_URL: &str =
    "https://api.kaspa.org/info/coinsupply/circulating?in_billion=false";

macro_rules!  get_tx_extra_info_req {
    ($tx_id:ident) => {
        format!("https://api.kaspa.org/transactions/{}?inputs=true&outputs=true&resolve_previous_outpoints=light", $tx_id)
    };
}
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

pub struct OutputCandidate {
    pub idx: usize,
    pub amount: f64,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct TxExtraInfo {
    pub is_accepted: bool,
    pub inputs: Vec<TxExtraInfoInput>,
    pub outputs: Vec<TxExtraInfoOutput>,
}

#[derive(Debug, Deserialize)]
pub struct TxExtraInfoInput {
    pub previous_outpoint_address: String,
    pub previous_outpoint_amount: u64,
}

#[derive(Debug, Deserialize)]
pub struct TxExtraInfoOutput {
    pub amount: u64,
    pub script_public_key_address: String,
}

pub struct RestHandler {
    circulation: Mutex<f64>,
}

impl RestHandler {
    pub fn handle(
        tx_send: SyncSender<Vec<OutputCandidate>>,
        websocket_url: String,
        whale_factor: f64,
    ) -> Arc<Self> {
        let arc = Arc::new(Self {
            circulation: Mutex::new(0.0),
        });
        let (supply_ready_send, supply_ready_recv): (SyncSender<()>, Receiver<()>) =
            sync_channel(1);
        let arc_clone_for_ws_listener = arc.clone();
        thread::spawn(move || {
            supply_ready_recv.recv().unwrap();
            info!("listening for ws_client errors...");
            loop {
                let (ws_client_err_send, ws_client_err_recv): (SyncSender<()>, Receiver<()>) =
                    sync_channel(1);
                let ws_client = Self::new_ws_client(
                    arc_clone_for_ws_listener.clone(),
                    tx_send.clone(),
                    ws_client_err_send,
                    websocket_url.clone(),
                    whale_factor,
                )
                .unwrap();
                ws_client_err_recv.recv().unwrap();
                drop(ws_client_err_recv);
                ws_client.disconnect().unwrap();
            }
        });
        arc.clone().listen_to_circulation(supply_ready_send);
        arc
    }

    fn new_ws_client(
        rest_handler: Arc<Self>,
        tx_send: SyncSender<Vec<OutputCandidate>>,
        err_send: SyncSender<()>,
        websocket_url: String,
        whale_factor: f64,
    ) -> Result<Client, Error> {
        let tx_clone = tx_send.clone();
        let block_handler = move |payload: Payload, _| match payload {
            Payload::String(string_data) => {
                if let Ok(block_payload) = serde_json::from_str::<NewBlockPayload>(&string_data) {
                    debug!("new block");
                    let mut amount_vec = Vec::<OutputCandidate>::new();
                    let txs = block_payload.txs;

                    let supply = rest_handler.get_circulation();
                    assert!(supply != 0.0);
                    let threshold = Self::get_threshold(whale_factor, supply);
                    for tx in &txs[1..txs.len()] {
                        // skip coinbase tx
                        for (i, output) in tx.outputs.iter().enumerate() {
                            let explicit_amount = output.1.parse::<u64>().unwrap();
                            let kas_amount = Self::explicit_amount_to_kas_amount(explicit_amount);
                            if kas_amount > threshold {
                                amount_vec.push(OutputCandidate {
                                    idx: i,
                                    amount: (kas_amount),
                                    id: (tx.tx_id.clone()),
                                })
                            }
                        }
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

    fn listen_to_circulation(self: Arc<Self>, ready_send: SyncSender<()>) {
        thread::spawn(move || {
            info!("sync started");
            let mut ready = false;
            loop {
                match self.update_circulation() {
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

    fn update_circulation(&self) -> Result<(), Error> {
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

    pub fn get_tx_extra_info(&self, tx_id: &str) -> Result<TxExtraInfo, Error> {
        let req_url = get_tx_extra_info_req!(tx_id);
        let response_str = reqwest::blocking::get(req_url)?.text()?;
        match serde_json::from_str::<TxExtraInfo>(&response_str) {
            Ok(tx_extra_info) => return Ok(tx_extra_info),
            Err(_) => Err("Parsing TxExtraInfo failed".into()),
        }
    }

    fn get_threshold(whale_factor: f64, supply: f64) -> f64 {
        whale_factor / 100.0 * supply
    }

    const EXPLICIT_AMOUNT_IN_KAS_AMOUNT: u64 = 100000000;

    fn explicit_amount_to_kas_amount(explicit: u64) -> f64 {
        (explicit / Self::EXPLICIT_AMOUNT_IN_KAS_AMOUNT) as f64
    }
}
