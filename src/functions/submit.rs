use crate::bracket_tournament::api::APIResult;
use crate::bracket_tournament::{
    api, match_id::update_match_id, region, update_battle::update_battle,
};
use crate::database_utils::config::get_config;
use crate::database_utils::battle::battle_happened;
use crate::database_utils::find::{find_enemy, find_player, find_round, is_mannequin};
use crate::database_utils::open::tournament;
use crate::misc::QuoteStripper;
use crate::{Context, Error};
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use poise::serenity_prelude::ChannelId;

/// If you are a participant, run this command once you have finished your match round.
///
/// Automatically grabs the user's match result from the game and updates the bracket.

pub async fn submit_result(ctx: &Context<'_>) -> Result<(), Error> {
    let msg = ctx
        .send(|s| {
            s.ephemeral(true)
                .reply(true)
                .content("Checking your match result...")
        })
        .await?;
    //Check if the user is in the tournament
    let caller = match find_player(&ctx).await.unwrap() {
        Some(caller) => caller,
        None => {
            msg.edit(*ctx, |s| {
                s.embed(|e| {
                    e.title("Sorry, you are not in the tournament!")
                        .description("You have to be in a tournament to use this command!")
                })
            })
            .await?;
            return Ok(());
        }
    };
    let region = region::Region::find_key(
        caller
            .get("region")
            .unwrap()
            .to_string()
            .strip_quote()
            .as_str(),
    )
    .unwrap();

    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(&ctx, &region).await;

    if !tournament(&ctx, &region).await {
        msg.edit(*ctx, |s| {
            s.embed(|e| {
                e.title("Tournament has not started yet!").description(
                    "Please wait for the tournament to start before using this command!",
                )
            })
        })
        .await?;
        return Ok(());
    }

    //Get player document via their discord_id
    let match_id: i32 = (caller.get("match_id").unwrap()).as_i32().unwrap();
    let caller_tag = caller.get("tag").unwrap().as_str().unwrap();
    //Check if the user has already submitted the result or not yet disqualified

    let mode = config.get("mode").unwrap().as_str().unwrap();
    // let map = config.get("map").unwrap().as_str().unwrap();
    let current_round: Collection<Document> = database.collection(&find_round(&config).as_str());
    let round = config.get("round").unwrap().as_i32().unwrap();
    let caller = match battle_happened(&ctx, &caller_tag, current_round, &msg).await? {
        Some(caller) => caller, // Battle did not happen yet
        None => return Ok(()),  // Battle already happened
    };
    let enemy = find_enemy(&ctx, &region, &round, &match_id, &caller_tag)
        .await
        .unwrap();
    if is_mannequin(&enemy) {
        let next_round = database.collection(format!("Round {}", round + 1).as_str());
        next_round.insert_one(update_match_id(caller), None).await?;
        msg.edit(*ctx, |s| {
            s.embed(|e| {
                e.title("Bye! See you next... round!").description(
                    "You have been automatically advanced to the next round due to bye!",
                )
            })
        })
        .await?;
        update_battle(database, round, match_id).await?;
        return Ok(());
    }
    println!("{:?}", config);
    let channel = config
        .get("channel")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let channel_to_announce = ChannelId(channel);
    match get_result(mode, caller, enemy).await {
        Some(winner) => {
            if round < config.get("total").unwrap().as_i32().unwrap() {
                let next_round: Collection<Document> =
                    database.collection(format!("Round {}", round + 1).as_str());
                next_round
                    .insert_one(update_match_id(winner.clone()), None)
                    .await?;
                update_battle(database, round, match_id).await?;
                msg.edit(*ctx, |s| {
                    s.embed(|e| {
                        e.title("Result is here!").description(format!(
                            "{}({}) has won this round! You are going to next round!",
                            winner.get("name").unwrap().to_string().strip_quote(),
                            winner.get("tag").unwrap().to_string().strip_quote()
                        ))
                    })
                })
                .await?;
                channel_to_announce
                    .send_message(ctx, |m| {
                        m.embed(|e| {
                            e.title("Result is here!").description(format!(
                                "{}({}) has won this round! You are going to next round!",
                                winner.get("name").unwrap().to_string().strip_quote(),
                                winner.get("tag").unwrap().to_string().strip_quote()
                            ))
                        })
                    })
                    .await?;
            } else {
                msg.edit(*ctx, |s| {
                    s.embed(|e| {
                        e.title("Result is here!").description(format!(
                            "CONGRATULATIONS! {}({}) IS THE TOURNAMENT CHAMPION!",
                            winner.get("name").unwrap().to_string().strip_quote(),
                            winner.get("tag").unwrap().to_string().strip_quote()
                        ))
                    })
                })
                .await?;
                channel_to_announce
                    .send_message(ctx, |m| {
                        m.embed(|e| {
                            e.title("Result is here!").description(format!(
                                "CONGRATULATIONS! {}({}) IS THE TOURNAMENT CHAMPION!",
                                winner.get("name").unwrap().to_string().strip_quote(),
                                winner.get("tag").unwrap().to_string().strip_quote()
                            ))
                        })
                    })
                    .await?;
            }
        }
        None => {
            ctx.send(|s| {
                s.reply(true)
                .ephemeral(true)
                    .embed(|e| {
                        e.title("There are not enough results yet!")
                            .description("As the result is recorded nearly in real-time, please try again later. It may take up to 30 minutes for a new battle to appear in the battlelog")
                    })
            }).await?;
        }
    }
    Ok(())
}

async fn get_result(mode: &str, caller: Document, enemy: Document) -> Option<Document> {
    let caller_tag = caller.get("tag").unwrap().as_str().unwrap();
    let enemy_tag = enemy.get("tag").unwrap().as_str().unwrap();
    let logs = match api::request("battle_log", &caller_tag).await {
        Ok(APIResult::Successful(battle_log)) => Some(battle_log["items"].as_array().unwrap().clone()),
        Ok(APIResult::APIError(_)) => None,
        Ok(APIResult::NotFound(_)) | Err(_) => None,
    };
    let mut results: Vec<String> = vec![];

    for log in logs.unwrap() {
        let mode_log = log["event"]["mode"].as_str().unwrap();
        let player1 = log["battle"]["teams"][0][0]["tag"].as_str().unwrap();
        let player2 = log["battle"]["teams"][1][0]["tag"].as_str().unwrap();
        if mode_log == mode
            && (caller_tag == player1 || caller_tag == player2)
            && (enemy_tag == player1 || enemy_tag == player2)
        {
            results.push(log["battle"]["result"].to_string().strip_quote());
        }
    }
    //If there are more than 1 result (best of 2), then we need to check the time
    if results.len() > 1 {
        let mut is_victory: Option<bool> = None;
        let mut count_victory = 0;
        let mut count_defeat = 0;

        for result in results.iter().rev() {
            match result.as_str() {
                "defeat" => count_defeat += 1,
                "victory" => count_victory += 1,
                _ => {} // Handle other cases if needed
            }

            if count_defeat == 2 && count_victory < 2 {
                is_victory = Some(false);
                break;
            } else if count_victory >= 2 {
                is_victory = Some(true);
                break;
            }
        }
        match is_victory {
            Some(true) => Some(caller),
            Some(false) => Some(enemy),
            None => None,
        }
    } else {
        None
    }
}
