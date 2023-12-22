use dbc_bot::Region;
use futures::StreamExt;
use poise::ReplyHandle;
use poise::serenity_prelude::ReactionType;
use crate::Context;
use crate::Error;
use crate::database::config::get_config;
use crate::database::find::find_round;
use crate::database::stat::count_registers;
const TIMEOUT: u64 = 300;

pub async fn tournament_mod_panel(ctx: &Context<'_>, msg: &ReplyHandle<'_>, region: &Region) -> Result<(), Error> {
  display_start_menu(ctx, msg, region).await?;
  let resp = msg.clone().into_message().await?;
  let cib = resp
      .await_component_interactions(&ctx.serenity_context().shard)
      .timeout(std::time::Duration::from_secs(TIMEOUT));
  let mut cic = cib.build();
  while let Some(mci) = &cic.next().await{
    match mci.data.custom_id.as_str(){
      "start" => {
        mci.defer(&ctx.http()).await?;
        display_start_menu(ctx, msg, region).await?;
      },
      "next" => {
        mci.defer(&ctx.http()).await?;
      },
      _ => {

      }
    }
  }
 
  Ok(())
}

async fn display_start_menu(ctx: &Context<'_>, msg: &ReplyHandle<'_>, region: &Region) -> Result<(), Error>{
  let round = find_round(&get_config(&ctx, &region).await);
  let valid = prerequisite(ctx, region).await;
 
  match round.as_str(){
    "Players" => {
        let count_prompt = format!("{}",count_registers(ctx, region).await?);
        let valid_prompt = match &valid{
          true => "All configurations are set! Ypu can start tournament now",
          false => "Some configuration is missing, please re-run </host:1185308022285799546> and check ⚙️ configuration menu",
        };
        display_start_buttons(ctx, msg, &valid).await?;
        msg.edit(*ctx, |m|{
          m.embed(|e|{
            e.title("Tournament menu")
            .description(format!("{}\n{}\n", count_prompt, valid_prompt))
          })
        }).await?;
    },
    _ => {

    }
  }
  Ok(())
}
async fn display_start_buttons(ctx: &Context<'_>, msg: &ReplyHandle<'_>, start: &bool) -> Result<(), Error>{
  msg.edit(*ctx, |m| {
    m.components(|c|{
      c.create_action_row(|row| {
        row.create_button(|b| {
          b.custom_id("start")
          .label("Start tournament")
          .style(poise::serenity_prelude::ButtonStyle::Primary)
          .emoji(ReactionType::Unicode("▶️".to_string()))
          .disabled(!start)
        });
        row.create_button(|b| {
          b.custom_id("next")
          .label("Next round")
          .emoji(ReactionType::Unicode("▶️".to_string()))
          .style(poise::serenity_prelude::ButtonStyle::Primary)
        })
      })
    })
  }).await?;
  Ok(())
}
async fn prerequisite(ctx: &Context<'_>, region: &Region) -> bool {
  let config = get_config(ctx, region).await;
  if config.get_bool("tournament").unwrap() == false 
  || config.get("mode").is_none() == true
  || config.get("role").is_none() == true 
  || config.get("channel").is_none() == true{
      return false; 
  } 
  true
}