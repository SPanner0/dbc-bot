use crate::{bracket_tournament::region::Region, misc::CustomError, Context, Error};
use mongodb::{
    bson::{doc, Document, Bson},
    Collection,
};
use strum::IntoEnumIterator;

pub async fn find_player(ctx: &Context<'_>) -> Result<Option<Document>, Error> {
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let collection: Collection<Document> = database.collection("Players");
        let filter = doc! {"discord_id": ctx.author().id.to_string()};
        match collection.find_one(filter, None).await {
            Ok(result) => match result {
                Some(p) => {
                    return Ok(Some(p));
                }
                None => continue,
            },
            Err(_) => {
                return Ok(None);
            }
        }
    }
    Ok(None)
}

pub fn find_round(config: &Document) -> String {
    let round = match config.get("round") {
        Some(round) => {
            if let Bson::Int32(0) = round {
                "Players".to_string()
            } else {
                format!("Round {}", round.as_i32().unwrap())
            }
        }
        _ => unreachable!("Round not found in config!"),
    };

    round
}
/// Asynchronously searches for enemy in the regional databases.
///
/// # Arguments
///
/// - `ctx` - The context of the application.
/// - `region` - The region of the player.
/// - `round` - The round of the match.
/// - `match_id` - The match id of the player.
/// - `other_tag` - The tag of the other player.
///
/// # Returns
///
/// An `Option<Document>` representing enemy if found, or `None` if not found or an error occurred.
pub async fn find_enemy(
    ctx: &Context<'_>,
    region: &Region,
    round: &i32,
    match_id: &i32,
    other_tag: &str,
) -> Option<Document> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection(format!("Round {}", round).as_str());
    let filter = doc! {
        "match_id": match_id,
        "tag": {
           "$ne": other_tag
        }
    };
    match collection.find_one(filter, None).await {
        Ok(Some(enemy)) => Some(enemy),
        Ok(None) => None,
        Err(_err) => {
            None
        }
    }
}

/// Asynchronously searches for a player's tag in the regional databases.
///
/// # Arguments
///
/// * `ctx` - The context of the application.
/// * `tag` - The tag to search for.
///
/// # Returns
///
/// An `Option<Document>` representing the player's data if found, or `None` if not found or an error occurred.
pub async fn find_tag(ctx: &Context<'_>, tag: &str) -> Option<Document> {
    let mut result: Option<Document> = None;
    let proper_tag = match tag.starts_with('#') {
        true => &tag[1..],
        false => tag,
    };
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let player_data: Collection<Document> = database.collection("Player");

        match player_data
            .find_one(doc! { "tag": format!("#{}",&proper_tag)}, None)
            .await
        {
            Ok(Some(player)) => {
                result = Some(player);
                break;
            }
            Ok(None) => {
                continue;
            }
            Err(_err) => {
                result = None;
                break;
            }
        }
    }
    result
}

pub fn is_mannequin(enemy: &Document) -> bool {
    enemy.get("tag").unwrap() == &Bson::Null
}

