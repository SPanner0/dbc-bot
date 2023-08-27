use crate::bracket_tournament::api;
use crate::utils::embed_color::get_color;
use crate::utils::misc::get_mode_icon;
use crate::utils::misc::QuoteStripper;
use crate::{Context, Error};

/// Get the latest log of a player
#[poise::command(slash_command, prefix_command)]
pub async fn latest_log(
    ctx: Context<'_>,
    #[description = "Put your tag here (without #)"] tag: String,
) -> Result<(), Error> {
    let endpoint = api::api_handlers::get_api_link("battle_log", &tag.to_uppercase());
    match api::api_handlers::request(&endpoint).await {
        Ok(log) => {
            let player_endpoint = api::api_handlers::get_api_link("player", &tag.to_uppercase());
            let player = api::api_handlers::request(&player_endpoint).await.unwrap();
            ctx.send(|s| {
                s.content("".to_string()).reply(true).embed(|e| {
                    e.author(|a| a.name(ctx.author().name.clone()))
                        .title(format!(
                            "{}'s most recent match: {}",
                            player["name"].to_string().strip_quote(),
                            log["items"][0]["battle"]["result"]
                                .to_string()
                                .strip_quote()
                        ))
                        .color(get_color(
                            log["items"][0]["battle"]["result"]
                                .to_string()
                                .strip_quote(),
                        ))
                        .thumbnail(get_mode_icon(&log["items"][0]["event"]["mode"])) // why it no shows?
                        .field(
                            "Battle Time",
                            log["items"][0]["battleTime"].to_string(),
                            false,
                        )
                        .fields(vec![
                            (
                                "Mode",
                                log["items"][0]["event"]["mode"].to_string().strip_quote(),
                                true,
                            ),
                            (
                                "Map",
                                log["items"][0]["event"]["map"].to_string().strip_quote(),
                                true,
                            ),
                            (
                                "Duration",
                                format!(
                                    "{}s",
                                    log["items"][0]["battle"]["duration"]
                                        .to_string()
                                        .strip_quote()
                                ),
                                true,
                            ),
                            (
                                "Game",
                                log["items"][0]["battle"]["type"].to_string().strip_quote(),
                                true,
                            ),
                            (
                                "Trophy Change",
                                log["items"][0]["battle"]["trophyChange"]
                                    .to_string()
                                    .strip_quote(),
                                true,
                            ),
                            ("", "".to_string(), false),
                        ])
                        .field(
                            "=============================================".to_string(),
                            "",
                            false,
                        )
                        .fields(vec![
                            (
                                log["items"][0]["battle"]["teams"][0][0]["name"]
                                    .to_string()
                                    .strip_quote(),
                                format!(
                                    " {}\n {}",
                                    log["items"][0]["battle"]["teams"][0][0]["brawler"]["name"]
                                        .to_string()
                                        .strip_quote(),
                                    log["items"][0]["battle"]["teams"][0][0]["brawler"]["power"]
                                        .to_string()
                                        .strip_quote()
                                ),
                                true,
                            ),
                            (
                                log["items"][0]["battle"]["teams"][0][1]["name"]
                                    .to_string()
                                    .strip_quote(),
                                format!(
                                    " {}\n {}",
                                    log["items"][0]["battle"]["teams"][0][1]["brawler"]["name"]
                                        .to_string()
                                        .strip_quote(),
                                    log["items"][0]["battle"]["teams"][0][1]["brawler"]["power"]
                                        .to_string()
                                        .strip_quote()
                                ),
                                true,
                            ),
                            (
                                log["items"][0]["battle"]["teams"][0][2]["name"]
                                    .to_string()
                                    .strip_quote(),
                                format!(
                                    " {}\n {}",
                                    log["items"][0]["battle"]["teams"][0][2]["brawler"]["name"]
                                        .to_string()
                                        .strip_quote(),
                                    log["items"][0]["battle"]["teams"][0][2]["brawler"]["power"]
                                        .to_string()
                                        .strip_quote()
                                ),
                                true,
                            ),
                        ])
                        .fields(vec![
                            ("".to_string(), "".to_string(), true),
                            ("VS".to_string(), "".to_string(), true),
                            ("".to_string(), "".to_string(), true),
                        ])
                        .fields(vec![
                            (
                                log["items"][0]["battle"]["teams"][1][0]["name"]
                                    .to_string()
                                    .strip_quote(),
                                format!(
                                    " {}\n {}",
                                    log["items"][0]["battle"]["teams"][1][0]["brawler"]["name"]
                                        .to_string()
                                        .strip_quote(),
                                    log["items"][0]["battle"]["teams"][1][0]["brawler"]["power"]
                                        .to_string()
                                        .strip_quote()
                                ),
                                true,
                            ),
                            (
                                log["items"][0]["battle"]["teams"][1][1]["name"]
                                    .to_string()
                                    .strip_quote(),
                                format!(
                                    " {}\n {}",
                                    log["items"][0]["battle"]["teams"][1][1]["brawler"]["name"]
                                        .to_string()
                                        .strip_quote(),
                                    log["items"][0]["battle"]["teams"][1][1]["brawler"]["power"]
                                        .to_string()
                                        .strip_quote()
                                ),
                                true,
                            ),
                            (
                                log["items"][0]["battle"]["teams"][1][2]["name"]
                                    .to_string()
                                    .strip_quote(),
                                format!(
                                    " {}\n {}",
                                    log["items"][0]["battle"]["teams"][1][2]["brawler"]["name"]
                                        .to_string()
                                        .strip_quote(),
                                    log["items"][0]["battle"]["teams"][1][2]["brawler"]["power"]
                                        .to_string()
                                        .strip_quote()
                                ),
                                true,
                            ),
                        ])
                })
            })
            .await?;
        }
        Err(err) => {
            ctx.send(|s| {
                s.content("".to_string())
                    .reply(true)
                    .ephemeral(false)
                    .embed(|e| {
                        e.title(format!("Error: {:#?}", err))
                            .description(format!("No player is associated with {}", tag.to_uppercase()))
                    })
            })
            .await?;
        }
    }

    Ok(())
}
