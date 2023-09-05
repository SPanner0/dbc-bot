use crate::bracket_tournament::{
    api, assign_match_id::update_match_id, config::get_config, region, update_battle::update_battle,
};
use crate::database_utils::{
    find_discord_id::find_discord_id,
    find_enemy::{find_enemy, is_mannequin},
};
use crate::misc::QuoteStripper;
use crate::{Context, Error};
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use poise::serenity_prelude::json::Value;

const MODE: &str = "wipeout";

///Once the match ends, please run this command to update the result.
#[poise::command(slash_command, guild_only)]
pub async fn submit(ctx: Context<'_>) -> Result<(), Error> {
    //Check if the user is in the tournament
    let caller = match find_discord_id(&ctx, None).await {
        Some(caller) => caller,
        None => {
            ctx.send(|s| {
                s.reply(true).ephemeral(false).embed(|e| {
                    e.title("You are not in the tournament!")
                        .description("Sorry, you are not in the tournament to use this command!")
                })
            })
            .await?;
            return Ok(());
        }
    };
    //Get player document via their discord_id
    let match_id: i32 = (caller.get("match_id").unwrap()).as_i32().unwrap();
    let caller_tag = caller.get("tag").unwrap().to_string().strip_quote();
    let region = region::Region::find_key(
        caller
            .get("region")
            .unwrap()
            .to_string()
            .strip_quote()
            .as_str(),
    )
    .unwrap();

    //Check if the user has already submitted the result or not yet disqualified
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let round = get_config(database)
        .await
        .get("round")
        .unwrap()
        .as_i32()
        .unwrap();
    let current_round: Collection<Document> =
        database.collection(format!("Round {}", round).as_str());
    let caller = match current_round
        .find_one(doc! {"tag": &caller_tag}, None)
        .await
    {
        Ok(Some(player)) => {
            if player.get("battle").unwrap().as_bool().unwrap() {
                ctx.send(|s| {
                    s.reply(true)
                        .ephemeral(false)
                        .embed(|e| {
                            e.title("You have already submitted the result!")
                                .description("You have already submitted the result! If you think this is a mistake, please contact the moderators!")
                        })
                }).await?;
                return Ok(());
            } else {
                player
            }
        }
        Ok(None) => {
            ctx.send(|s| {
                s.reply(true).ephemeral(false).embed(|e| {
                    e.title("You are not in this round!")
                        .description("Oops! Better luck next time")
                })
            })
            .await?;
            return Ok(());
        }
        Err(_) => {
            ctx.send(|s| {
                s.reply(true).ephemeral(false).embed(|e| {
                    e.title("An error pops up!")
                        .description("Please run this command later!")
                })
            })
            .await?;
            return Ok(());
        }
    };

    let enemy = find_enemy(&ctx, &region, &round, &match_id, &caller_tag)
        .await
        .unwrap();
    if is_mannequin(&enemy) {
        let round2 = database.collection(format!("Round {}", round + 1).as_str());
        round2.insert_one(update_match_id(caller), None).await?;
        ctx.send(|s| {
            s.reply(true).ephemeral(false).embed(|e| {
                e.title("Bye! See you next... round!").description(
                    "You have been automatically advanced to the next round due to bye!",
                )
            })
        })
        .await?;
        update_battle(database, round, match_id).await?;
        return Ok(());
    }

    match get_result(caller, enemy).await {
        Some(winner) => {
            let next_round: Collection<Document> =
                database.collection(format!("Round {}", round + 1).as_str());
            next_round
                .insert_one(update_match_id(winner.clone()), None)
                .await?;
            update_battle(database, round, match_id).await?;
            ctx.send(|s| {
                s.reply(true).ephemeral(false).embed(|e| {
                    e.title("Result is here!").description(format!(
                        "{}({}) has won this round! You are going to next round!",
                        winner.get("name").unwrap().to_string().strip_quote(),
                        winner.get("tag").unwrap().to_string().strip_quote()
                    ))
                })
            })
            .await?;
            return Ok(());
        }
        None => {
            ctx.send(|s| {
                s.reply(true)
                    .ephemeral(false)
                    .embed(|e| {
                        e.title("No battle logs found (yet?)")
                            .description("As the result is recorded nearly in real-time, please try again later. It may take up to 30 minutes for a new battle to appear in the battlelog")
                    })
            })
            .await?;
            return Ok(());
        }
    };
}

async fn get_result(caller: Document, enemy: Document) -> Option<Document> {
    let caller_tag = caller.get("tag").unwrap().to_string().strip_quote();
    let enemy_tag = enemy.get("tag").unwrap().to_string().strip_quote();
    let endpoint = api::get_api_link("battle_log", &caller_tag);
    let raw_logs = api::request(&endpoint).await.unwrap();
    let logs: &Vec<Value> = raw_logs["items"].as_array().unwrap();
    let mut results: Vec<String> = vec![];

    for log in logs.clone() {
        let mode = log["event"]["mode"].to_string().strip_quote();
        let player1 = log["battle"]["teams"][0][0]["tag"]
            .to_string()
            .strip_quote();
        let player2 = log["battle"]["teams"][1][0]["tag"]
            .to_string()
            .strip_quote();
        if mode == *MODE
            && (caller_tag == player1 || caller_tag == player2)
            && (enemy_tag == player1 || enemy_tag == player2)
        {
            println!("Found the log");
            results.push(log["battle"]["result"].to_string().strip_quote());
        }
    }
    //If there are more than 1 result (best of 2), then we need to check the time
    if results.len() > 1 {
        let mut is_victory = true;
        let mut count_victory = 0;
        let mut count_defeat = 0;

        for result in results.iter().rev() {
            match (*result).strip_quote().as_str() {
                "defeat" => {
                    count_defeat += 1;
                    if count_defeat == 2 || count_victory < 2 {
                        is_victory = false;
                        println!("is_victory = false");
                        break;
                    }
                }
                "victory" => {
                    count_victory += 1;
                    if count_defeat == 2 || count_victory < 2 {
                        println!("is_victory = false");
                        is_victory = false;
                        break;
                    }
                }
                _ => {} // Handle other cases if needed
            }
        }
        if is_victory {
            Some(caller)
        } else {
            Some(enemy)
        }
    } else {
        None
    }
}
