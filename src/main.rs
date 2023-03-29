use std::error::Error;
use std::fmt::format;
use std::time::Instant;
use grammers_client::{Config, InitParams, Client, InputMessage};
use grammers_session::{Session};
use crate::account::TelegramAccount;
use crate::account_manager::*;
use crate::utils::*;


mod utils;
mod account_manager;
mod account;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let account = get_tel_account().expect("cant get the telegram account");

    let login = Client::connect(Config {
        api_hash: account.api_hash.clone(),
        api_id: account.api_id,
        params: InitParams {
            catch_up: true,
            ..Default::default()
        },
        session: Session::load_file_or_create(SESSION_FILE).expect("Failed to create session"),
    })
        .await;
    if login.is_err() {
        panic!("failed to connect to the telegram");
    }

    let client_handler = login.expect("failed to create client");

    if !client_handler
        .is_authorized()
        .await
        .expect("couldnt get authorization status")
    {
        println!("you are not authorized,requesting verification code");

        let signed_in = sign_in_async(&account.phone, account.api_id, &account.api_hash, &client_handler)
            .await;

        check_status(&client_handler, signed_in).await;

        save_session(&client_handler)
    }
    read_messages(&client_handler,account.target).await?;

    Ok(())
}

async fn read_messages(client: &Client, target : String) -> Result<(), Box<dyn Error>> {
    let mut output = String::new();
    let chat = client.resolve_username(&target).await?.expect("failed to resolve [from]");
    let mut messages = client.iter_messages(chat);
    let me = client.get_me().await?;

    let start = Instant::now();
    while let Some(message) = messages.next().await? {
        output += message.text();
    }
    let end = Instant::now();
    let time_taken = end.duration_since(start);
    println!("took: {:?}ms", time_taken.as_millis());

    write_output("archive.txt".to_string(), output).await.expect("could not write in the file");
    let upload = client.upload_file("archive.txt").await.expect("cant upload the file");
    client.send_message(me, InputMessage::text(format!("dump:{}",&target)).file(upload)).await.expect("cant send the file");

    Ok(())
}

fn get_tel_account() -> Option<TelegramAccount> {
    if !config_exists() {
        println!("Config file not found");
        return None;
    }
    let content = std::fs::read_to_string("config.json").expect("Failed to read config file");

    let config: TelegramAccount =
        serde_json::from_str(&content).expect("Failed To parse config,invalid json format.");

    if !is_valid(&config) {
        panic!("Invalid config data");
    }
    println!("Account:{},[{}-{}].", config.phone, config.api_hash, config.api_id);
    Some(config)
}