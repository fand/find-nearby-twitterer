use anyhow::Result;

extern crate reqwest;
use reqwest::{header::AUTHORIZATION, Url};
use reqwest::{Client, Response};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json;

use std::fs::File;
use std::io::prelude::*;

use async_recursion::async_recursion;

use std::collections::HashMap;
#[macro_use]
extern crate maplit;

use tokio::time::{sleep, Duration};

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
    meta: Option<MetaJSON>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MetaJSON {
    result_count: Option<i64>,
    previous_token: Option<String>,
    next_token: Option<String>,
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

    async fn fetch(
        &self,
        method: &str,
        url_string: &String,
        body: &HashMap<&str, &str>,
    ) -> Response {
        let signature = auth::make_signature(
            url_string.as_str(),
            method,
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

        client.send().await.unwrap()
    }

    async fn fetch_json<T>(
        &self,
        method: &str,
        url_string: &String,
        body: &HashMap<&str, &str>,
    ) -> T
    where
        T: DeserializeOwned,
    {
        self.fetch(method, url_string, body)
            .await
            .json::<T>()
            .await
            .unwrap()
    }

    async fn fetch_text(
        &self,
        method: &str,
        url_string: &String,
        body: &HashMap<&str, &str>,
    ) -> String {
        self.fetch(method, url_string, body)
            .await
            .text()
            .await
            .unwrap()
    }

    #[async_recursion(?Send)]
    async fn fetch_json_with_token<T>(
        &self,
        method: &str,
        url_string: &String,
        body: &HashMap<&str, &str>,
        pagenation_token: Option<&'async_recursion String>,
    ) -> Vec<T>
    where
        T: DeserializeOwned,
    {
        let mut body_with_token = body.clone();
        if let Some(k) = pagenation_token {
            println!("pagenation token: {}", k);
            body_with_token.insert("pagination_token", &k);
        }

        let res_json = self
            .fetch(method, url_string, &body_with_token)
            .await
            .text()
            .await
            .unwrap();

        // println!("{}", res_json);

        let res: DataJSON<Vec<T>> = serde_json::from_str(&res_json).unwrap();
        let mut head = res.data;

        sleep(Duration::from_millis(3000)).await;

        match res.meta.map(|m| m.next_token) {
            Some(Some(next_token)) => {
                let r = self
                    .fetch_json_with_token::<T>(method, url_string, body, Some(next_token).as_ref())
                    .await;
                head.extend(r);
                head
            }
            _ => head,
        }
    }

    async fn fetch_json_all<T>(
        &self,
        method: &str,
        url_string: &String,
        body: &HashMap<&str, &str>,
    ) -> Vec<T>
    where
        T: DeserializeOwned,
    {
        self.fetch_json_with_token(method, url_string, body, None)
            .await
    }

    pub async fn get_user(&self, screen_name: &str) -> UserJSON {
        let url_string = format!(
            "https://api.twitter.com/2/users/by/username/{}",
            screen_name
        );
        let res = self
            .fetch_json::<DataJSON<UserJSON>>("GET", &url_string, &HashMap::new())
            .await;
        res.data
    }

    pub async fn get_timeline(&self, user_id: String) -> Vec<TimelineTweetJSON> {
        let url_string = format!(
            "https://api.twitter.com/2/users/{}/timelines/reverse_chronological",
            user_id
        );
        let res = self
            .fetch_json::<DataJSON<Vec<TimelineTweetJSON>>>("GET", &url_string, &HashMap::new())
            .await;
        res.data
    }

    pub async fn get_followers(&self, user_id: &String) -> Vec<UserJSON> {
        let url_string = format!("https://api.twitter.com/2/users/{}/followers", user_id);
        self.fetch_json_all::<UserJSON>("GET", &url_string, &hashmap! { "max_results" => "1000" })
            .await
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
    // let user = twit.get_user("amagitakayosi").await;

    // Show user's home timeline
    // let timeline = twit.get_timeline(user.id).await;
    // for tweet in timeline {
    //     println!("{}", tweet.text);
    // }

    // // Get followers of the user
    // let followers = twit.get_followers(&user.id).await;
    // let followers_json = serde_json::to_string(&followers)?;
    // let mut file = File::create("followers.json")?;
    // file.write_all(followers_json.as_bytes())?;

    // Read followers from JSON
    let file = File::open("followers.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents);

    println!("{}", contents);

    Ok(())
}
