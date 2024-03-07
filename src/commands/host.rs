use crate::{discord::menu::mod_menu, Context, Error};
use dbc_bot::Region;
use mongodb::bson::{doc, Document};
use poise::serenity_prelude::RoleId;
use tracing::{error, info};

/// Host all-in-one command
#[poise::command(slash_command, guild_only)]
pub async fn host(
    ctx: Context<'_>,
    #[description = "Select region to configurate"] region: Region,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    match is_host(ctx).await {
        Ok(true) => {}
        Ok(false) => {
            ctx.send(|s| {
                s.ephemeral(true)
                    .reply(true)
                    .content("You don't have permissions to host")
            })
            .await?;
            return Ok(());
        }
        Err(e) => {
            error!("{e}");
            return Ok(());
        }
    }
    let msg = ctx
        .send(|s| {
            s.embed(|e| {
                e.title("Host-only menu")
                    .description(format!(
                        "The following mod menu is set for region: {region}"
                    ))
                    .image("")
            })
        })
        .await?;
    mod_menu(&ctx, &msg, &region, true, true, true, true).await
}

async fn is_host(ctx: Context<'_>) -> Result<bool, Error> {
    let server_id = ctx.guild_id().unwrap().to_string();
    let doc: Document = ctx
        .data()
        .database
        .general
        .collection("Managers")
        .find_one(doc!{"server_id": server_id}, None)
        .await?
        .unwrap();
    let hosts = doc.get_array("role_id").unwrap().to_vec();
    let guild = ctx.guild_id().unwrap();
    for host in hosts.iter() {
        let id = host.as_str().unwrap().parse::<u64>()?;
        info!("Checking {id}");
        let role = RoleId::to_role_cached(RoleId(id), ctx.cache()).unwrap();
        match ctx.author().has_role(ctx.http(), guild, &role).await {
            Ok(true) => return {
                info!("{} is authenticated to host due to the role {}", ctx.author().name, role.name);
                Ok(true)
            },
            Ok(false) => {
                info!("{} doesn't have the role {}", ctx.author().name, role.name);
                continue;
            }
            Err(e) => {
                error!("{e}");
                return Ok(false);
            }
        }
    }
    info!("No permissions to host");
    Ok(false)
}
