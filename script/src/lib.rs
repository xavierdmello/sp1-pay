use anyhow::Result;
use reqwest::get;
use alloy_primitives::Bytes;

pub async fn fetch_google_jwt_cert() -> Result<Bytes, Box<dyn std::error::Error>> {
    let url = "https://www.googleapis.com/oauth2/v3/certs";
    let response = get(url).await?.json::<serde_json::Value>().await?;
    let bytes = Bytes::from(serde_json::to_vec(&response).unwrap());

    Ok(bytes)
}