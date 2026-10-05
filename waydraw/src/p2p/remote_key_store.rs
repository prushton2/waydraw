use reqwest;

const URL: &str = "https://keyserver.prushton.com";

pub async fn set(value: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let response = client.put(format!("{}/keys", URL))
        .body(value.to_owned())
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    Ok(response)
}

pub async fn get(key: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/keys/{}", URL, key))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    Ok(response)
}

pub async fn delete(key: &str) {
    let client = reqwest::Client::new();
    let _response = client.delete(format!("{}/keys/{}", URL, key))
        .send()
        .await
        .unwrap();
}