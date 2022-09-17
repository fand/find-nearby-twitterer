use anyhow::Result;
use kuon;
use std::env;

extern crate reqwest;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let url = format!(
        "https://api.twitter.com/1.1/statuses/user_timeline.json?screen_name=amagitakayosi",
    );

    let token = std::env::var("TWITTER_BEARER_TOKEN").unwrap();
    let token = format!("Bearer {}", token);
    let client = Client::new().get(&url).header("authorization", token);

    let res = client.send().await.unwrap().text().await.unwrap();
    println!("{}", res);
    Ok(())

    // println("{}", client);

    // let access_token = env::var("TWITTER_ACCESS_TOKEN").unwrap();
    // let access_token_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET").unwrap();
    // let api_key = env::var("TWITTER_API_KEY").unwrap();
    // let api_key_secret = env::var("TWITTER_API_KEY_SECRET").unwrap();

    // let builder = kuon::TwitterAPI::builder()
    //     .access_token(access_token)
    //     .access_token_secret(access_token_secret)
    //     .api_key(api_key)
    //     .api_secret_key(api_key_secret);

    // let api = builder.build().await?;

    // let res = api.user_timeline().screen_name("yulily100").send().await;
    // // let res = api.search_tweets().q("rust").send().await;

    // match res {
    //     Ok(search_result) => {
    //         println!(">> {} tweets found!", search_result.len());
    //         for tweet in search_result {
    //             println!("{}", tweet.text);
    //         }
    //     }
    //     Err(kuon::Error::TwitterAPIError(e, param_str)) => {
    //         // You can confirm a error originated from Twitter API.
    //         println!("{}", param_str);
    //         assert!(e.errors.len() > 0)
    //     }
    //     Err(kuon::Error::HTTPRequestError(e)) => {
    //         println!("{}", e);
    //         // Do something!
    //     }
    //     _ => panic!("Unexpected error!"),
    // }

    // Ok(())
    // // let res = api.favorite().id(0).send().await?;
    // // let res = api.retweet().id(0).send().await?;
}
