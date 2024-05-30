use mpl_token_metadata::accounts::Metadata;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use solana_account_decoder::parse_account_data::ParsedAccount;
use solana_client::client_error::reqwest;
use solana_client::client_error::reqwest::{Error, Url};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_request::TokenAccountsFilter::ProgramId;
use solana_sdk::pubkey::Pubkey;
use spl_token::ID;
use std::str::FromStr;
use mime::Mime;

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenCreator {
    pub name: String,
    pub site: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenExtensions {
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenData {
    pub name: String,
    pub symbol: Option<String>,
    pub image: Option<String>,
    pub description: Option<String>,
    pub extensions: Option<TokenExtensions>,
    pub tags: Option<Vec<String>>,
    pub creator: Option<TokenCreator>,
}

impl TokenData {
    pub async fn get_image(&self, pubkey: &String) -> Result<String, Box<dyn std::error::Error>> {
        let default_file_path = "app/assets/token-images/default.png"; // default path
        let uri = "https://solana-cdn.com/cdn-cgi/image/width=40/".to_string()+&self.image.as_ref().unwrap().to_string();
        let url = Url::parse(&uri)?;
        let pubkey_filtered = pubkey.replace(|c: char| !c.is_alphanumeric(), "");

        // Send a GET request and check the response status before continuing
        let response = reqwest::get(url.as_str()).await?;
        if response.status() != 200 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Download failed")));
        }

        let content_type = response.headers().get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<Mime>().ok());

        let ext = match content_type {
            Some(mime) => match mime.type_().as_str() {
                "image" => match mime.subtype().as_str() {
                    "png" => "png",
                    "jpeg" => "jpg",
                    "jpg" => "jpg",
                    "gif" => "gif",
                    "bmp" => "bmp",
                    "webp" => "webp",
                    "svg" => "svg",
                    "ico" => "ico",
                    "tiff" => "tiff",
                    "heif" => "heif",
                    "avif" => "avif",
                    "raw" => "raw",
                    _ => "img",
                },
                _ => {
                    // Return the default file path if the file is not an image
                    return Ok(String::from(default_file_path))
                },
            },
            None => {
                // Return the default file path if there's no content type
                return Ok(String::from(default_file_path))
            },
        };

        let file_path_str = format!("app/assets/token-images/{}.{}", pubkey_filtered, ext);

        // Download and save the image
        let bytes = response.bytes().await?;
        tokio::fs::write(&file_path_str, &bytes).await?;

        Ok(file_path_str)
    }
}

#[derive(Debug)]
pub struct ParsedAccountInfo<'a> {
    pub mint: String,
    pub owner: String,
    pub token_amount_int: f32,
    pub token_amount_string: &'a str,
    // pub token_amount_formatted: String,
}

impl ParsedAccountInfo<'_>  {
    pub fn cleaned_mint_address(&self) -> String {
        return self.mint.replace("\"", "");
    }
}

#[derive(Debug)]
pub struct WalletToken {
    pub parsed_account: ParsedAccount,
    pub metadata_uri: Option<String>,
}

impl WalletToken {
    pub async fn metadata(&self) -> Result<TokenData, Error> {
        let uri = &self.metadata_uri.as_ref().unwrap().to_string();
        let response = reqwest::get(uri).await?;
        let text = response.text().await?;
        let data: TokenData = serde_json::from_str(&text).expect("Failed to parse JSON");
        Ok(data)
    }

    pub fn parsed_account_info(&self) -> ParsedAccountInfo{
        let account_info = &self.parsed_account.parsed.get("info").unwrap();
        let token_amount = account_info.get("tokenAmount").unwrap().get("uiAmount").unwrap().as_f64().unwrap();
        return ParsedAccountInfo {
            mint: account_info.get("mint").unwrap().to_string(),
            owner: account_info.get("owner").unwrap().to_string(),
            token_amount_int: token_amount as f32,
            token_amount_string: account_info.get("tokenAmount").unwrap().get("uiAmountString").unwrap().as_str().unwrap(),
            // token_amount_formatted: format_token_amount(token_amount),
        }
    }

