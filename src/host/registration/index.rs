use crate::database::open::{registration_open, tournament};
use crate::database::stat::count_registers;
use crate::database::update::toggle_registration;
use crate::{Context, Error};
use dbc_bot::Region;
use futures::StreamExt;
use poise::serenity_prelude::ReactionType;
use poise::ReplyHandle;

use super::detail::{detail, term};

const TIMEOUT: u64 = 300;
struct Reg {
    registration: bool,
    tournament: bool,
    count: i32,
    region: Region,
}

pub async fn registration_mod_panel(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    let mut reg = getter(ctx, region).await?;
    display_info(ctx, msg, &reg).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "registration" => {
                mci.defer(&ctx.http()).await?;
                toggle_registration(ctx, region, !reg.registration).await?;
            }
            "detail" => {
                mci.defer(&ctx.http()).await?;
                detail(ctx, msg, region).await?;
            }
            _ => {
                reg = getter(ctx, region).await?;
                display_info(ctx, msg, &reg).await?;
                continue;
            }
        }
        reg = getter(ctx, region).await?;
        display_info(ctx, msg, &reg).await?;
    }
    Ok(())
}

async fn display_info(ctx: &Context<'_>, msg: &ReplyHandle<'_>, reg: &Reg) -> Result<(), Error> {
    let flag = if reg.tournament {
        "\n. Tournament is currently running. Toggle is disabled!"
    } else {
        ""
    };
    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title("**Registration Panel**")
                .description(format!(
                    r#"Registration is currently: {}
There are {} registered players for the tournament of {}.{flag}
Below are options:
🔒: Toggle registration
- Open/Close registration phase for players.
- This will be disabled during the tournament phase.
🔍: View
- Lets you see all players who has already registered in the tournament
"#,
                    term(reg.registration),
                    reg.count,
                    reg.region,
                ))
                .color(0xFFFF00)
        })
        .components(|c| {
            c.create_action_row(|row| {
                row.create_button(|b| {
                    b.custom_id("registration")
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
                        .disabled(reg.tournament)
                        .emoji(ReactionType::Unicode("🔒".to_string()))
                });
                row.create_button(|b| {
                    b.custom_id("detail")
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("🔍".to_string()))
                })
            })
        })
    })
    .await?;
    Ok(())
}

async fn getter(ctx: &Context<'_>, region: &Region) -> Result<Reg, Error> {
    let status = registration_open(ctx).await;
    let tournament_status = tournament(ctx, region).await;
    let count = count_registers(ctx, region).await?;
    Ok(Reg {
        registration: status,
        tournament: tournament_status,
        count,
        region: region.clone(),
    })
}
