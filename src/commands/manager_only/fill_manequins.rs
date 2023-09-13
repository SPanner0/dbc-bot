use crate::{
    bracket_tournament::{mannequin::add_mannequin, region::Region},
    checks::user_is_manager,
    Context, Error,
};
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use strum::IntoEnumIterator;
use tracing::{info, instrument};

/// Fill database with mannequins for testing purpose
#[instrument]
#[poise::command(
    slash_command,
    rename = "fill-mannequins",
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
pub async fn fill_mannequins(
    ctx: Context<'_>,
    #[description = "The number of mannequins to add"] quantity: i32,
) -> Result<(), Error> {
    if !user_is_manager(ctx).await? {
        return Ok(());
    }

    info!("Filling databases with mannequins for testing...");
    ctx.say("Filling databases with mannequins...").await?;
    for region in Region::iter() {
        info!("Filling mannequins for {}", region);
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let collection: Collection<Document> = database.collection("Player");
        for _ in 0..quantity {
            collection
                .insert_one(add_mannequin(&region, None, None), None)
                .await?;
        }
        ctx.channel_id()
            .send_message(ctx, |s| {
                s.content(format!("Added {} mannequins to {}", quantity, region))
            })
            .await?;
    }
    info!("Finished filling mannequins");
    ctx.channel_id()
        .send_message(ctx, |s| s.content("Complete!"))
        .await?;
    Ok(())
}
