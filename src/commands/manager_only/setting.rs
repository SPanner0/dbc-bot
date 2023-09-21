use crate::{
    bracket_tournament::config::set_config,
    bracket_tournament::region::{Mode, Region},
    checks::user_is_manager,
    misc::{CustomError, QuoteStripper},
    Context, Error,
};
use futures::StreamExt;
use mongodb::{
    bson::doc,
    bson::Document,
    Collection
};
use strum::IntoEnumIterator;
use poise::{serenity_prelude::{Role, CreateSelectMenuOption, model::application::interaction::InteractionResponseType, CreateComponents, ApplicationCommandInteraction, Message}, ReplyHandle};
use tracing::{info, instrument};

#[derive(Debug, poise::Modal)]
#[allow(dead_code)] // fields only used for Debug print
struct TournamentMap{
    name: String,
}
/// Set config for the tournament
#[poise::command(slash_command, guild_only, rename = "set-config")]
pub async fn config(
    ctx: Context<'_>,
    #[description = "Select region"] region: Region,
) -> Result<(), Error> {
    if !user_is_manager(ctx).await? {
        return Ok(());
    }
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let collection: Collection<Document> = database.collection("Config");
    let msg = ctx.send(|s|
        s
        .ephemeral(false)
        .reply(true)
        .embed(|e|
            e.title("Awaiting to get config")
            .description("Please wait while I get the config for you")
        )
        .components(|c|{
            c.create_action_row(|a|
                a.create_select_menu(|m|
                    m.custom_id("config")
                    .placeholder("Select field to configurate")
                    .options(|o|
                        o.create_option(|o|
                            o.label("Mode")
                            .value("mode")
                            .description("Select game mode for the tournament")
                        )
                        .create_option(|o|
                            o.label("Map")
                            .value("map")
                            .description("Set the map for that game mode")
                        )
                        .create_option(|o|
                            o.label("Role")
                            .value("role")
                            .description("Set the role to assign the players for the tournament")
                        )
                        .create_option(|o|
                            o.label("Channel")
                            .value("channel")
                            .description("Set the channel to send the tournament updates")
                        )
                    )
                )
            )
        })
    ).await?;
    display_config(&ctx, &msg, &region).await?;

    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(120));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await{
        match mci.data.values[0].as_str(){
            "mode" => {
                mci.defer(&ctx.http()).await?;
                let follow_up = mci.create_followup_message(&ctx.http(), |f|{
                    f.content("Select a mode")
                    .ephemeral(false)
                    .components(|c|
                        c.create_action_row(|c|
                            c.create_select_menu(|m|
                                m.custom_id("menu")
                                .placeholder("Select a mode")
                                .options(|o|{
                                    for mode in Mode::iter(){
                                        let mut option = CreateSelectMenuOption::default();
                                        option.label(mode.to_string())
                                            .value(mode.to_string());
                                        o.add_option(option);
                                    }
                                    o
                                })
                            )
                        )
                    )
                }).await?;
                let resp = follow_up.clone();
                let cib = resp
                    .await_component_interactions(&ctx.serenity_context().shard)
                    .timeout(std::time::Duration::from_secs(120));
                let mut cic = cib.build();
                while let Some(mci2) = &cic.next().await{
                    mci2.defer(ctx.http()).await?;
                    println!("{}", mci2.data.values[0].as_str());
                    let mode = Mode::find_key(mci2.data.values[0].as_str()).unwrap();
                    collection.update_one(doc! {}, set_config("mode", format!("{:?}", mode).as_str()), None).await?;
                    let mut msg2 = mci2.message.clone();
                    msg2.edit(ctx.http(), |s|
                        s.components(|c|c)
                            .embed(|e|
                                e.title("Mode has been set!")
                                .description(format!("Mode has been set to {}
                                You may safely dismiss this message and change to other config", mci.data.values[0].as_str()))
                            )
                    ).await?;
                    break;
                }
            },
            "role" => {
                mci.defer(&ctx.http()).await?;
                let roles = ctx.guild_id().unwrap().roles(ctx.http()).await?;
                let follow_up = mci.create_followup_message(&ctx.http(), |f|{
                    f.content("Select a role to assign to players!")
                    .ephemeral(false)
                    .components(|c|
                        c.create_action_row(|c|
                            c.create_select_menu(|m|
                                m.custom_id("menu")
                                .placeholder("Select a role.")
                                .options(|o|{
                                    for (role_id, role) in roles.iter(){
                                        let mut option = CreateSelectMenuOption::default();
                                        option.label(role.clone().name)
                                            .value(role_id.to_string());
                                        o.add_option(option);
                                    }
                                    o
                                })
                            )
                        )
                    )
                }).await?;
                let resp = follow_up.clone();
                let cib = resp
                    .await_component_interactions(&ctx.serenity_context().shard)
                    .timeout(std::time::Duration::from_secs(120));
                let mut cic = cib.build();
                while let Some(mci2) = &cic.next().await{
                    mci2.defer(ctx.http()).await?;
                    let role_id = mci2.data.values[0].as_str();
                    collection.update_one(doc! {}, set_config("role", role_id), None).await?;
                    let mut msg2 = mci2.message.clone();
                    msg2.edit(ctx.http(), |s|
                        s.components(|c|c)
                            .embed(|e|
                                e.title("Role has been set!")
                                .description(format!("<@&{}> will be assigned to players when they register.
                                You may safely dismiss this message and change to other config.", role_id))
                            )
                    ).await?;
                    break;
                }
            },
            "channel" => {
                mci.defer(&ctx.http()).await?;
                let channels = ctx.guild_id().unwrap().channels(ctx.http()).await?;
                let follow_up = mci.create_followup_message(&ctx.http(), |f|{
                    f.content("Select a channel to publish matches' result!")
                    .ephemeral(false)
                    .components(|c|
                        c.create_action_row(|c|
                            c.create_select_menu(|m|
                                m.custom_id("menu")
                                .placeholder("Select a channel")
                                .options(|o|{
                                    for (channel_id, channel) in channels.iter(){
                                        let mut option = CreateSelectMenuOption::default();
                                        option.label(channel.clone().name)
                                            .value(channel_id.to_string());
                                        o.add_option(option);
                                    }
                                    o
                                })
                            )
                        )
                    )
                }).await?;
                let resp = follow_up.clone();
                let cib = resp
                    .await_component_interactions(&ctx.serenity_context().shard)
                    .timeout(std::time::Duration::from_secs(120));
                let mut cic = cib.build();
                while let Some(mci2) = &cic.next().await{
                    mci2.defer(ctx.http()).await?;
                    let channel_id = mci2.data.values[0].as_str();
                    collection.update_one(doc! {}, set_config("channel", channel_id), None).await?;
                    let mut msg2 = mci2.message.clone();
                    msg2.edit(ctx.http(), |s|
                        s.components(|c|c)
                            .embed(|e|
                                e.title("Role has been set!")
                                .description(format!("All tournament updates will be posted in <#{}>.
                                You may safely dismiss this message and change to other config", channel_id))
                            )
                    ).await?;
                    break;
                }
            },
            "map" => {
                let map = poise::execute_modal_on_component_interaction::<TournamentMap>(ctx, mci.clone(), None, None).await?.unwrap();
                collection.update_one(doc! {}, set_config("map", map.name.as_str()), None).await?;
            },
            _ => unreachable!("No way this is triggered")

        };
        display_config(&ctx, &msg, &region).await?;
    };

    Ok(())
}

async fn display_config(ctx: &Context<'_>, msg: &ReplyHandle<'_>, region: &Region) -> Result<(), Error>{
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection("Config");
    let config = collection.find_one(doc! {}, None).await.unwrap().unwrap();
    let registration_status = if config.get("registration").unwrap().as_bool().unwrap() {
        "Open"
    } else {
        "Closed"
    };
    let tournament_status = if config.get("tournament_started").unwrap().as_bool().unwrap() {
        "Ongoing"
    } else {
        "Not yet started"
    };
    let map = match config.get("map").unwrap().as_str(){
        Some(map) => map,
        None => "Not yet set"
    };
    let mode = match config.get("mode").unwrap().as_str(){
        Some(mode) => format!("{}",Mode::find_key(mode).unwrap()),
        None => "Not yet set".to_string()
    };
    let role = match config.get("role").unwrap().as_str(){
        Some(role) => {
            format!("<@&{}>",role)
        },
        None => "Not yet set".to_string()
    };
    let channel = match config.get("channel").unwrap().as_str(){
        Some(channel) => format!("<#{}>",channel),
        None => "Not yet set".to_string()
    };
    msg.edit(*ctx, |s|
        s.reply(true)
        .ephemeral(true)
        .embed(|e|
            e.title("Current Configuration")
            .description(
                format!(
                    "**Registration status:** {}
                    **Tournament status:** {}
                    **Mode:** {}
                    **Map:** {}
                    **Role assigned to players:** {}
                    **Channel to publish results of matches:** {}",
                    registration_status, tournament_status, mode, map, role, channel 
                )
            )
        )
    ).await?;
    Ok(())
}