    pub fn display_mint_address(&self) -> (String, String) {
        let mint = &self.parsed_account_info().mint.to_string();
        let first_five: String = mint.chars().take(6).collect();
        let total_chars: Vec<char> = mint.chars().collect();
        let last_five: String = total_chars.iter().rev().take(6).rev().cloned().collect();

        (first_five, last_five)
    }

    pub fn format_token_amount(&self) -> String {
        let account_info = &self.parsed_account.parsed.get("info").unwrap();
        let amount = account_info.get("tokenAmount").unwrap().get("uiAmount").unwrap().as_f64().unwrap();
        let abs_num = amount.abs();
        let suffixes = ["", "K", "M", "B", "T", "Q", "Qi", "S", "Sp"];

        let mut counter: usize = 1;
        let mut value: f64;

        if abs_num < 1000000.0 {
            let s = format!("{:.2}", abs_num);
            let parts: Vec<&str> = s.split('.').collect();
            let integral = parts[0];
            let decimals = if parts.len() > 1 { parts[1] } else { "" };
            let integral_with_commas: String = integral
                .chars()
                .rev()
                .enumerate()
                .map(|(i, c)| if i % 3 == 0 && i != 0 { format!(",{}", c) } else { format!("{}", c) })
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>();

            return format!("{}.{}", integral_with_commas, decimals);
        }

        while counter < suffixes.len() {
            value = abs_num / (10f64.powf(3.0 * counter as f64));
            if value < 1000.0 {
                return format!("{:.2}{}", value, suffixes[counter]);
            }
            counter += 1;
        }

        format!("{:.2}{}", abs_num / (10f64.powf(3.0 * (suffixes.len() - 1) as f64)), suffixes[suffixes.len() - 1])
    }
}

pub fn my_wallet(pubkey: &Pubkey, rpc_client: &RpcClient) -> Vec<WalletToken>{
    let token_accounts = rpc_client.get_token_accounts_by_owner(&pubkey, ProgramId(ID)).unwrap();
    let mut wallet_tokens_data: Vec<WalletToken> = Vec::new();

    for token_account in token_accounts {
        let ui_account_data= token_account.account.data;
        let ui_account_data_json_string = to_string(&ui_account_data);

        match ui_account_data_json_string {
            Ok(json) => {
                let deserialized: Result<ParsedAccount, _> = serde_json::from_str(&json);
                match deserialized {
                    Ok(parsed_account) => {
                        let parsed_account_amount = parsed_account.parsed
                            .get("info").unwrap()
                            .get("tokenAmount").unwrap()
                            .get("amount").unwrap()
                            .as_str().unwrap();
                        if parsed_account_amount != "0" {
                            let parsed_account_value = &parsed_account.parsed.get("info").unwrap().get("mint").unwrap();
                            let parsed_account_pubkey = Metadata::find_pda(&Pubkey::from_str(parsed_account_value.as_str().unwrap()).unwrap());

                            let parsed_account_data =  rpc_client.get_account_data(&parsed_account_pubkey.0).expect("TODO: panic message");
                            let parsed_account_metadata = Metadata::from_bytes(&parsed_account_data);
                            let temp_meta = parsed_account_metadata.unwrap().uri;
                            let metadata_uri = *&temp_meta.trim_end_matches('\0');

                            let wallet_token_instance = WalletToken {
                                parsed_account,
                                metadata_uri: Some(String::from(metadata_uri)),
                            };
                            wallet_tokens_data.push(wallet_token_instance);
                        }
                    }
                    Err(e) => eprintln!("Failed to deserialize JSON: {:?}", e),
                }
            }
            Err(e) => eprintln!("Failed to serialize UiAccountData to JSON: {:?}", e),
        }
    }
    wallet_tokens_data
}