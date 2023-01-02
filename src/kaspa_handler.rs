use crate::proto::kaspad_message::Payload;
use crate::proto::rpc_client::RpcClient;
use crate::proto::{
    BlockAddedNotificationMessage, GetInfoRequestMessage, KaspadMessage,
    NotifyBlockAddedRequestMessage, RpcBlockVerboseData,
};

use crate::Error;
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::{Channel, Endpoint};
use tonic::Streaming;

pub struct KaspaHandler {
    // client: RpcClient<Channel>,
    stream: Streaming<KaspadMessage>,
    send_channel: Sender<KaspadMessage>,
}

impl KaspaHandler {
    pub async fn connect<A>(address: A) -> Result<Self, Error>
    where
        A: std::convert::TryInto<Endpoint>,
        A::Error: Into<Error>,
    {
        let mut client = RpcClient::connect(address).await?;
        let (send_channel, recv) = mpsc::channel(3);
        send_channel.send(GetInfoRequestMessage {}.into()).await?;
        let stream = client
            .message_stream(ReceiverStream::new(recv))
            .await?
            .into_inner();
        Ok(Self {
            // client,
            stream,
            send_channel,
        })
    }

    pub async fn listen(&mut self) -> Result<(), Error> {
        loop {
            while let Some(msg) = self.stream.message().await? {
                match msg.payload {
                    Some(payload) => self.handle_message(payload).await?,
                    None => println!("kaspad message payload is empty"),
                }
            }
        }
    }

    async fn handle_message(&self, message: Payload) -> Result<(), Error> {
        match message {
            Payload::GetInfoResponse(info) => {
                println!("Kaspad version: {}", info.server_version);
                self.send_channel
                    .send(NotifyBlockAddedRequestMessage {}.into())
                    .await?;
            }
            Payload::NotifyBlockAddedResponse(response_message) => {
                if let Some(err) = response_message.error {
                    return Err(err.message.into());
                }
            }
            Payload::BlockAddedNotification(block_added_notif_message) => {
                return self.handle_new_block(block_added_notif_message).await;
            }
            _ => println!("unrelated message received: {:?}", message),
        }
        Ok(())
    }

    async fn handle_new_block(
        &self,
        block_notif: BlockAddedNotificationMessage,
    ) -> Result<(), Error> {
        println!("new block notified!",);
        let verbose_data = extract_verbose_data(&block_notif)?;
        if !verbose_data.is_chain_block {
            // println!("not part of chain");
            return Ok(());
        }

        let txs = block_notif.block.unwrap().transactions;
        for tx in txs.iter() {
            if tx.inputs.len() == 0 {
                continue;
            }
            // println!("{}", tx.inputs.len());
            // println!("{}", tx.outputs.len());

            let total_output: u64 = tx.outputs.iter().map(|op| op.amount).sum();
            let max_output: u64 = tx.outputs.iter().map(|op| op.amount).max().unwrap();
            // println!(
            //     "tx total output amount {}, largest output amount {}",
            //     total_output, max_output
            // );
        }

        // println!("accepted block!");

        Ok(())
    }
}

impl From<GetInfoRequestMessage> for KaspadMessage {
    #[inline(always)]
    fn from(a: GetInfoRequestMessage) -> Self {
        KaspadMessage {
            payload: Some(Payload::GetInfoRequest(a)),
        }
    }
}

impl From<NotifyBlockAddedRequestMessage> for KaspadMessage {
    fn from(a: NotifyBlockAddedRequestMessage) -> Self {
        KaspadMessage {
            payload: Some(Payload::NotifyBlockAddedRequest(a)),
        }
    }
}

fn extract_verbose_data(
    block_notif: &BlockAddedNotificationMessage,
) -> Result<RpcBlockVerboseData, Error> {
    let err_message = "failed to extract verbose data from block";
    match &block_notif.block {
        Some(block) => match &block.verbose_data {
            Some(verbose_data) => return Ok(verbose_data.clone()),
            None => Err(err_message.into()),
        },
        None => Err(err_message.into()),
    }
}
