// use crate::bracket_tournament::player::Player;
// use crate::{Context, Error};
// use crate::utils::api;

// /// Get the player's profile
// #[poise::command(slash_command, prefix_command)]
// pub async fn player(
//   ctx: Context<'_>, 
//   #[description = "Put your tag here (without #)" ] tag: String) 
// -> Result<(), Error>{
//   match api::api_handlers::get_api_link("player", &tag){
    
//   }















//   Ok(())
// }
// //   match Player::new(&tag).await{  
// //     Ok(player) => {
// //       ctx.channel_id()
// //         .send_message(&ctx, |response|{
// //           response
// //             .allowed_mentions(|a| a.replied_user(true))
// //             .embed(|e|{
// //               e.title(format!("**{}({})**",player.name, player.tag))
// //                 .thumbnail(format!("https://cdn-old.brawlify.com/profile-low/{}.png", player.icon.id))
// //                 .fields(vec![
// //                   ("Trophies", &player.trophies.to_string(), true),
// //                   ("Highest Trophies", &player.highest_trophies.to_string(), true),
// //                   ("3v3 Victories",&player.victories_3v3.to_string(), true),
// //                   ("Solo Victories", &player.solo_victories.to_string(), true),
// //                   ("Duo Victories", &player.duo_victories.to_string(), true),
// //                   ("Best Robo Rumble Time", &player.best_robo_rumble_time.to_string(), true),
// //                   ("Club", &player.club.name.to_string(), true),
// //               ])
// //               .timestamp(ctx.created_at())
// //             })
// //       }).await;
// //     },
// //    Err(false) => {
// //       ctx.say(format!("Player with tag {} not found", tag)).await?;
// //   },
// //   Err(_err) => {
// //       ctx.say("Something went wrong").await?;
// //   }
// // }
// //   Ok(())
// // }
