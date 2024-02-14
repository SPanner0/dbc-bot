use std::{fs::File, io::Read};

use crate::Error;
use poise::serenity_prelude::json::Value;
use reqwest;

pub enum APIResult {
    Successful(Value),
    NotFound(u16),
    APIError(u16),
}
fn get_player(player_tag: &str) -> String {
    format!("https://api.brawlstars.com/v1/players/%23{}", player_tag)
}

fn get_battle_log(player_tag: &str) -> String {
    format!(
        "https://api.brawlstars.com/v1/players/%23{}/battlelog",
        player_tag
    )
}

pub async fn request(option: &str, tag: &str) -> Result<APIResult, Error> {
    let proper_tag = match tag.starts_with('#') {
        true => &tag[1..],
        false => tag,
    };
    let endpoint = match option {
        "player" => get_player(proper_tag),
        "battle_log" => get_battle_log(proper_tag),
        _ => unreachable!("Invalid option"),
    };

    let token = std::env::var("BRAWL_STARS_TOKEN").expect("Brawl Stars API token not found.");
    let mut buf = Vec::new();
    File::open("cacert.pem")?.read_to_end(&mut buf)?;
    let cert = reqwest::Certificate::from_pem(&buf)?;
    let builder = reqwest::Client::builder().add_root_certificate(cert);
    
    let response = builder.build().unwrap()
        .get(endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if response.status().is_success() {
        let data: Value = response.json().await?;
        Ok(APIResult::Successful(data))
    } else if response.status().is_client_error() {
        Ok(APIResult::NotFound(response.status().as_u16()))
    } else {
        Ok(APIResult::APIError(response.status().as_u16()))
    }
}
