use crate::brawlstars::{api::request, api::APIResult, player::stat};
use crate::database::config::get_config;
use crate::database::find::{find_player_by_discord_id, find_round_from_config};
use crate::database::open::tournament;
use crate::discord::prompt::prompt;
use crate::discord::role::{get_region_from_role, get_roles_from_user};
use crate::players::tournament::view2::view_opponent;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use tracing::info;

#[poise::command(context_menu_command = "Player information", guild_only)]
pub async fn get_individual_player_data(
    ctx: Context<'_>,
    user: serenity::User,
) -> Result<(), Error> {
    info!("Getting participant data");
    ctx.defer_ephemeral().await?;
    let msg = ctx
        .send(|s| s.content("Getting player info...").reply(true))
        .await?;
    let roles = get_roles_from_user(&ctx, Some(&user)).await?;
    let region = match get_region_from_role(&ctx, roles).await {
        Some(region) => region,
        None => {
            return prompt(
                &ctx,
                &msg,
                "Failed to fetch user data due to lacking of region role",
                "Either this member is not in the tournament or the region role is not assigned",
                None,
                None,
            )
            .await;
        }
    };
    let id: u64 = user.id.into();
    let round = find_round_from_config(&get_config(&ctx, &region).await);
    let player_from_db = match find_player_by_discord_id(&ctx, &region, id, round).await {
        Ok(player) => match player {
            Some(p) => p,
            None => {
                return prompt(
                    &ctx,
                    &msg,
                    "404 not found",
                    "Player not found in the database",
                    None,
                    None,
                )
                .await;
            }
        },
        Err(_) => {
            return prompt(
                &ctx,
                &msg,
                "Error accessing database",
                "Please try again later",
                None,
                None,
            )
            .await;
        }
    };
    let player = request("player", player_from_db.get_str("tag").unwrap()).await?;
    match player {
        APIResult::Successful(p) => stat(&ctx, &msg, &p, &region, Some(&player_from_db)).await,
        APIResult::NotFound(_) => {
            prompt(
                &ctx,
                &msg,
                "Could not find player from API",
                "Please make sure the player tag is valid",
                None,
                None,
            )
            .await
        }
        APIResult::APIError(_) => {
            prompt(
                &ctx,
                &msg,
                "500: Internal Server Error from",
                "Unable to fetch player data from Brawl Stars API",
                None,
                None,
            )
            .await
        }
    }
}

#[poise::command(context_menu_command = "View battle", guild_only)]
pub async fn view_battle(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let msg = ctx
        .send(|s| {
            s.reply(true)
                .ephemeral(true)
                .embed(|e| e.title("Getting battle from this player..."))
        })
        .await?;

    let roles = get_roles_from_user(&ctx, Some(&user)).await?;
    let region = match get_region_from_role(&ctx, roles).await {
        Some(region) => region,
        None => {
            return prompt(
                &ctx,
                &msg,
                "Failed to fetch user data due to lacking of region role",
                "Either this member is not in the tournament or the region role is not assigned",
                None,
                None,
            )
            .await;
        }
    };
    if !tournament(&ctx, &region).await {
        return prompt(
            &ctx,
            &msg,
            "Tournament is not open!",
            "The tournament is not open yet! Please run this command again when the tournament is open!",
            None,
            None,
        )
        .await;
    }

    let round = find_round_from_config(&get_config(&ctx, &region).await);
    let user_doc = match find_player_by_discord_id(&ctx, &region, user.id.into(), round).await {
        Ok(user) => match user {
            Some(u) => u,
            None => {
                return prompt(
                    &ctx,
                    &msg,
                    "Not found",
                    "Player not found in the database",
                    None,
                    None,
                )
                .await;
            }
        },
        Err(_) => {
            return prompt(
                &ctx,
                &msg,
                "Error accessing database",
                "Please try again later",
                None,
                None,
            )
            .await;
        }
    };
    view_opponent(&ctx, &msg, &region, user_doc).await?;
    Ok(())
}
