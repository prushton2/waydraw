use rand::Rng;
use reqwest;

const URL: &str = "https://keyserver.prushton.com";
const KEY_CHARS: &[u8] = b"0123456789ABCDEF";

pub fn generate_pin() -> String {
    let mut rng = rand::rng();
    (0..4)
        .map(|_| KEY_CHARS[rng.random_range(0..KEY_CHARS.len())] as char)
        .collect()
}

pub async fn set(key: &str, value: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let _response = client.put(format!("{}/keys/{}", URL, key))
        .body(value.to_owned())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn get(key: &str) -> String {
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/keys/{}", URL, key))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    response
}

pub async fn delete(key: &str) {
    let client = reqwest::Client::new();
    let _response = client.delete(format!("{}/keys/{}", URL, key))
        .send()
        .await
        .unwrap();
}