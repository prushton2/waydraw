use reqwest;

const URL: &str = "https://keyserver.prushton.com";

pub async fn set(value: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let response = client
        .put(format!("{}/keys", URL))
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
    let response = client
        .get(format!("{}/keys/{}", URL, key))
        .send()
        .await;

    let response = match response {
        Ok(t) if t.status().is_success() => {
            Ok(t)
        }
        Ok(t) if !t.status().is_success() => {
            Err("Pin does not exist".to_owned())
        }
        t => t.map_err(|e| e.to_string())
    };

    let response = response?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    Ok(response)
}

pub async fn delete(key: &str) {
    let client = reqwest::Client::new();
    let _response = client
        .delete(format!("{}/keys/{}", URL, key))
        .send()
        .await
        .unwrap();
}
