use crate::bracket_tournament::config::get_config;
use crate::bracket_tournament::region::Region;
use crate::database_utils::remove::remove_player;
use crate::misc::CustomError;
use crate::{Context, Error};
use futures::StreamExt;
use mongodb::bson::Document;
use poise::serenity_prelude::ButtonStyle;
use poise::ReplyHandle;
use std::ops::Deref;
use tracing::info;
const REGISTRATION_TIME: u64 = 120;
async fn display_deregister_menu(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |b| {
        b.components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.label("Deregister")
                        .style(ButtonStyle::Danger)
                        .custom_id("deregister")
                })
            })
        })
        .embed(|e| {
            e.title("Deregisteration").description(format!(
                "Are you sure you want to deregister from the tournament?"
            ))
        })
    })
    .await?;
    Ok(())
}

pub async fn deregister_menu(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    player: Document,
) -> Result<(), Error> {
    display_deregister_menu(ctx, msg).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(REGISTRATION_TIME));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "deregister" => {
                let region = Region::find_key(player.get_str("region").unwrap()).unwrap();
                remove_player(&ctx, &player).await?;
                remove_role(&ctx, &msg, &get_config(&ctx, &region).await).await?;
                msg.edit(*ctx, |b| {
                    b.components(|c| c)
                        .embed(|e| {
                            e.title("Deregisteration").description(
                                "You have been deregistered from the tournament\nYou can safely dismiss this message."
                            )
                        })
                })
                .await?;
                break;
            }
            _ => {
                unreachable!("This should never happen!")
            }
        }
    }

    Ok(())
}

async fn remove_role(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    config: &Document,
) -> Result<(), Error> {
    let role_id = config
        .get("role")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let mut member = match ctx.author_member().await {
        Some(m) => m.deref().to_owned(),
        None => {
            let user = *ctx.author().id.as_u64();
            msg.edit(*ctx, |s| {
                s.content("Removing role failed! Please contact Host or Moderators for this issue")
            })
            .await?;
            info!("Failed to assign role for <@{}>", user);
            return Err(Box::new(CustomError(format!(
                "Failed to assign role for <@{}>",
                user
            ))));
        }
    };
    match member.remove_role((*ctx).http(), role_id).await {
        Ok(_) => Ok(()),
        Err(_) => {
            let user = *ctx.author().id.as_u64();
            msg.edit(*ctx, |s| {
                s.content("Removing role failed! Please contact Host or Moderators for this issue")
            })
            .await?;
            info!("Failed to remove role from <@{}>", user);
            Ok(())
        }
    }
}
