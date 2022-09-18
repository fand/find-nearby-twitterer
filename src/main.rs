use anyhow::Result;

extern crate reqwest;
use reqwest::Client;
use reqwest::{header::AUTHORIZATION, Url};

use serde::{Deserialize, Serialize};

mod auth;

#[derive(Serialize, Deserialize, Debug)]
struct UserJSON {
    id: String,
    name: String,
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DataJSON<T> {
    data: T,
}

struct Twit {
    client: Client,
    bearer_token: String,
}

impl Twit {
    pub fn new(bearer_token: String) -> Twit {
        let client = Client::new();

        Twit {
            bearer_token: format!("Bearer {}", bearer_token),
            client,
        }
    }

    pub async fn get_user(&self, screen_name: &str) -> UserJSON {
        let url_string = format!(
            "https://api.twitter.com/2/users/by/username/{}",
            screen_name
        );

        let body = vec![];

        let api_key = std::env::var("TWITTER_API_KEY").unwrap();
        let access_token = std::env::var("TWITTER_ACCESS_TOKEN").unwrap();
        let api_key_secret = std::env::var("TWITTER_API_KEY_SECRET").unwrap();
        let access_token_secret = std::env::var("TWITTER_ACCESS_TOKEN_SECRET").unwrap();

        let signature = auth::make_signature(
            url_string.as_str(),
            "GET",
            api_key,
            api_key_secret,
            access_token,
            access_token_secret,
            &body,
        );

        let url = Url::parse(&url_string).unwrap();
        let client = Client::new()
            .get(url)
            .header(AUTHORIZATION, &signature.auth_header)
            .form(&body);

        let res = client
            .send()
            .await
            .unwrap()
            .json::<DataJSON<UserJSON>>()
            .await
            .unwrap();

        res.data
    }

    pub async fn get_timeline(&self, user_id: String) -> String {
        // let url = format!("https://api.twitter.com/2/user_timeline",);
        let url_string = format!(
            "https://api.twitter.com/2/users/{}/timelines/reverse_chronological",
            user_id
        );

        let body = vec![];

        let api_key = std::env::var("TWITTER_API_KEY").unwrap();
        let access_token = std::env::var("TWITTER_ACCESS_TOKEN").unwrap();
        let api_key_secret = std::env::var("TWITTER_API_KEY_SECRET").unwrap();
        let access_token_secret = std::env::var("TWITTER_ACCESS_TOKEN_SECRET").unwrap();

        let signature = auth::make_signature(
            url_string.as_str(),
            "GET",
            api_key,
            api_key_secret,
            access_token,
            access_token_secret,
            &body,
        );

        let url = Url::parse(&url_string).unwrap();
        let client = Client::new()
            .get(url)
            .header(AUTHORIZATION, &signature.auth_header)
            .form(&body);

        client.send().await.unwrap().text().await.unwrap()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let twit = Twit::new(std::env::var("TWITTER_BEARER_TOKEN").unwrap());
    let user = twit.get_user("amagitakayosi").await;
    let timeline = twit.get_timeline(user.id).await;
    println!("{}", timeline);

    // println!("{}", user.username);

    // let screen_name = "amagitakayosi";

    // let url = format!(
    //     "https://api.twitter.com/2/users/by?usernames={}",
    //     screen_name
    // );

    // let url = format!(
    //     "https://api.twitter.com/1.1/statuses/user_timeline.json?screen_name=amagitakayosi",
    // );

    // let token = std::env::var("TWITTER_BEARER_TOKEN").unwrap();
    // let token = format!("Bearer {}", token);
    // let client = Client::new().get(&url).header("authorization", token);

    // let res = client.send().await.unwrap().text().await.unwrap();
    // println!("{}", res);

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
