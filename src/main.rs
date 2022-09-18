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
struct TimelineTweetJSON {
    id: String,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DataJSON<T> {
    data: T,
}

struct Twit {
    api_key: String,
    api_key_secret: String,
    access_token: String,
    access_token_secret: String,
}

impl Twit {
    pub fn new(
        api_key: String,
        api_key_secret: String,
        access_token: String,
        access_token_secret: String,
    ) -> Twit {
        Twit {
            api_key,
            api_key_secret,
            access_token,
            access_token_secret,
        }
    }

    pub async fn get_user(&self, screen_name: &str) -> UserJSON {
        let url_string = format!(
            "https://api.twitter.com/2/users/by/username/{}",
            screen_name
        );

        let body = vec![];

        let signature = auth::make_signature(
            url_string.as_str(),
            "GET",
            &self.api_key,
            &self.api_key_secret,
            &self.access_token,
            &self.access_token_secret,
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

    pub async fn get_timeline(&self, user_id: String) -> Vec<TimelineTweetJSON> {
        // let url = format!("https://api.twitter.com/2/user_timeline",);
        let url_string = format!(
            "https://api.twitter.com/2/users/{}/timelines/reverse_chronological",
            user_id
        );

        let body = vec![];

        let signature = auth::make_signature(
            url_string.as_str(),
            "GET",
            &self.api_key,
            &self.api_key_secret,
            &self.access_token,
            &self.access_token_secret,
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
            .json::<DataJSON<Vec<TimelineTweetJSON>>>()
            .await
            .unwrap();

        res.data
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let twit = Twit::new(
        std::env::var("TWITTER_API_KEY").unwrap(),
        std::env::var("TWITTER_ACCESS_TOKEN").unwrap(),
        std::env::var("TWITTER_API_KEY_SECRET").unwrap(),
        std::env::var("TWITTER_ACCESS_TOKEN_SECRET").unwrap(),
    );
    let user = twit.get_user("amagitakayosi").await;
    let timeline = twit.get_timeline(user.id).await;

    for tweet in timeline {
        println!("{}", tweet.text);
    }

    Ok(())
}
