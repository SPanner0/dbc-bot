use crate::database::config::set_config;
use crate::database::find::{find_enemy_by_match_id_and_self_tag, find_self_by_discord_id};
use crate::{Context, Error};
use base64::{engine::general_purpose, Engine as _};
use dbc_bot::{CustomError, QuoteStripper, Region};
use futures::TryStreamExt;
use mongodb::bson::doc;
use std::env;
use std::process::Command;
use std::{io::Read, process::Stdio};
use tracing::{error, info};

pub async fn update_bracket(ctx: &Context<'_>, region: Option<&Region>) -> Result<(), Error> {
    let current_dir = match env::current_dir() {
        Ok(dir) => {
            info!("Current directory: {:?}", dir);
            dir
        }
        Err(e) => {
            error!("Failed to get current directory: {e}");
            return Err(Box::new(CustomError(format!("{e}"))));
        }
    };

    let current_region = match region {
        Some(region) => region.clone(),
        None => {
            let caller = match find_self_by_discord_id(ctx, "Players".to_string())
                .await
                .unwrap()
            {
                Some(caller) => caller,
                None => {
                    info!("Player is not in a tournament, but the function did not return early.");
                    return Err(Box::new(CustomError(
                        "Player is not in a tournament, but the function did not return early."
                            .to_string(),
                    )));
                }
            };
            Region::find_key(caller.get_str("region").unwrap()).unwrap()
        }
    };

    let database = ctx
        .data()
        .database
        .regional_databases
        .get(&current_region)
        .unwrap();
    let collection: mongodb::Collection<mongodb::bson::Document> = database.collection("Config");
    let config = collection.find_one(None, None).await?.unwrap();

    let mut player_data: Vec<(i32, i32, String, String, bool, bool)> = Vec::new();
    let mut match_ids = Vec::new();

    for round_number in 1..=config.get("total").unwrap().as_i32().unwrap() {
        let mut database: mongodb::Cursor<mongodb::bson::Document> = ctx
            .data()
            .database
            .regional_databases
            .get(&current_region)
            .unwrap()
            .collection(format!("Round {}", round_number).as_str())
            .find(None, None)
            .await?;

        while let Some(current_document) = database.try_next().await? {
            let match_id = current_document
                .get("match_id")
                .and_then(|n| n.as_i32())
                .unwrap_or(0);
            if match_ids.contains(&match_id) {
                continue;
            }
            let tag = if let Some(tag) = current_document.get("tag").and_then(|n| n.as_str()) {
                tag
            } else {
                continue;
            };
            match_ids.push(match_id);
            player_data.push((
                round_number,
                match_id,
                current_document
                    .get("name")
                    .map_or(" ".to_string(), |name| name.to_string().strip_quote()),
                (find_enemy_by_match_id_and_self_tag(
                    ctx,
                    &current_region,
                    &round_number,
                    &match_id,
                    tag,
                )
                .await)
                    .map_or(" ".to_string(), |document| {
                        document.get("name").unwrap().to_string().strip_quote()
                    }),
                current_document
                    .get("winner")
                    .map_or(false, |is_winner| is_winner.as_bool().unwrap()),
                (find_enemy_by_match_id_and_self_tag(
                    ctx,
                    &current_region,
                    &round_number,
                    &match_id,
                    tag,
                )
                .await)
                    .map_or(false, |document| {
                        document
                            .get("winner")
                            .map_or(false, |is_winner| is_winner.as_bool().unwrap())
                    }),
            ));
        }
        match_ids.clear();
    }

    let output = Command::new("python")
        .arg("bracket_tournament/bracket_generation.py")
        .arg(current_region.to_string())
        .arg(config.get("total").unwrap().to_string())
        .arg(match player_data.is_empty() {
            true => "1|1| | | | ".to_string(),
            false => player_data.iter().map(|(round, match_id, player1_tag, player2_tag, is_winner1, is_winner2)| format!("{round}|{match_id}|{player1_tag}|{player2_tag}|{is_winner1}|{is_winner2}")).collect::<Vec<String>>().join(",")
        })
        .stdout(Stdio::piped())
        .current_dir(current_dir)
        .spawn()?;

    let mut stdout = output
        .stdout
        .ok_or_else(|| Error::from("Failed to capture Python script output"))?;
    let mut buffer = String::new();
    stdout.read_to_string(&mut buffer)?;

    let image_bytes = general_purpose::STANDARD.decode(buffer.trim_end()).unwrap();
    let attachment = poise::serenity_prelude::AttachmentType::Bytes {
        data: image_bytes.into(),
        filename: "Tournament_Bracket.png".to_string(),
    };

    match config
        .get("bracket_channel")
        .and_then(|v| v.as_str().map(|s| s.parse::<u64>().ok()))
    {
        Some(channel_id) => {
            match config
                .get("bracket_message_id")
                .and_then(|v| v.as_str().map(|s| s.parse::<u64>().ok()))
            {
                Some(bracket_message_id) => {
                    match poise::serenity_prelude::ChannelId(channel_id.unwrap())
                        .edit_message(&ctx, bracket_message_id.unwrap(), |m| {
                            m.attachment(attachment)
                        })
                        .await
                    {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(Error::from(err));
                        }
                    }
                }
                None => {
                    match poise::serenity_prelude::ChannelId(channel_id.unwrap())
                        .send_message(&ctx, |m| m.add_file(attachment))
                        .await
                    {
                        Ok(message) => {
                            match collection
                                .update_one(
                                    doc! {},
                                    set_config(
                                        "bracket_message_id",
                                        Some(message.id.to_string().as_str()),
                                    ),
                                    None,
                                )
                                .await
                            {
                                Ok(_) => {}
                                Err(err) => match err.kind.as_ref() {
                                    mongodb::error::ErrorKind::Command(code) => {
                                        error!("Command error: {:?}", code);
                                    }
                                    mongodb::error::ErrorKind::Write(code) => {
                                        error!("Write error: {:?}", code);
                                    }
                                    _ => {
                                        error!("Error: {:?}", err);
                                    }
                                },
                            }
                        }
                        Err(err) => {
                            return Err(Error::from(err));
                        }
                    }
                } // _ => {
                  //     info!("Failed to retrieve bracket results channel data.");
                  //     return Err(Box::new(CustomError(
                  //         "Failed to retrieve bracket results channel data.".to_string(),
                  //     )));
                  // }
            }
        }
        _ => {
            info!("Failed to retrieve bracket results channel data.");
            return Err(Box::new(CustomError(
                "Failed to retrieve bracket results channel data.".to_string(),
            )));
        }
    };

    Ok(())
}
