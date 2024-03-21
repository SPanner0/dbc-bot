use super::prompt::prompt;
use crate::host::registration::index::registration_mod_panel;
use crate::host::tournament::index::tournament_mod_panel;
use crate::host::utilities::index::utilities_mod_panel;
use crate::players::registration::deregister::deregister_menu;
use crate::players::registration::register::register_menu;
use crate::players::tournament::ready;
use crate::players::tournament::submit::submit_result;
use crate::players::tournament::view2::{view_managers, view_opponent_wrapper};
use crate::players::view::view_info;
use crate::Context;
use crate::Error;
use dbc_bot::Region;
use futures::StreamExt;
use mongodb::bson::Document;
use poise::serenity_prelude::{ButtonStyle, ReactionType};
use poise::ReplyHandle;

const TIMEOUT: u64 = 300;
/// Displays a registration menu with various options.
/// - `ctx`: Context<'_>.
/// - `msg`: The message to edit.
/// - `register`: Whether to show the register button.
/// - `view`: Whether to show the view button.
/// - `deregister`: Whether to show the deregister button.
/// - `help`: Whether to show the help button.
/// - `player`: The player document.
pub async fn registration_menu(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    register: bool,
    view: bool,
    deregister: bool,
    help: bool,
    player: Option<Document>,
) -> Result<(), Error> {
    msg.edit(*ctx, |e| {
        e.components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| {
                    b.custom_id("register")
                        .disabled(!register)
                        .style(ButtonStyle::Success)
                        .emoji(ReactionType::Unicode("📝".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("personal")
                        .disabled(!view)
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("🤓".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("deregister")
                        .disabled(!deregister)
                        .style(ButtonStyle::Danger)
                        .emoji(ReactionType::Unicode("🚪".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("help")
                        .disabled(!help)
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::Unicode("❓".to_string()))
                })
            })
        })
        .embed(|e| {
            e.title("Registration Menu")
                .description(
                    r#"Below are options:
📝: Register.
🤓: View personal information.
🚪: Deregister.
❓: Help."#,
                )
                .color(0xFFFF00)
        })
    })
    .await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "register" => {
                mci.defer(&ctx.http()).await?;
                return register_menu(ctx, msg).await;
            }
            "deregister" => {
                mci.defer(&ctx.http()).await?;
                return deregister_menu(ctx, msg, player.unwrap()).await;
            }
            "personal" => {
                mci.defer(&ctx.http()).await?;
                return view_info(ctx, msg, player.unwrap()).await;
            }
            "help" => {
                mci.defer(&ctx.http()).await?;
                return prompt(
                  ctx,
                  msg,
                  "This is still under development!", 
                  "This feature is still under development, please be patient!", 
                  Some("https://tenor.com/view/josh-hutcherson-josh-hutcherson-whistle-edit-whistle-2014-meme-gif-1242113167680346055"),
                  None
              ).await;
            }
            _ => {}
        }
    }
    Ok(())
}

pub async fn tournament_menu(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    schedule: bool,
    _managers: bool,
    submit: bool,
    help: bool,
    player: Document,
) -> Result<(), Error> {
    let region = Region::find_key(player.get_str("region").unwrap()).unwrap();
    msg.edit(*ctx, |e| {
        e.components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| {
                    b.custom_id("enemy")
                        .disabled(!schedule)
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("⚔️".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("ready")
                        .style(ButtonStyle::Success)
                        .emoji(ReactionType::Unicode("💪".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("submit")
                        .disabled(!submit)
                        .style(ButtonStyle::Success)
                        .emoji(ReactionType::Unicode("📥".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("ready")
                        .style(ButtonStyle::Success)
                        .emoji(ReactionType::Unicode("💪".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("help")
                        .disabled(!help)
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::Unicode("❓".to_string()))
                })
            })
            // .create_action_row(|r| {
            //     r.create_button(|b| {
            //         b.custom_id("managers")
            //             .disabled(!managers)
            //             .style(ButtonStyle::Danger)
            //             .emoji(ReactionType::Unicode("🛡️".to_string()))
            //     })
            // })
        })
        .embed(|e| {
            e.title("Tournament Menu")
                .description(
                    r#"Below are the available options!
⚔️: Find out who your opponent is for the current round!
💪: Mark your activity!
📥: Submit your result!
👤: View Personal Information
❓: Help.
"#,
                )
                .color(0xFFFF00)
        })
    })
    .await?;
    let resp = msg.clone().into_message().await?;
    let mut cic = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT))
        .build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "enemy" => {
                mci.defer(&ctx.http()).await?;
                return view_opponent_wrapper(ctx, msg, &region).await;
            }
            "managers" => {
                mci.defer(&ctx.http()).await?;
                return view_managers(ctx).await;
            }
            "ready" => {
                mci.defer(&ctx.http()).await?;
                return ready::ready(ctx, msg, &region, player).await;
            }
            "submit" => {
                mci.defer(&ctx.http()).await?;
                return submit_result(ctx, msg, &region).await;
            }

            "view" => {
                mci.defer(&ctx.http()).await?;
                return view_info(ctx, msg, player).await;
            }
            "help" => {
                mci.defer(&ctx.http()).await?;
                return prompt(
                    ctx,
                    msg,
                    "This is still under development!",
                    "This feature is still under development, please be patient!",
                    None,
                    None,
                )
                .await;
            }
            _ => {}
        }
    }
    Ok(())
}

pub async fn mod_menu(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
    _disqualify: bool,
    managers: bool,
    _submit: bool,
    _help: bool,
) -> Result<(), Error> {
    msg.edit(*ctx, |e| {
        e.components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| {
                    b.custom_id("registration")
                        .disabled(!managers)
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("📥".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("tournament")
                        .disabled(!managers)
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("🚩".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("setting")
                        .disabled(!managers)
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("⚙️".to_string()))
                })
            })
        })
        .embed(|e| {
            e.title("Host-only menu").description(format!(
                r#"The following mod menu is set for {}
Below are options:
📥: Registration
- Lets you manage registration status and view all players information.
🚩: Tournament
- Lets you start, end, and manage rounds.
⚙️: Utilities
- Lets you configurate announcements, and bot settings i.e role, channels, game modes, etc.
"#,
                region.full()
            ))
        })
    })
    .await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "registration" => {
                mci.defer(&ctx.http()).await?;
                return registration_mod_panel(ctx, msg, region).await;
            }
            "tournament" => {
                mci.defer(&ctx.http()).await?;
                return tournament_mod_panel(ctx, msg, region).await;
            }
            "setting" => {
                mci.defer(&ctx.http()).await?;
                return utilities_mod_panel(ctx, msg, region).await;
            }
            "help" => {
                mci.defer(&ctx.http()).await?;
                return prompt(
                  ctx,
                  msg,
                  "This is still under development!", 
                  "This feature is still under development, please be patient!", 
                  Some("https://tenor.com/view/josh-hutcherson-josh-hutcherson-whistle-edit-whistle-2014-meme-gif-1242113167680346055"),
                  None
              ).await;
            }
            _ => {}
        }
    }
    Ok(())
}

#[allow(dead_code)]
async fn host_registration_menu(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    registration_menu(ctx, msg, true, true, true, true, None).await
}
