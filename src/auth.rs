use chrono::Utc;
use std::collections::HashMap;

extern crate base64;
use hmacsha1::hmac_sha1;
use percent_encoding::{utf8_percent_encode, AsciiSet, PercentEncode, NON_ALPHANUMERIC};

const FRAGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'~')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_');

fn encode(input: &str) -> PercentEncode {
    utf8_percent_encode(input, FRAGMENT)
}

fn make_signature_base_string(method: &str, url: &str, parameter_string: &str) -> String {
    format!(
        "{}&{}&{}",
        encode(method),
        encode(url),
        encode(parameter_string),
    )
}

fn make_signature_key(oauth_consumer_secret: &str, oauth_token_secret: &str) -> String {
    let oauth_consumer_secret = encode(oauth_consumer_secret);
    let oauth_token_secret = encode(oauth_token_secret);
    format!("{}&{}", oauth_consumer_secret, oauth_token_secret)
}

fn encode_signature(key: &str, data: &str) -> String {
    let hash = hmac_sha1(key.as_bytes(), data.as_bytes());
    base64::encode(&hash)
}

fn make_parameter_string(params: &HashMap<&str, &str>) -> String {
    let mut params = params
        .into_iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect::<Vec<String>>();
    params.sort();
    params.join("&")
}

pub struct Signature {
    pub signature: String,
    pub auth_header: String,
}

pub fn make_signature<'a>(
    url: &str,
    method: &str,
    api_key: &String,
    access_token: &String,
    api_key_secret: &String,
    access_token_secret: &String,
    body: &HashMap<&str, &str>,
) -> Signature {
    let timestamp = Utc::now().timestamp().to_string();

    let mut params: HashMap<&str, &str> = HashMap::new();
    params.insert("oauth_consumer_key", api_key);
    params.insert("oauth_nonce", &timestamp);
    params.insert("oauth_timestamp", &timestamp);
    params.insert("oauth_signature_method", "HMAC-SHA1");
    params.insert("oauth_version", "1.0");
    params.insert("oauth_token", access_token);

    for (k, v) in body {
        params.insert(k, v);
    }

    let parameter_string = make_parameter_string(&params);

    let signature_base_string = make_signature_base_string(method, url, &parameter_string);

    let signature_key = make_signature_key(api_key_secret, access_token_secret);

    let signature = encode_signature(&signature_key, &signature_base_string);

    let auth_header = format!(
        r#"OAuth oauth_nonce="{}", oauth_signature_method="{}", oauth_timestamp="{}", oauth_consumer_key="{}", oauth_signature="{}", oauth_version="{}", oauth_token="{}""#,
        encode(params.get("oauth_nonce").unwrap()),
        encode(params.get("oauth_signature_method").unwrap()),
        encode(params.get("oauth_timestamp").unwrap()),
        encode(params.get("oauth_consumer_key").unwrap()),
        encode(&signature),
        encode(params.get("oauth_version").unwrap()),
        encode(&access_token),
    );

    Signature {
        signature,
        auth_header,
    }
}
