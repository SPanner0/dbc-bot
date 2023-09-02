use crate::Context;
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use poise::serenity_prelude::Colour;
use std::error::Error;
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// A trait for stripping quotes from a string.
pub trait QuoteStripper {
    /// Strip double quotes from the string and return a new String.
    fn strip_quote(&self) -> String;
}

impl QuoteStripper for String {
    /// Strip double quotes from the string and return a new String.
    ///
    /// # Examples
    ///
    /// ```
    /// let s = String::from("\"Hello, world!\"");
    /// let stripped = s.strip_quote();
    /// assert_eq!(stripped, "Hello, world!");
    /// ```
    fn strip_quote(&self) -> String {
        let mut result = String::new();

        for c in self.chars() {
            if c != '"' {
                result.push(c);
            }
        }

        result
    }
}

/// This function converts a difficulty level represented as a serde_json::Value into its corresponding
/// textual representation.
///
/// # Arguments
///
/// * `num` - A reference to a serde_json::Value representing the difficulty level.
///
/// # Returns
///
/// A String representing the textual description of the difficulty level. If the provided numeric value
/// does not correspond to a recognized difficulty level, a default message is returned.
///
/// # Examples
///
/// ```
/// use serde_json::json;
///
/// let num = json!(3);
/// let difficulty = get_difficulty(&num);
/// assert_eq!(difficulty, "Expert");
/// ```
pub fn get_difficulty(num: &serde_json::Value) -> String {
    let option: i32 = serde_json::from_value(num.clone()).unwrap();
    match option {
        0 => "Easy".to_string(),
        1 => "Normal".to_string(),
        2 => "Hard".to_string(),
        3 => "Expert".to_string(),
        4 => "Master".to_string(),
        5 => "Insane".to_string(),
        6 => "Insane II".to_string(),
        7 => "Insane III".to_string(),
        8 => "Insane IV".to_string(),
        9 => "Insane V".to_string(),
        10 => "Insane VI".to_string(),
        11 => "Insane VII".to_string(),
        12 => "Insane VIII".to_string(),
        13 => "Insane IX".to_string(),
        14 => "Insane X".to_string(),
        15 => "Insane XI".to_string(),
        16 => "Insane XII".to_string(),
        17 => "Insane XIII".to_string(),
        18 => "Insane XIV".to_string(),
        19 => "Insane XV".to_string(),
        20 => "Insane XVI".to_string(),
        _ => "Congratulations, either we were wrong, or you unlocked new difficulty".to_string(),
    }
}

/// This function returns the URL of a game mode icon based on the provided event name.
///
/// # Arguments
///
/// * `event_name` - A string slice containing the name of the event.
///
/// # Returns
///
/// An `Option<&str>` representing the URL of the event icon. If the event name is recognized,
/// it returns `Some(&str)` with the URL; otherwise, it returns `None`.
///
/// # Examples
///
/// ```
/// let event_name = "gemGrab";
/// match get_mode_icon(event_name) {
///     Some(url) => println!("URL for {} is {}", event_name, url),
///     None => println!("Event name {} not found.", event_name),
/// }
/// ```
pub fn get_mode_icon(event_name: &serde_json::Value) -> &str {
    let binding = event_name.to_string().strip_quote();
    let event_link: &str = binding.as_str();

    // Match the event_name to known event names and return the corresponding URL as Some(&str)
    match event_link {
        "brawlBall" => "https://cdn.brawlstats.com/event-icons/event_mode_gem_grab.png",
        "bounty" => "https://cdn.brawlstats.com/event-icons/event_mode_bounty.png",
        "gemGrab" => "https://cdn.brawlstats.com/event-icons/event_mode_gem_grab.png",
        "wipeout" => "https://cdn.brawlstats.com/event-icons/event_mode_wipeout.png",
        "heist" => "https://cdn.brawlstats.com/event-icons/event_mode_heist.png",
        "hotZone" => "https://cdn.brawlstats.com/event-icons/event_mode_hot_zone.png",
        "knockout" => "https://cdn.brawlstats.com/event-icons/event_mode_knockout.png",
        "siege" => "https://cdn.brawlstats.com/event-icons/event_mode_siege.png",
        "raid" => "https://cdn.brawlstats.com/event-icons/event_mode_raid.png",
        "soloShowdown" => "https://cdn.brawlstats.com/event-icons/event_mode_showdown.png",
        "duoShowdown" => "https://cdn.brawlstats.com/event-icons/event_mode_showdown.png",
        _ => {
            "https://cdn.discordapp.com/emojis/1133867752155779173.webp?size=4096&quality=lossless"
        }
    }
}

