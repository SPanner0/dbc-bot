use std::io::Cursor;

use crate::{
    bracket_tournament::{config::get_config, region},
    database_utils::{
        battle::battle_happened,
        find::{find_player, find_enemy, find_round, is_mannequin},
        open::tournament,
    },
    misc::QuoteStripper,
    visual::pre_battle::generate_pre_battle_img,
    Context, Error,
};

use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use poise::serenity_prelude as serenity;

/// View your opponent
pub async fn view_opponent(ctx: &Context<'_>) -> Result<(), Error> {
    let msg = ctx
        .send(|s| {
            s.ephemeral(true)
                .reply(true)
                .content("Getting your opponent...")
        })
        .await?;
    let caller = match find_player(&ctx).await.unwrap() {
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

    //Checking if the tournament has started
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

    let current_round: Collection<Document> = database.collection(find_round(&config).as_str());
    let round = config.get("round").unwrap().as_i32().unwrap();
    let caller = match battle_happened(&ctx, &caller_tag, current_round, &msg).await? {
        Some(caller) => caller, // Battle did not happen yet
        None => return Ok(()),  // Battle already happened
    };
    let enemy = match find_enemy(&ctx, &region, &round, &match_id, &caller_tag).await {
        Some(enemy) => {
            if is_mannequin(&enemy) {
                msg.edit(*ctx, |s|
                    s.embed(|e| {
                            e.title("Congratulations! You are the bye player for this round!")
                                .description("Please run </submit-result:1148650981555441894> to be in the next round!")
                        })
                )
                .await?;
                return Ok(());
            } else {
                enemy
            }
        }
        None => {
            ctx.send(|s| {
                s.reply(true).ephemeral(true).embed(|e| {
                    e.title("An error occurred!")
                        .description("Please run this command later.")
                })
            })
            .await?;
            return Ok(());
        }
    };

    let image = generate_pre_battle_img(&caller, &enemy, &config)
        .await
        .unwrap();
    let mut bytes: Vec<u8> = Vec::new();
    image.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)?;
    let attachment = serenity::model::channel::AttachmentType::Bytes {
        data: bytes.into(),
        filename: "pre_battle.png".to_string(),
    };
    let message = "Please plan with your opponent to schedule at least 2 games in the friendly battle mode (please turn off all bots).";
    ctx.send(|s| {
        s.reply(true)
            .ephemeral(true)
            .embed(|e| {
                e.title("**DISCORD BRAWL CUP TOURNAMENT**")
                    .description(format!(
                        "Round {} - Match {}\n<@{}> vs. <@{}\n{}>",
                        round,
                        match_id,
                        caller.get("discord_id").unwrap().as_str().unwrap(),
                        enemy.get("discord_id").unwrap().as_str().unwrap(),
                        message
                    ))
            })
            .attachment(attachment)
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
