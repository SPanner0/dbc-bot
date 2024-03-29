use crate::brawlstars::api::{self, APIResult};
use crate::database::battle::battle_happened;
use crate::database::config::get_config;
use crate::database::find::{
    find_enemy_by_match_id_and_self_tag, find_round_from_config, find_self_by_discord_id,
    is_disqualified, is_mannequin,
};
use crate::database::update::update_match_id;
use crate::database::update::update_result;
use crate::discord::prompt::prompt;
use crate::discord::role::remove_role;
use crate::{Context, Error};
use dbc_bot::{QuoteStripper, Region};
use mongodb::bson::Document;
use mongodb::Collection;
use poise::serenity_prelude::{ChannelId, UserId};
use poise::ReplyHandle;
use tracing::error;

const HAMSTER_VIOLIN_MEME: &str =
    "https://tenor.com/view/sad-hamster-meme-violin-gif-17930564980222230194";

pub async fn submit_result(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    prompt(
        ctx,
        msg,
        "Submitting result...",
        "Please wait while we are submitting your result...",
        None,
        Some(0xFFFF00),
    )
    .await?;
    let round = find_round_from_config(&get_config(ctx, region).await);
    //Check if the user is in the tournament
    let caller = match find_self_by_discord_id(ctx, round).await.unwrap() {
        Some(caller) => caller,
        None => {
            return prompt(
                ctx,
                msg,
                "Sorry, you are not in the tournament!",
                "You have to be in a tournament to use this command!",
                Some(HAMSTER_VIOLIN_MEME),
                Some(0xFF0000),
            )
            .await;
        }
    };
    let region = Region::find_key(
        caller
            .get("region")
            .unwrap()
            .to_string()
            .strip_quote()
            .as_str(),
    )
    .unwrap();

    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(ctx, &region).await;
    let channel = config
        .get("channel")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let channel_to_announce = ChannelId(channel);

    //Get player document via their discord_id
    let match_id: i32 = caller.get_i32("match_id").unwrap();
    let caller_tag = caller.get_str("tag").unwrap();

    let mode = config.get_str("mode").unwrap();
    let map = config.get_str("map").unwrap_or("Any");
    let round_name = find_round_from_config(&config);
    let current_round: Collection<Document> = database.collection(&round_name);
    let round = config.get("round").unwrap().as_i32().unwrap();
    let caller = match battle_happened(ctx, caller_tag, &current_round, msg).await? {
        Some(caller) => caller, // Battle did not happen yet
        None => return Ok(()),  // Battle already happened
    };
    let enemy =
        find_enemy_by_match_id_and_self_tag(ctx, &region, &round_name, &match_id, caller_tag)
            .await
            .unwrap();
    if is_mannequin(&enemy) || is_disqualified(&enemy) {
        update_result(ctx, &region, &round_name, &caller, &enemy, None).await?;
        let m = channel_to_announce
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title("Result is here!")
                        .thumbnail(format!(
                            "https://cdn-old.brawlify.com/profile/{}.png",
                            caller.get_i64("icon").unwrap_or(28000000)
                        ))
                        .description(format!(
                        "Congratulations! <@{}> ({}-{}) has won round {} and proceeds to round {}!",
                        caller.get_str("discord_id").unwrap(),
                        caller.get_str("name").unwrap(),
                        caller.get_str("tag").unwrap(),
                        round,
                        round + 1
                    ))
                        .color(0xFFFF00)
                        .timestamp(ctx.created_at())
                })
            })
            .await?;
        msg.edit(*ctx, |s|{
            s.embed(|e|{
                e.title("Bye... See you next round")
                .description(format!("Congratulation, you advance to next round!\nCheck out your result at [here]({})", m.link()))
                .color(0xFFFF00)
                .footer(|f| f.text("According to Dictionary.com, in a tournament, a bye is the preferential status of a player or team not paired with a competitor in an early round and thus automatically advanced to play in the next round."))
            })
        }).await?;

        // update_bracket(ctx, None).await?;
        return Ok(());
    }

    // let bracket_msg_id = config.get_str("bracket_message_id").unwrap();
    // let bracket_chn_id = config.get_str("bracket_channel").unwrap();
    // let server_id = ctx.guild_id().unwrap().0;

    match get_result(mode, map, caller, enemy).await {
        Some(players) => {
            let (winner, defeated) = players;
            if round < config.get("total").unwrap().as_i32().unwrap() {
                update_result(ctx, &region, &round_name, &winner, &defeated, None).await?;
                let defeated_user = UserId(
                    defeated
                        .get_str("discord_id")
                        .unwrap_or("0")
                        .parse::<u64>()?,
                )
                .to_user(ctx.http())
                .await?;
                if let Err(e) = remove_role(ctx, &defeated_user, &region).await {
                    error!("{e}");
                }
                // update_bracket(ctx, None).await?;
                let m = channel_to_announce
                    .send_message(ctx, |m| {
                        m.embed(|e| {
                            e.title("Result is here!")
                            .thumbnail(format!(
                                "https://cdn-old.brawlify.com/profile/{}.png",
                                winner.get_i64("icon").unwrap_or(28000000)
                            ))
                                .description(format!(
                                    r#"Congratulations! <@{}> ({}-{}) has won round {} and proceeds to round {}!"#,
                                    winner.get_str("discord_id").unwrap(),
                                    winner.get_str("name").unwrap(),
                                    winner.get_str("tag").unwrap(),
                                    round,
                                    round + 1 // guild = server_id,
                                              // chn = bracket_chn_id,
                                              // msg_id = bracket_msg_id
                                ))
                                .color(0xFFFF00)
                                .timestamp(ctx.created_at())
                        })
                    })
                    .await?;
                msg.edit(*ctx, |s| {
                    s.embed(|e| {
                        e.title("Result is here!")
                            .description(format!(
                                r#"Result is submitted [here]({})"#,
                                m.link() // guild = server_id,
                                         // chn = bracket_chn_id,
                                         // msg_id = bracket_msg_id
                            ))
                            .color(0xFFFF00)
                    })
                    .components(|c| c)
                })
                .await?;
            } else {
                update_result(ctx, &region, &round_name, &winner, &defeated, None).await?;
                // update_bracket(ctx, None).await?;
                let m = channel_to_announce
                    .send_message(ctx, |m| {
                        m.embed(|e| {
                            e.title("Result is here!").description(format!(
                                "CONGRATULATIONS! {}({}) IS THE TOURNAMENT CHAMPION!",
                                winner.get_str("name").unwrap(),
                                winner.get_str("tag").unwrap()
                            ))
                        })
                    })
                    .await?;
                msg.edit(*ctx, |s| {
                    s.embed(|e| {
                        e.title("Result is here!")
                            .thumbnail(format!(
                                "https://cdn-old.brawlify.com/profile/{}.png",
                                winner.get_i64("icon").unwrap_or(28000000)
                            ))
                            .description(format!(
                                "CONGRATULATIONS! <@{}>({}-{}) IS THE TOURNAMENT CHAMPION!\n
Your result is shown here [here]({})!",
                                winner.get_str("discord_id").unwrap(),
                                winner.get_str("name").unwrap(),
                                winner.get_str("tag").unwrap(),
                                m.link()
                            ))
                            .color(0xFFFF00)
                    })
                    .components(|c| c)
                })
                .await?;
            }
        }
        None => {
            prompt(
                ctx,
                msg,
                "There are not enough results yet!",
                format!(
                    r#"As the result is recorded nearly in real-time, please try again later.
It may take up to 30 seconds for a new battle to appear in the battle log!
In the meantime, please make sure that all of the recent battles satisfy these conditions: 
- ⚔️ Mode: {mode}
- 🗺️ Map: {map}
- 🧑‍🤝‍🧑 Friendly room
- 🤖 Turn OFF all bots"#,
                    mode = config.get_str("mode").unwrap_or("Any"),
                    map = config.get_str("map").unwrap_or("Any")
                ),
                None,
                Some(0xFFFF00),
            )
            .await?;
        }
    }
    Ok(())
}

