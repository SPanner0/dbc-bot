use crate::{Context, Error};
use mongodb::bson::{doc, Document};

#[poise::command(slash_command, prefix_command)]
pub async fn get_player_data(
    ctx: Context<'_>,
    #[description = "Check a player registration status by user ID here"] id: String,
) -> Result<(), Error> {
    let player_data = ctx
        .data()
        .db_client
        .database("DBC-bot")
        .collection("PlayerDB");
    let individual_player: Document = player_data
        .find_one(
            doc! {
                "id": &id
            },
            None,
        )
        .await?
        .expect(&format!("Missing: {} document.", &id));

    let name = individual_player.get("name").and_then(|n| n.as_str()).unwrap_or("Player username not found in database.");
    let tag = individual_player.get("tag").and_then(|t| t.as_str()).unwrap_or("Player tag not found in database");

    ctx.channel_id()
        .send_message(&ctx, |response| {
            response
                .allowed_mentions(|a| a.replied_user(true))
                .embed(|e| {
                    e.title(format!("**{}**", name))
                        .description(tag)
                })
        })
        .await?;

    Ok(())
}
