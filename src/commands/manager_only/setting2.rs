use std::collections::HashMap;

use crate::{
    bracket_tournament::{
        config::{get_config, update_round},
        region::Region,
    },
    checks::{tournament_started, user_is_manager},
    database_utils::find_round::get_round,
    misc::QuoteStripper,
    Context, Error,
};
use futures::StreamExt;
use mongodb::{
    bson::{self, doc, Document},
    Database,
};
use poise::{serenity_prelude::Role, ReplyHandle};
use tracing::{info, instrument};

/// Set a role as a manager to access manager-only commands. Only the bot owner can run this.
#[instrument]
#[poise::command(slash_command, guild_only, owners_only, rename = "set-manager")]
pub async fn set_manager(
    ctx: Context<'_>,
    #[description = "Select a role to hold permission to monitor the tournament"] role: Role,
) -> Result<(), Error> {
    info!("Setting manager for {}", role);
    let database = &ctx.data().database.general;
    let guild_id = ctx.guild_id().unwrap().to_string();
    let guild_name = ctx.guild().unwrap().name;
    let role_id = role.id.to_string();
    let role_name = role.name;

    if role_exists(database, &guild_id, &role_id).await? {
        ctx.send(|s| {
            s.ephemeral(true)
                .reply(true)
                .content(format!("{} is already a manager!", &role_name))
        })
        .await?;
    } else {
        let collection = database.collection::<Document>("Managers");
        let new_role: Document = doc! {
            "guild_id": &guild_id,
            "guild_name": &guild_name,
            "role_id": &role_id,
            "role_name": &role_name,
        };
        collection.insert_one(new_role, None).await?;
        ctx.send(|s| {
            s.ephemeral(true)
                .reply(true)
                .content(format!("{} is now a manager!", &role_name))
        })
        .await?;
    };
    Ok(())
}

async fn role_exists(
    database: &Database,
    guild_id: &String,
    role_id: &String,
) -> Result<bool, Error> {
    let collection = database.collection::<Document>("Managers");
    match collection
        .find_one(
            doc! {
                "guild_id": guild_id,
                "role_id": role_id
            },
            None,
        )
        .await
    {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(err) => Err(err.into()),
    }
}
/////////////////////////////////////////////////////////////////
/// Get the current round of the tournament
#[poise::command(slash_command, guild_only)]
pub async fn set_round(
    ctx: Context<'_>,
    #[description = "Select the region"] region: Region,
    #[description = "(Optional) Set the round. By default, without this parameter, the round is increased by 1"]
    round: Option<i32>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let msg = ctx
        .send(|s| {
            s.ephemeral(true)
                .reply(true)
                .content("Setting the round...")
        })
        .await?;
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(database).await;
    println!("Config is got!");
    if !user_is_manager(ctx).await? {
        return Ok(());
    }

    if !tournament_started(&config).await? {
        msg.edit(ctx,|s| {
            s.content("Unable to set the round for the current tournament: the tournament has not started yet!")
        }).await?;
        return Ok(());
    }
    println!("Checking tournament started: DONE");
    if !all_battles_occured(&ctx, &msg, database, &config).await? {
        return Ok(());
    }
    println!("Check all battle occured: DONE");
    match database
        .collection::<Document>("Config")
        .update_one(config, update_round(round), None)
        .await
    {
        Ok(_) => {}
        Err(_) => {
            ctx.say("Error occurred while updating config").await?;
            return Ok(());
        }
    }

    let post_config = get_config(database).await;
    match sort_collection(database, &post_config).await {
        Ok(_) => {}
        Err(_) => {
            ctx.send(|s| {
                s.content("Error occurred while sorting collection")
                    .ephemeral(true)
                    .reply(true)
            })
            .await?;
            return Ok(());
        }
    };
    ctx.send(|s| {
        s.ephemeral(true).reply(true).embed(|e| {
            e.title("Round is set successfully!").description(format!(
                "Round is set! We are at round {}",
                post_config.get("round").unwrap()
            ))
        })
    })
    .await?;
    Ok(())
}

async fn sort_collection(database: &Database, config: &Document) -> Result<(), Error> {
    let round = config.get("round").unwrap();
    let collection = database.collection::<Document>(format!("Round{}", round).as_str());
    let pipeline = vec![doc! {
        "$sort": {
            "match_id": 1
        }
    }];
    collection.aggregate(pipeline, None).await?;
    Ok(())
}

async fn all_battles_occured(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    database: &Database,
    config: &Document,
) -> Result<bool, Error> {
    let round = get_round(config);
    let collection = database.collection::<Document>(round.as_str());
    println!("Round is got!");
    let mut battles = collection
        .find(
            doc! {
                "battle": false
            },
            None,
        )
        .await?;
    println!("Battles is got!");
    if Some(battles.current()).is_none() {
        println!("No battle is left!");
        return Ok(false);
    }
    println!("There are battles left!");
    let mut players: Vec<Document> = Vec::new();

    while let Some(player) = battles.next().await {
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
    let ongoing: Vec<(String, String, bool)> = match_groups
        .values()
        .map(|group| {
            if group.len() == 2 {
                let player1 = &group[0];
                let player2 = &group[1];
                let dis1 = player1.get("discord_id").unwrap().as_str().unwrap();
                let name1 = player1.get("name").unwrap().as_str().unwrap();
                let tag1 = player1.get("tag").unwrap().as_str().unwrap();
                let dis2 = player2.get("discord_id").unwrap().to_string().strip_quote();
                let name2 = player2.get("name").unwrap().as_str().unwrap();
                let tag2 = player2.get("tag").unwrap().to_string().strip_quote();
                (
                    format!("Match {}", player1.get("match_id").unwrap()),
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
        .collect();

    msg.edit(*ctx, |s| {
        s.reply(true).ephemeral(false).embed(|e| {
            e.title("**Unable to start next round due to ongoing battles!**")
                .description(format!(
                    "There are {} matches left to be completed\nNote: <@> means Mannequin",
                    players.len() / 2
                ))
                .fields(ongoing)
        })
    })
    .await?;

    Ok(false)
}
