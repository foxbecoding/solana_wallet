mod wallet;
use wallet::wallet::my_wallet;

use solana_client::{
    rpc_client::RpcClient,
};
use solana_sdk::{msg, pubkey::Pubkey};
use solana_sdk::native_token::LAMPORTS_PER_SOL;

use dotenv::dotenv;
use std::{
    env,
    str::FromStr,
    path::Path,
    io::Read
};

use std::rc::Rc;
use slint::{SharedString, ModelRc, VecModel, Image};
use slint_generatedApp::Token as SlintToken;
use slint_generatedApp::Solana as SolToken;

slint::include_modules!();
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    match env::var("PUBLIC_ADDRESS_SOLFLARE") {
        Ok(pubkey) => {
            let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
            let public_address = Pubkey::from_str(&pubkey.as_str()).unwrap();
            let wallet = my_wallet(&public_address, &rpc_client);
            let app = App::new().unwrap();

            let sol_token_img_path = Path::new("app/assets/token-images/solana.png");
            let sol_token_img = Image::load_from_path(&sol_token_img_path);
            let sol_token_img = sol_token_img.unwrap_or_else(|_| Image::load_from_path(Path::new("app/assets/token-images/default.png")).expect("Cannot load fallback image"));

            let sol_bal = rpc_client.get_balance(&public_address).unwrap_or_else(|_| 0) as f32 / LAMPORTS_PER_SOL as f32 ;

            let my_sol = SolToken {
                amount: SharedString::from(format!("{:.5}", sol_bal).to_string()),
                amount_int: sol_bal,
                name: SharedString::from("Solana".to_string()),
                symbol: SharedString::from("SOL".to_string()),
                mint: SharedString::from("So11111111111111111111111111111111111111112".to_string()),
                image: sol_token_img
            };

            SolanaToken::get(&app).set_solana(my_sol);

            // Set wallet tokens
            let weak_app_start = app.as_weak().upgrade().unwrap();
            let mut my_tokens: Vec<SlintToken> = Vec::new();
            for token in &wallet {
                let parsed_account_info = token.parsed_account_info();
                if token.metadata_uri.clone().unwrap().as_str() != "" {
                    let metadata = token.metadata().await?;
                    let token_image_filename = metadata.get_image(&parsed_account_info.mint).await;
                    let token_image_filename = token_image_filename.unwrap_or_else(|_| "app/assets/token-images/default.png".to_string());
                    let token_image_path = Path::new(token_image_filename.as_str());
                    let token_image = Image::load_from_path(token_image_path);

                    let token_image = token_image.unwrap_or_else(|_| Image::load_from_path(Path::new("app/assets/token-images/default.png")).expect("Cannot load fallback image"));

                    let token_description = metadata.description.unwrap_or_else(|| "".to_string());

                    let mint_address_parts = &token.display_mint_address();

                    let token_symbol = metadata.symbol.unwrap_or_else(|| format!("{}.....{}", mint_address_parts.0, mint_address_parts.1).to_string());

                    let slint_token = SlintToken {
                        amount: SharedString::from(parsed_account_info.token_amount_string),
                        amount_int: parsed_account_info.token_amount_int,
                        amount_formatted: SharedString::from(parsed_account_info.token_amount_formatted),
                        description: SharedString::from(token_description),
                        image: token_image,
                        mint: SharedString::from(parsed_account_info.mint.to_string()),
                        name: SharedString::from(metadata.name),
                        owner: SharedString::from(parsed_account_info.owner),
                        symbol: SharedString::from(token_symbol),
                    };
                    my_tokens.push(slint_token);
                }
            }

            let the_model : Rc<VecModel<SlintToken>> = Rc::new(VecModel::from(my_tokens));
            let the_model_rc = ModelRc::from(the_model.clone());
            weak_app_start.set_tokens(the_model_rc);

            // Run App
            app.run().unwrap();
        },
        Err(e) => println!("Couldn't read MY_VARIABLE: {}", e),
    }
    Ok(())
}