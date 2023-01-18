use async_std::task;
use log::{error, info};
use std::error::Error;
use twitter_v2::authorization::Oauth1aToken;
use twitter_v2::TwitterApi;

pub struct TwitterKeys {
    auth: Oauth1aToken,
    tokio_runtime: tokio::runtime::Runtime,
}

impl TwitterKeys {
    pub fn new(
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        token_secret: String,
    ) -> Self {
        let auth = Oauth1aToken::new(consumer_key, consumer_secret, access_token, token_secret);
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        TwitterKeys {
            auth,
            tokio_runtime,
        }
    }

    pub(crate) fn tweet(&self, text: String) {
        self.tokio_runtime.block_on(async {
            let result = self.tweet_async(text).await;
            if result.is_err() {
                error!("{:?}", result);
            }
        });
    }

    async fn tweet_async(&self, text: String) -> Result<(), Box<dyn Error>> {
        let api = TwitterApi::new(self.auth.clone());
        let resp = api.post_tweet().text(text).send().await?;
        info!("{}", resp.data().unwrap().id);
        Ok(())
    }
}
