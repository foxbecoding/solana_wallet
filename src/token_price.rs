use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;

#[derive(Serialize, Deserialize, Debug)]
pub struct PriceData {
    pub id: String,
    pub mintSymbol: String,
    pub vsToken: String,
    pub vsTokenSymbol: String,
    pub price: f64,
}

impl PriceData {
    pub fn price_as_f32(&self) -> f32 {
        self.price as f32
    }
    pub fn price_formatted(&self) -> f32 {
        let price = self.price_as_f32();
        format!("{:.6}", price).parse::<f32>().unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData {
    #[serde(flatten)]
    pub data: HashMap<String, PriceData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub data: ResponseData,
    pub timeTaken: f64,
}

pub async fn token_price(pubkey: Pubkey) -> Result<Response, Error> {
    let url = format!("https://price.jup.ag/v6/price?ids={}", pubkey);
    let response = reqwest::get(url).await?.text().await?;
    let data: Response = serde_json::from_str(response.as_ref()).unwrap();
    Ok(data)
}