/// Converts a string result into a corresponding color represented as a `poise::serenity_prelude::Colour` struct.
///
/// This function takes a `result` parameter, which is a string indicating the result of an event.
/// It matches the input string to predefined cases and returns a `poise::serenity_prelude::Colour` value representing the
/// associated color.
///
/// # Arguments
///
/// * `result` - A string representing the result of an event ("victory", "defeat", "draw").
///
/// # Returns
///
/// A `poise::serenity_prelude::Colour` struct representing the color associated with the input result. If the input result
/// is not recognized (i.e., not "victory", "defeat", or "draw"), the function returns a default
/// color (black).
///
/// # Examples
///
/// ```
/// use your_crate_name::color;
/// use poise::serenity_prelude::Colour;
///
/// let victory_color = color("victory".to_string());
/// assert_eq!(victory_color, Colour::new(0x00800)); // Green
///
/// let defeat_color = color("defeat".to_string());
/// assert_eq!(defeat_color, Colour::new(0xFF0000)); // Red
///
/// let draw_color = color("draw".to_string());
/// assert_eq!(draw_color, Colour::new(0xFFFFFF)); // White
///
/// let unknown_color = color("unknown".to_string());
/// assert_eq!(unknown_color, Colour::new(0x000000)); // Default (black)
/// ```
pub fn get_color(result: String) -> Colour {
    match result.as_str() {
        "victory" => Colour::new(u32::from_str_radix("90EE90", 16).unwrap()), // Green
        "defeat" => Colour::new(u32::from_str_radix("FF0000", 16).unwrap()),  // Red
        "draw" => Colour::new(u32::from_str_radix("FFFFFF", 16).unwrap()),    // White
        _ => Colour::new(000000), // Default color (black) for unknown cases
    }
}

// Define a custom error type for your application
#[derive(Debug)]
pub struct CustomError(pub String);
/// Implements the `fmt::Display` trait for the `CustomError` struct.
///
/// This implementation allows instances of the `CustomError` struct to be formatted as strings
/// when using the `format!`, `println!`, or `write!` macros. It displays the error message
/// contained within the `CustomError` struct.
///
/// # Example
///
/// ```
/// use your_crate_name::CustomError;
///
/// let error = CustomError("An error occurred".to_string());
/// println!("Error: {}", error); // Prints "Error: CustomError: An error occurred"
/// ```
impl fmt::Display for CustomError {
    /// Formats the `CustomError` instance as a string.
    ///
    /// # Arguments
    ///
    /// * `self` - The `CustomError` instance to format.
    /// * `f` - The formatter used to write the formatted output.
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating whether the formatting operation was successful.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CustomError: {}", self.0)
    }
}

impl Error for CustomError {}