async fn get_result(
    mode: &str,
    map: &str,
    caller: Document,
    enemy: Document,
) -> Option<(Document, Document)> {
    let caller_tag = caller.get("tag").unwrap().as_str().unwrap();
    let enemy_tag = enemy.get("tag").unwrap().as_str().unwrap();
    let logs = match api::request("battle_log", caller_tag).await {
        Ok(APIResult::Successful(battle_log)) => battle_log["items"].as_array().unwrap().clone(),
        Ok(APIResult::APIError(_)) => return None,
        Ok(APIResult::NotFound(_)) | Err(_) => return None,
    };
    let mut results: Vec<String> = vec![];

    for log in logs.iter() {
        if !log_check(log, mode, map) {
            continue;
        }

        let player1 = log["battle"]["teams"][0][0]["tag"].as_str().unwrap();
        let player2 = log["battle"]["teams"][1][0]["tag"].as_str().unwrap();
        if (compare_tag(caller_tag, player1) || compare_tag(caller_tag, player2))
            && (compare_tag(enemy_tag, player1) || compare_tag(enemy_tag, player2))
        {
            results.push(log["battle"]["result"].as_str().unwrap().to_string());
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
            Some(true) => Some((caller, enemy)),
            Some(false) => Some((enemy, caller)),
            None => None,
        }
    } else {
        None
    }
}

fn compare_tag(s1: &str, s2: &str) -> bool {
    s1.chars()
        .zip(s2.chars())
        .all(|(c1, c2)| c1 == c2 || (c1 == 'O' && c2 == '0') || (c1 == '0' && c2 == 'O'))
        && s1.len() == s2.len()
}

fn compare_strings(str1: &str, str2: &str) -> bool {
    // Remove punctuation and convert to lowercase
    let str1_normalized = str1
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();

    let str2_normalized = str2
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    str1_normalized == str2_normalized
}

fn log_check(log: &serde_json::Value, mode: &str, map: &str) -> bool {
    // info!("{:?}", log); // Debugging purposes
    match log["event"]["mode"].as_str() {
        Some(m) => {
            if !compare_strings(m, mode) {
                return false;
            }
        }
        None => return false,
    };
    match log["battle"]["type"].as_str() {
        Some(t) => {
            if !compare_strings(t, "friendly") {
                return false;
            }
        }
        None => return false,
    }
    match log["event"]["map"].as_str() {
        Some(m) => {
            if map != "Any" && !compare_strings(m, map) {
                return false;
            }
        }
        None => return false,
    };
    match log["battle"]["teams"][0].as_array() {
        Some(t) => {
            if t.len() > 1 {
                return false;
            }
        }
        None => return false,
    }

    match log["battle"]["teams"][1].as_array() {
        Some(t) => {
            if t.len() > 1 {
                return false;
            }
        }
        None => return false,
    }
    true
}
