use crate::database::battle::battle_happened;
use crate::database::config::get_config;
use crate::database::find::{
    find_enemy_by_match_id_and_self_tag, find_round_from_config, find_self_by_discord_id,
    is_mannequin,
};
use crate::discord::prompt::prompt;
use crate::visual::pre_battle::get_image;
use crate::{Context, Error};
use dbc_bot::{QuoteStripper, Region};
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use poise::{serenity_prelude as serenity, ReplyHandle};
use tracing::info;

/// View your opponent
pub async fn view_opponent(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    prompt(
        ctx,
        msg,
        "Getting your opponent...",
        "<a:loading:1187839622680690689> Searching for your opponent...",
        None,
        Some(0xFFFF00),
    )
    .await?;
    let round = find_round_from_config(&get_config(ctx, region).await);
    let caller = match find_self_by_discord_id(ctx, round).await.unwrap() {
        Some(caller) => caller,
        None => {
            msg.edit(*ctx, |s| {
                s.embed(|e| {
                    e.title("You are not in the tournament!")
                        .description("Sorry, you are not in the tournament to use this command!")
                })
            })
            .await?;
            return Ok(());
        }
    };

    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let config = get_config(ctx, region).await;
    //Get player document via their discord_id
    let match_id: i32 = caller.get_i32("match_id").unwrap();
    let caller_tag = caller.get_str("tag").unwrap();

    let current_round: Collection<Document> =
        database.collection(find_round_from_config(&config).as_str());
    let round = config.get_i32("round").unwrap();
    let caller = match battle_happened(ctx, caller_tag, current_round, msg).await? {
        Some(caller) => caller, // Battle did not happen yet
        None => return Ok(()),  // Battle already happened
    };
    let enemy =
        match find_enemy_by_match_id_and_self_tag(ctx, region, &round, &match_id, caller_tag).await
        {
            Some(enemy) => {
                if is_mannequin(&enemy) {
                    msg.edit(*ctx, |s| {
                        s.embed(|e| {
                            e.title("Congratulations! You are the bye player for this round!")
                                .description("Please run the bot again to submit the result!")
                        })
                    })
                    .await?;
                    return Ok(());
                } else {
                    enemy
                }
            }
            None => {
                msg.edit(*ctx, |s| {
                    s.embed(|e| {
                        e.title("An error occurred!")
                            .description("Please run this command later.")
                    })
                })
                .await?;
                return Ok(());
            }
        };

    let prebattle = match get_image(&caller, &enemy, &config).await {
        Ok(prebattle) => prebattle,
        Err(e) => {
            info!("{e}");
            prompt(
                ctx,
                msg,
                "An error occurred!",
                "An error occurred while getting enemy. Please notify to the Host.",
                None,
                Some(0xFF0000),
            )
            .await?;
            return Err(e);
        }
    };
    let attachment = serenity::model::channel::AttachmentType::Bytes {
        data: prebattle.into(),
        filename: "pre_battle.png".to_string(),
    };
    msg.edit(*ctx,|s| {
        s
            .embed(|e| {
                e.title("**DISCORD BRAWL CUP TOURNAMENT**")
                    .description(format!(
r#"# Round {round} - Match {match_id}
**<@{}> vs. <@{}>**
**🗣️ Before you start:**
Plan with your opponent to schedule at least 2 CONSECUTIVE battles.
**⚙️ During the battle:**
- Set up a friendly mode.
- Mode: {}.
- Map: {}.
- Turn off all bots.
**🗒️ After the battle:**
- Wait for 30s. 
- Run this bot again to submit the result.
**⚠️ Note:**
- Only the EARLIEST determinable number of matches with the opponent is considered once you submit your results.
- Due to limitations, only up to 25 battles are viewable, so please submit the result as soon as possible!
# Good luck!"#, 
                        caller.get_str("discord_id").unwrap(),
                        enemy.get_str("discord_id").unwrap(),
                        config.get_str("mode").unwrap(),
                        config.get_str("map").unwrap_or("Any")

                    )
                    )
            })
            .attachment(attachment)
            .components(|c|c)
    })
    .await?;
    Ok(())
}

///View list of roles as manager of the tournament
pub async fn view_managers(ctx: &Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let database = &ctx.data().database.general;
    let mut list: Vec<String> = vec![];
    let mut managers = database
        .collection::<Document>("Managers")
        .find(doc! {"guild_id": &guild_id}, None)
        .await?;
    while let Some(manager) = managers.try_next().await? {
        let role_id = manager.get("role_id").unwrap().to_string().strip_quote();
        list.push(role_id);
    }
    let role_msg = list
        .iter()
        .map(|role| format!("<@&{}>", role))
        .collect::<Vec<String>>()
        .join(", ");
    ctx.send(|s| {
        s.reply(true).ephemeral(true).embed(|e| {
            e.title("**These following roles have permission to run manager-only commands: **")
                .description(role_msg)
        })
    })
    .await?;
    Ok(())
}
