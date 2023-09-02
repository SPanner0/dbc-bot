use crate::misc::Region;
use crate::{Context, Error};
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use strum::IntoEnumIterator;
///Reset all match_id of players and remove mannequins
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
pub async fn reset_match_id(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Resetting match id and removing mannequins...")
        .await?;
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let collection: Collection<Document> = database.collection("Player");
        collection
            .update_many(doc! {}, doc! { "$set": { "match_id": null } }, None)
            .await?;
        ctx.channel_id()
            .send_message(ctx, |s| {
                s.content(format!("All Match IDs from {} are reset!", region))
            })
            .await?;
        collection
            .delete_many(doc! { "name": "Mannequin" }, None)
            .await?;
        ctx.channel_id()
            .send_message(ctx, |s| {
                s.content(format!("All mannequins in {} are removed!", region))
            })
            .await?;
    }
    ctx.channel_id()
        .send_message(ctx, |s| s.content("Complete!"))
        .await?;
    Ok(())
}
