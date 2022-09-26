#[macro_use]
extern crate maplit;
extern crate reqwest;
use anyhow::Result;
use async_recursion::async_recursion;
use clap::{App, Arg, SubCommand};
use colored::*;
use regex::Regex;
use reqwest::{header::AUTHORIZATION, Url};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use tokio::time::{sleep, Duration};

mod auth;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UserJSON {
    id: String,
    name: String,
    username: String,
    location: Option<String>,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TimelineJSON {
    data: Vec<TimelineTweetJSON>,
    includes: TimelineIncludeJSON,
    meta: Option<MetaJSON>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TimelineIncludeJSON {
    users: Vec<UserJSON>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TimelineTweetJSON {
    id: String,
    text: String,
    author_id: String,
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

    async fn fetch(&self, method: &str, url_string: &str, body: &HashMap<&str, &str>) -> Response {
        let signature = auth::make_signature(
            url_string,
            method,
            &self.api_key,
            &self.api_key_secret,
            &self.access_token,
            &self.access_token_secret,
            body,
        );

        let url = Url::parse(url_string).unwrap();
        let client = Client::new()
            .get(url)
            .header(AUTHORIZATION, &signature.auth_header)
            .form(&body);

        client.send().await.unwrap()
    }

    async fn fetch_json<T>(&self, method: &str, url_string: &str, body: &HashMap<&str, &str>) -> T
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
        url_string: &str,
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
            body_with_token.insert("pagination_token", k);
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

    pub async fn get_user(&self, screen_name: &str, body: &HashMap<&str, &str>) -> UserJSON {
        let url_string = format!(
            "https://api.twitter.com/2/users/by/username/{}",
            screen_name
        );
        let res = self
            .fetch_json::<DataJSON<UserJSON>>("GET", &url_string, body)
            .await;
        res.data
    }

    pub async fn get_timeline<'a>(
        &self,
        user_id: String,
    ) -> (Vec<TimelineTweetJSON>, HashMap<String, UserJSON>) {
        let url_string = format!(
            "https://api.twitter.com/2/users/{}/timelines/reverse_chronological",
            user_id
        );

        let res_json = self
            .fetch_text(
                "GET",
                &url_string,
                &hashmap! {
                "expansions"=>"author_id",
                "user.fields" => "id,name,username" },
            )
            .await;
        let res: TimelineJSON = serde_json::from_str(&res_json).unwrap();

        let mut usermap = HashMap::new();
        for user in &res.includes.users {
            usermap.insert(user.id.clone(), user.clone());
        }

        (res.data, usermap)
    }

    pub async fn get_followers(&self, user_id: &String) -> Vec<UserJSON> {
        let url_string = format!("https://api.twitter.com/2/users/{}/followers", user_id);
        self.fetch_json_all::<UserJSON>(
            "GET",
            &url_string,
            &hashmap! { "max_results" => "1000", "user.fields" => "id,name,username,location,description" },
        )
        .await
    }

    pub async fn get_following(&self, user_id: &String) -> Vec<UserJSON> {
        let url_string = format!("https://api.twitter.com/2/users/{}/following", user_id);
        self.fetch_json_all::<UserJSON>(
            "GET",
            &url_string,
            &hashmap! { "max_results" => "1000", "user.fields" => "id,name,username,location,description" },
        )
        .await
    }
}

fn print_users_in_location(followers: &[UserJSON], location: Regex) {
    let followers: Vec<&UserJSON> = followers
        .iter()
        .filter(|f| match &f.location {
            Some(loc) => location.is_match(loc.to_lowercase().as_str()),
            _ => false,
        })
        .collect();

    println!(
        ">> Followers in '{}': {}",
        location.as_str().green(),
        followers.len()
    );
    for f in &followers {
        println!(
            "{} (@{}): {} {}",
            f.name.green().bold(),
            f.username.bright_black(),
            f.location.as_ref().unwrap_or(&String::new()).bright_blue(),
            format!("https://twitter.com/{}", f.username).bright_black(),
        );
    }
}

fn env(key: &str) -> String {
    std::env::var(key).unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = initialize_app();
    let matches = app.get_matches();

    let twit = Twit::new(
        env("TWITTER_API_KEY"),
        env("TWITTER_ACCESS_TOKEN"),
        env("TWITTER_API_KEY_SECRET"),
        env("TWITTER_ACCESS_TOKEN_SECRET"),
    );

    if let Some(matches) = matches.subcommand_matches("user") {
        // Show user profile
        let username = matches.value_of("name").unwrap();
        let user = twit
            .get_user(
                username,
                &hashmap! {"user.fields" => "id,name,username,location,description"},
            )
            .await;
        println!("{:?}", user);
    } else if let Some(matches) = matches.subcommand_matches("timeline") {
        // Show user's home timeline
        let username = matches.value_of("name").unwrap();
        let user = twit
            .get_user(username, &hashmap! {"user.fields" => "id"})
            .await;

        let (tweets, usermap) = twit.get_timeline(user.id).await;
        for tweet in tweets {
            let user = usermap.get(&tweet.author_id).unwrap();
            println!(
                "{} {}: {}\n",
                user.name.green().bold(),
                format!("@{}", user.username).bright_black(),
                tweet.text.bright_blue()
            );
        }
    } else if let Some(matches) = matches.subcommand_matches("following") {
        // Get following users
        let username = matches.value_of("name").unwrap();
        let user = twit
            .get_user(
                username,
                &hashmap! {"user.fields" => "id,name,username,location"},
            )
            .await;

        let followers = twit.get_following(&user.id).await;
        let followers_json = serde_json::to_string(&followers)?;
        let mut file = File::create("following.json")?;
        file.write_all(followers_json.as_bytes())?;
    } else if let Some(matches) = matches.subcommand_matches("followers") {
        // Get followers of the user
        let username = matches.value_of("name").unwrap();
        let user = twit
            .get_user(
                username,
                &hashmap! {"user.fields" => "id,name,username,location"},
            )
            .await;

        let followers = twit.get_followers(&user.id).await;
        let followers_json = serde_json::to_string(&followers)?;
        let mut file = File::create("followers.json")?;
        file.write_all(followers_json.as_bytes())?;
    } else if let Some(matches) = matches.subcommand_matches("list") {
        let followers_path = matches.value_of("followers").unwrap();
        let following_path = matches.value_of("following").unwrap();
        let patterns = matches.values_of("pattern").unwrap();

        // Read followers from JSON
        let mut file = File::open(followers_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let followers: Vec<UserJSON> = serde_json::from_str(&contents)?;
        println!(">> Followers: {}", followers.len());

        // Read following users from JSON
        let mut file = File::open(following_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let following: Vec<UserJSON> = serde_json::from_str(&contents)?;
        println!(">> Following: {}", following.len());

        // Get users listed both on follows and following.
        let followers_ids: HashSet<String> = followers.iter().map(|f| f.id.clone()).collect();
        let users: Vec<UserJSON> = following
            .into_iter()
            .filter(|f| followers_ids.contains(&f.id))
            .collect();
        println!(">> Mutual: {}", users.len());

        for pattern in patterns {
            println!();
            print_users_in_location(&users, Regex::new(pattern)?);
            println!();
        }
    }

    Ok(())
}

fn initialize_app() -> App<'static> {
    App::new("find-nearby-twitterer")
        .version("0.1.0")
        .author("AMAGI")
        .about("Find twitter friends living nearby")
        .subcommand(
            SubCommand::with_name("user")
                .about("Show account info")
                .arg(
                    Arg::with_name("name")
                        .help("Account name to see info")
                        .takes_value(true)
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("timeline")
                .about("Show home timeline for the user")
                .arg(
                    Arg::with_name("name")
                        .help("Account name to show the timeline")
                        .takes_value(true)
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("followers")
                .about("Get your followers and save to a JSON file")
                .arg(
                    Arg::with_name("name")
                        .help("Account name to find followers")
                        .takes_value(true)
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("filename")
                        .help("Output JSON filename")
                        .takes_value(true)
                        .required(true)
                        .name("output")
                        .short('o')
                        .default_value("followers.json"),
                ),
        )
        .subcommand(
            SubCommand::with_name("following")
                .about("Get users you follow and save to a JSON file")
                .arg(
                    Arg::with_name("name")
                        .help("Account name to find following users")
                        .takes_value(true)
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("filename")
                        .help("Output JSON filename")
                        .takes_value(true)
                        .required(true)
                        .name("output")
                        .short('o')
                        .default_value("following.json"),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("Get users you follow and save to a JSON file")
                .arg(
                    Arg::with_name("followers")
                        .help("JSON file of users following you")
                        .takes_value(true)
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("following")
                        .help("JSON file of users you follow")
                        .takes_value(true)
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::with_name("pattern")
                        .help("Pattern to filter user location")
                        .takes_value(true)
                        .name("pattern")
                        .short('p')
                        .multiple(true)
                        .required(true),
                ),
        )
}
