use std::error::Error;
use std::io::{BufRead, Write};
use async_std::fs::{OpenOptions};
use async_std::io::prelude::*;
use crate::account::TelegramAccount;

pub fn prompt(message: &str) -> Option<String> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(message.as_bytes()).unwrap();
    stdout.flush().unwrap();

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut line = String::new();
    stdin.read_line(&mut line).unwrap();
    return Some(line);
}

pub async fn write_output(path: String, content: String) -> Result<(), Box<dyn Error>>{
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)
        .await?;

    file.write_all(content.as_bytes()).await?;
    file.sync_all().await?;
    Ok(())
}

pub fn config_exists() -> bool {
    std::env::current_dir()
        .unwrap()
        .join("config.json")
        .exists()
}

pub fn is_valid(config: &TelegramAccount) -> bool {
    if config.api_hash.is_empty() || config.from.is_empty() || config.to.is_empty() {
        return false;
    }
    if config.api_hash.len() < 3
        || config.api_id < 10
        || config.from.is_empty()
        || config.to.len() < 3
        || config.phone.len() < 5
    {
        return false;
    }
    true
}