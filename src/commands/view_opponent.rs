use crate::{
    bracket_tournament::{
        api::{
            get_api_link, 
            request
        },
        assign_match_id::update_match_id,
        config::get_config,
        region,
        update_battle::update_battle,
    },
    database_utils::{
        find_discord_id::find_discord_id,
        find_enemy::{find_enemy, is_mannequin},
    },
    misc::QuoteStripper,
    Context, Error,
};

use mongodb::{
    bson::{doc, Document},
    Collection,
};

/// View your opponent
#[poise::command(slash_command, guild_only)]
pub async fn view_opponent(ctx: Context<'_>) -> Result<(), Error> {
    let caller = match find_discord_id(&ctx, None).await {
        Some(caller) => caller,
        None => {
            ctx.send(|s| {
                s.reply(true).ephemeral(false).embed(|e| {
                    e.title("You are not in the tournament!")
                        .description("Sorry, you are not in the tournament to use this command!")
                })
            })
            .await?;
            return Ok(());
        }
    };
    //Get player document via their discord_id
    let match_id: i32 = (caller.get("match_id").unwrap()).as_i32().unwrap();
    let caller_tag = caller.get("tag").unwrap().to_string().strip_quote();
    let region = region::Region::find_key(
        caller
            .get("region")
            .unwrap()
            .to_string()
            .strip_quote()
            .as_str(),
    )
    .unwrap();

    //Check if the user has already submitted the result or not yet disqualified
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let round = get_config(database)
        .await
        .get("round")
        .unwrap()
        .as_i32()
        .unwrap();
    let current_round: Collection<Document> =
        database.collection(format!("Round {}", round).as_str());
    let caller: Document = match current_round
        .find_one(doc! {"tag": &caller_tag}, None)
        .await
    {
        Ok(Some(player)) => player,
        Ok(None) => {
            ctx.send(|s| {
                s.reply(true).ephemeral(false).embed(|e| {
                    e.title("You are not in this round!")
                        .description("Oops! Better luck next time")
                })
            })
            .await?;
            return Ok(());
        }
        Err(_) => {
            ctx.send(|s| {
                s.reply(true).ephemeral(false).embed(|e| {
                    e.title("An error pops up!")
                        .description("Please run this command later!")
                })
            })
            .await?;
            return Ok(());
        }
    };
    let enemy = find_enemy(&ctx, &region, &round, &match_id, &caller_tag)
        .await
        .unwrap();
    if is_mannequin(&enemy) {
        let next_round: Collection<Document> =
            database.collection(format!("Round {}", round + 1).as_str());
        next_round.insert_one(update_match_id(caller), None).await?;
        ctx.send(|s| {
            s.reply(true).ephemeral(false).embed(|e| {
                e.title("Bye! See you next... round!").description(
                    "You have been automatically advanced to the next round due to bye!",
                )
            })
        })
        .await?;
        update_battle(database, round, match_id).await?;
        return Ok(());
    }
    let enemy_tag = enemy.get("tag").unwrap().to_string().strip_quote();

    // let player1 = request(get_api_link("player", &caller_tag).as_str()).await?;
    // let player2 = request(get_api_link("player", &enemy_tag).as_str()).await?;

    ctx.send(|s| {
        s.reply(true).ephemeral(false).embed(|e| {
            e.title("**DISCORD BRAWL CUP TOURNAMENT**")
                .description(format!("Round {} - Match {}", round, match_id))
                .fields(vec![
                    (caller.get("name").unwrap().to_string(), caller_tag, true),
                    ("".to_string(), "**VS**".to_string(), true),
                    (enemy.get("name").unwrap().to_string(), enemy_tag, true),
                ])
        })
    })
    .await?;
    Ok(())
}
