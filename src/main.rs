use dotenv::dotenv;
use twitter_v2::authorization::Oauth1aToken;
use twitter_v2::TwitterApi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let consumer_key = std::env::var("CONSUMER_KEY").expect("CONSUMER_KEY must be set.");
    let consumer_secret = std::env::var("CONSUMER_SECRET").expect("CONSUMER_SECRET must be set.");
    let access_token = std::env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN must be set.");
    let token_secret = std::env::var("TOKEN_SECRET").expect("TOKEN_SECRET must be set.");

    let auth = Oauth1aToken::new(consumer_key, consumer_secret, access_token, token_secret);

    let api = TwitterApi::new(auth);
    let resp = api
        .post_tweet()
        .text("test tweet 2".to_string())
        .send()
        .await?;
    println!("{}", resp.data().unwrap().id);
    Ok(())
}