/// Checks if a player with the given Discord user ID exists in the database.
///
/// This function queries the specified MongoDB collection for a document that matches
/// the Discord user ID of the invoking user in the provided `Context`. If a matching
/// document is found, it returns the player's data as a `Document`. If no matching
/// document is found or an error occurs during the query, it returns `None`.
///
/// # Parameters
///
/// - `ctx`: A reference to the Serenity `Context` containing information about the
///          current Discord interaction and server context.
///
/// # Returns
///
/// An `Option<Document>` representing the player's data if found, or `None` if the player
/// is not in the database or an error occurs.
///
/// # Examples
///
/// ```rust
/// use serenity::prelude::*;
/// use mongodb::Collection;
/// use mongodb::bson::doc;
///
/// struct PlayerDataKey;
///
/// impl TypeMapKey for PlayerDataKey {
///     type Value = Collection<Document>;
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let ctx = Context::new().await; // Create a Serenity context (replace with your actual setup).
///
///     match is_in_db(&ctx).await {
///         Some(player) => {
///             println!("The player exists in the database. Player data: {:?}", player);
///         }
///         None => {
///             println!("The player does not exist in the database.");
///         }
///     }
/// }
/// ```
pub async fn is_in_db(ctx: &Context<'_>, discord_id: Option<String>) -> Option<Document> {
    let invoker_id = match discord_id {
        Some(id) => id,
        None => ctx.author().id.to_string(),
    };
    // Define a variable to hold the result
    let mut result: Option<Document> = None;

    // Iterate through the regions and check each database
    for region in Region::iter() {
        let player_data: Collection<Document> = ctx
            .data()
            .database
            .regional_databases
            .get(&region)
            .unwrap()
            .collection("Player");
        match player_data
            .find_one(doc! { "discord_id": &invoker_id.strip_quote()}, None)
            .await
        {
            Ok(Some(player)) => {
                result = Some(player);
                break; // Exit the loop when a match is found
            }
            Ok(None) => {
                continue;
            }
            Err(err) => {
                eprintln!("Error while querying database: {:?}", err);
                result = None;
                break;
            }
        }
    }
    result
}
/// Get human-readable details for a given region abbreviation.
///
/// This function takes a region abbreviation as a string and returns a corresponding
/// human-readable description of that region. If the abbreviation matches a known region,
/// it returns the description; otherwise, it provides a humorous response for unknown
/// or unexpected inputs.
///
/// # Parameters
///
/// - `region`: A reference to a string representing the region abbreviation.
///
/// # Returns
///
/// A `&str` containing the human-readable details of the region.
///
/// # Examples
///
/// ```rust
/// let region_details = region_details("EU");
/// assert_eq!(region_details, "Europe");
///
/// let unknown_region_details = region_details("Mars");
/// assert_eq!(unknown_region_details, "You are not from Earth, aren't you?");
/// ```
///
/// # Notes
///
/// Known region abbreviations:
///
/// - "APAC": Asia & Oceania
/// - "EU": Europe
/// - "NASA": North America & South America
///
/// Unknown abbreviations will result in the humorous response.
///
/// # Panics
///
/// This function does not panic under any circumstances and always returns a valid `&str`.
pub fn region_details(region: &str) -> &str {
    match region {
        "APAC" => "Asia & Oceania",
        "EU" => "Europe",
        "NASA" => "North America & South America",
        _ => "You are not from Earth, aren't you?",
    }
}

/// Define an enum called `Region` to represent geographical regions.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, poise::ChoiceParameter, EnumIter, Eq, Hash, PartialEq)]
pub enum Region {
    #[name = "North America & South America"]
    NASA,
    #[name = "Europe"]
    EU,
    #[name = "Asia & Oceania"]
    APAC,
}

impl Region {
    /// Custom function to find a variant by its associated name.
    ///
    /// # Arguments
    ///
    /// * `name` - A string containing the associated name of the variant to find.
    ///
    /// # Returns
    ///
    /// * `Some(Region)` - The enum variant if found.
    /// * `None` - If no variant is found for the given name.
    ///
    /// # Example
    ///
    /// ```
    /// use my_module::Region;
    ///
    /// let name_to_find = "Europe";
    ///
    /// if let Some(region) = Region::find_key(name_to_find) {
    ///     println!("Found variant: {:?}", region);
    /// } else {
    ///     println!("Variant not found for name: {}", name_to_find);
    /// }
    /// ```
    pub fn find_key(name: &str) -> Option<Region> {
        match name {
            "NASA" => Some(Region::NASA),
            "EU" => Some(Region::EU),
            "APAC" => Some(Region::APAC),
            _ => None,
        }
    }
}
