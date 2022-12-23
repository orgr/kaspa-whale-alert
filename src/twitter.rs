use std::error::Error;
use twitter_v2::authorization::Oauth1aToken;
use twitter_v2::TwitterApi;

pub struct TwitterKeys {
    auth: Oauth1aToken,
}

impl TwitterKeys {
    pub fn new(
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        token_secret: String,
    ) -> Self {
        let auth = Oauth1aToken::new(consumer_key, consumer_secret, access_token, token_secret);
        TwitterKeys { auth: auth }
    }

    pub(crate) async fn tweet(&self, text: String) -> Result<(), Box<dyn Error>> {
        let api = TwitterApi::new(self.auth.clone());
        let resp = api.post_tweet().text(text).send().await?;
        println!("{}", resp.data().unwrap().id);
        Ok(())
    }
}
