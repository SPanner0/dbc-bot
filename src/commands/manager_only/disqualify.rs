use mongodb::bson::{Document, doc};
use tracing::{instrument, info};

use crate::bracket_tournament::{mannequin::add_mannequin, region::Region};
use crate::{Context, Error};

#[instrument]
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
async fn disqualify(ctx: Context<'_>, #[description = "The ID of the user to disqualify"] user_id: u64, region: Region) -> Result<(), Error> {
    info!("Attempting to disqualify player");
    let collection = ctx.data().database.regional_databases.get(&region).unwrap().collection::<Document>("Player");

    let player = collection.find_one(doc! {"discord_id": user_id.to_string()}, None).await?;

    match player {
        Some(player) => {
            let match_id = player.get("match_id").unwrap().to_string().parse::<i32>().unwrap();
            let mannequin = add_mannequin(&region, Some(match_id), None);
            let collection = ctx.data().database.regional_databases.get(&region).unwrap().collection::<Document>("Player");
            collection.delete_one(doc! {"discord_id": user_id.to_string()}, None).await?;
            collection.insert_one(mannequin, None).await?;
            ctx.say(format!("Successfully disqualified user")).await?;

            info!("Sucessfully disqualified player {}", user_id);
            return Ok(());
        },
        None => {
            ctx.say("No user was found for this ID!").await?;
            return Ok(());
        }
    }
}