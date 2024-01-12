use dbc_bot::Region;
use futures::stream::StreamExt;
use mongodb::bson::{self, Document};
use poise::ReplyHandle;
use std::collections::HashMap;
use crate::database::find::find_all_false_battles;
use crate::database::update::update_round_config;
use crate::{Context, Error};
const TIMEOUT: u64 = 300;
pub async fn display_next_round(ctx: &Context<'_>, msg: &ReplyHandle<'_>, region: &Region) -> Result<(), Error> {
    let battles = display_false_battles(ctx, region).await;
    if battles.is_empty() {
        msg.edit(*ctx,|m| {
            m.embed(|e|{
              e.title("All matches are finished!")
              .description("You can safely continue to next round of the tournament!")
            })
            .components(|c|{
              c.create_action_row(|a|{
                a.create_button(|b|{
                  b.label("Next Round")
                  .disabled(false)
                  .custom_id("continue")
                })
              })
            })
        }).await?;
    } else {
      msg.edit(*ctx,|m| {
        m.embed(|e|{
          e.title("Some matches are not finished!")
          .description("Please require the players to finish their matches before continuing to next round!")
          .fields(battles)
        })
        .components(|c|{
          c.create_action_row(|a|{
            a.create_button(|b|{
              b.label("Next Round")
              .disabled(true)
              .custom_id("continue")
            })
          })
        })
    }).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "continue" => {
                mci.defer(&ctx.http()).await?;
                update_round_config(ctx, region).await?;
                return Ok(());
            }
            _ => {}
        }
    }
    }
    Ok(())
}

pub async fn display_false_battles(
    ctx: &Context<'_>,
    region: &Region,
) -> Vec<(String, String, bool)> {
    let mut players = vec![];
    let mut result = find_all_false_battles(ctx, region).await;
    while let Some(player) = result.next().await {
        match player {
            Ok(p) => players.push(p),
            Err(err) => {
                eprintln!("Error reading document: {}", err);
                // Handle the error as needed
            }
        }
    }
    let mut match_groups: HashMap<i32, Vec<&Document>> = HashMap::new();
    for player in &players {
        if let Some(match_id) = player.get("match_id").and_then(bson::Bson::as_i32) {
            match_groups
                .entry(match_id)
                .or_insert(Vec::new())
                .push(player);
        }
    }
    let mut battles: Vec<(String, String, bool)> = match_groups
        .values()
        .map(|group| {
            if group.len() == 2 {
                let player1 = &group[0];
                let player2 = &group[1];
                let dis1 = player1.get_str("discord_id").unwrap_or("").to_string();
                let name1 = player1.get_str("name").unwrap_or("").to_string();
                let tag1 = player1.get_str("tag").unwrap_or("").to_string();
                let dis2 = player2.get_str("discord_id").unwrap_or("").to_string();
                let name2 = player2.get_str("name").unwrap_or("").to_string();
                let tag2 = player2.get_str("tag").unwrap_or("").to_string();
                (
                    format!("Match {}", player1.get_i32("match_id").unwrap()),
                    format!(
                        "<@{}> - <@{}>\n{}({}) - {}({})",
                        dis1, dis2, name1, tag1, name2, tag2
                    ),
                    false,
                )
            } else {
                unreachable!("There should be 2 players in each match!")
            }
        })
        .collect::<Vec<(String, String, bool)>>();
    battles.sort_by(|a, b| a.0.cmp(&b.0));
    battles
}
