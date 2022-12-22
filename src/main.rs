use reqwest::Client;
use reqwest_oauth1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // twitter client initialization
    // let secrets = reqwest_oauth1::Secrets::new(consumer_key, consumer_secret)
    //     .token(access_token, token_secret);
    // let client = Client::new();
    // let resp = client
    //     .get("https://api.twitter.com/2/tweets/search/recent?query=from:twitterdev")
    //     // .oauth1(secrets)
    //     .send()
    //     .await?;
    // println!("{}", resp.text().await.unwrap());
    Ok(())
}
