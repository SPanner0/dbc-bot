use mongodb::bson::{doc, Document};
use poise::serenity_prelude::User;
use poise::ReplyHandle;
use tracing::{error, span, Level};
use crate::{database::find::find_player_by_discord_id, discord::{checks::is_host, prompt::prompt}, players::view::view_info};
use crate::{
    database::find::find_tag,
    discord::role::{get_region_from_role, get_roles_from_user},
    Context, Error,
};
/// Lookup player by tag or user
#[poise::command(slash_command, guild_only, check = "is_host")]
pub async fn lookup_player(
    ctx: Context<'_>,
    player_tag: Option<String>,
    user: Option<User>,
) -> Result<(), Error> {
    span!(Level::INFO, "lookup_player", player_tag);
    let msg = ctx.send(|s|{
        s.embed(|e|{
            e.title("Looking up player")
        })
    }).await?;
    // We probably don't need this. I'll give it another look later. - Doof
    match (player_tag, user){
        (Some(tag), None) => {
            if let Some(player) = find_tag(&ctx, &tag).await{
                return view_info(&ctx, &msg, player).await;  
            } else {
                return prompt(
                    &ctx, 
                    &msg, 
                    "Cannot find player with this tag", 
                    "Unable to find this tag from any regions!", 
                    None, 
                    Some(0xFF0000)
                ).await;
            }

        }
        (None, Some(user)) => {
            match find_by_player_discord_id(&ctx, &msg, user).await{
                Ok(player) => {
                    if let Some(player) = player{
                        return view_info(&ctx, &msg, player).await;
                    } else {
                        return prompt(
                            &ctx, 
                            &msg, 
                            "Cannot find player with this discord id", 
                            "Unable to find this discord id from any regions!", 
                            None, 
                            Some(0xFF0000)
                        ).await;
                    }
                },
                Err(e) => {
                    error!("{e}");
                    return prompt(
                        &ctx, 
                        &msg, 
                        "Cannot find player with this discord id", 
                        "Unable to find this discord id from any regions!", 
                        None, 
                        Some(0xFF0000)
                    ).await;
                }
                
            }
        }
        (None, None) => {
            return prompt(
                &ctx,
                &msg,
                "Cannot search for player",
                "Please provide either a player tag or a discord user to search",
                None,
                Some(0xFF0000),
            ).await;
        }
        (Some(_), Some(_)) => {
            return prompt(
                &ctx,
                &msg,
                "The developers are lazy to handle this case",
                "Why would you do this to us :c. One parameter is enough!",
                None,
                Some(0xFF0000),
            ).await;
        }
    }
}

async fn find_by_player_discord_id(ctx: &Context<'_>, msg: &ReplyHandle<'_>, user: User) -> Result<Option<Document>, Error>{
    let user_id = user.id.0;
            let roles = match get_roles_from_user(&ctx, Some(&user)).await {
                Some(roles) => roles,
                None => {
                    ctx.send(|s| {
                        s.reply(true)
                            .ephemeral(true)
                            .embed(|e| e.title("Failed to get user roles"))
                    })
                    .await?;
                    return Err("Failed to get user roles".into());
                }
            };
            let region = match get_region_from_role(&ctx, roles).await {
                Some(region) => region,
                None => {
                    msg.edit(*ctx,|s| {
                        s.reply(true)
                            .ephemeral(true)
                            .embed(|e| e.title("Failed to get user region"))
                    })
                    .await?;
                    return Err("Failed to get user region".into());
                }
            };
            find_player_by_discord_id(ctx, &region, user_id, "Players".to_string()).await
            
 
        }
    