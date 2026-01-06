use crate::commands::{mimic::*, schedule::*, vox::*};
use crate::pawthos::enums::color_errors::ColorError;
use crate::pawthos::enums::embed_type;
use crate::pawthos::enums::wallet_errors::WalletError;
use crate::pawthos::{
    enums::embed_type::EmbedType,
    structs::data::Data,
    types::{Context, Error, Reply, Result},
};
use crate::utils::{self, create_embed_builder};
use image::ImageEncoder;
use poise::serenity_prelude::{self as serenity, EditRole, Guild, RoleId};
mod mimic;
mod oot;
mod schedule;
mod vox;

pub fn return_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        daily(),
        shop(),
        pfp(),
        register(),
        vox(),
        mimic(),
        schedule(),
        help(),
        color(),
    ]
}

#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result {
    let config = poise::builtins::HelpConfiguration {
        show_subcommands: true,
        include_description: true,
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

// TODO: add the ability to grab both global pfp and guild.

/// Displays the calling users' profile picture
#[poise::command(slash_command)]
pub async fn pfp(
    ctx: Context<'_>,
    #[description = "User to show pfp of"] user: serenity::User,
) -> Result {
    let ce =
        utils::create_embed_builder("pfp", format!("{}'s pfp", &user.name), EmbedType::Neutral)
            .image(user.avatar_url().unwrap_or_else(|| {
                log::warn!("Slash Command Error: pfp: user.avatar_url() is None.");
                user.default_avatar_url()
            }));

    let r = Reply::default().embed(ce);
    ctx.send(r).await?;
    Ok(())
}
/// Gives you 10 tabs. can only be used once every 24 hours.
#[poise::command(slash_command)]
pub async fn daily(ctx: Context<'_>) -> Result {
    pub const DAILY_REWARD: i64 = 10;
    pub const DAILY_COOLDOWN_SECS: i64 = 24 * 60 * 60;

    let user_id = ctx.author().id;

    let now = chrono::Utc::now().timestamp();

    let res = ctx
        .data()
        .with_wallet_user_write(user_id, |w| {
            let elapsed = now - w.last_daily_ts;

            if w.last_daily_ts != 0 && elapsed < DAILY_COOLDOWN_SECS {
                let remaining = DAILY_COOLDOWN_SECS - elapsed;
                return Err(WalletError::DailyOnCooldown {
                    remaining_secs: remaining,
                });
            }

            w.tabs += DAILY_REWARD;
            w.last_daily_ts = now;

            Ok(w.tabs)
        })
        .await;

    match res {
        Ok(new_balance) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("✅ You claimed **{DAILY_REWARD} Tabs**! You now have **{new_balance} Tabs**."))
                    .ephemeral(true),
            ).await?;
        }
        Err(WalletError::DailyOnCooldown { remaining_secs }) => {
            // nice formatting
            let hrs = remaining_secs / 3600;
            let mins = (remaining_secs % 3600) / 60;

            ctx.send(
                poise::CreateReply::default()
                    .content(format!(
                        "⏳ You already claimed your daily Tabs. Try again in **{hrs}h {mins}m**."
                    ))
                    .ephemeral(true),
            )
            .await?;
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

/// Opens the shop.
#[poise::command(slash_command)]
pub async fn shop(ctx: Context<'_>) -> Result {
    Ok(())
}

#[poise::command(slash_command, subcommands("preview", "set"))]
pub async fn color(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// previews a specific hexadecimal color
#[poise::command(slash_command)]
pub async fn preview(
    ctx: Context<'_>,

    #[description = "User to show pfp of"] color: String,
) -> Result {
    let trimmed = color.strip_prefix("0x").unwrap_or(&color);

    let color = poise::serenity_prelude::Colour::new(
        u32::from_str_radix(trimmed, 16).map_err(|_| ColorError::IncorrectFormat)?,
    );
    let size: u32 = 256;
    let mut img = image::RgbaImage::new(size, size);
    for px in img.pixels_mut() {
        *px = image::Rgba([color.r(), color.g(), color.b(), 255]);
    }

    // Encode as PNG into bytes
    let mut png_bytes = Vec::new();
    {
        use image::ColorType;
        use image::codecs::png::PngEncoder;

        let encoder = PngEncoder::new(&mut png_bytes);

        encoder.write_image(&img, size, size, ColorType::Rgba8.into())?;
    }

    let filename = "color.png";
    let attachment = serenity::CreateAttachment::bytes(png_bytes, filename);

    // TODO: meow~

    ctx.send(
        poise::CreateReply::default().attachment(attachment).embed(
            serenity::CreateEmbed::default()
                .title("Color Preview")
                .description(color.hex())
                .color(color),
        ),
    )
    .await?;

    Ok(())
}

/// sets your role and role color to your choice. Costs 10 tabs to do so.
#[poise::command(slash_command)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "Name of your role."] name: String,
    #[description = "Color of your role."] color: String,
) -> Result {
    const FIZZ_ROLE_ID: RoleId = RoleId::new(1441247546163990651);
    const ADMIN_ROLE_ID: RoleId = RoleId::new(1441247113282326709);
    let user_id = ctx.author().id;
    let guild_id = ctx.guild_id().unwrap();

    ctx.data()
        .with_wallet_user_read(user_id, |user| {
            if user.tabs < 10 {
                return Err(WalletError::NotEnoughTabs);
            }
            Ok(())
        })
        .await?;

    let trimmed = color.strip_prefix("0x").unwrap_or(&color);

    let color = poise::serenity_prelude::Colour::new(
        u32::from_str_radix(trimmed, 16).map_err(|_| ColorError::IncorrectFormat)?,
    );
    let member = ctx.author_member().await.unwrap();

    if let Some(mut r) = member.roles(ctx.cache()).and_then(|v| {
        v.into_iter()
            .filter(|r| r.id != FIZZ_ROLE_ID && r.id != ADMIN_ROLE_ID)
            .next_back()
    }) {
        r.edit(ctx.http(), EditRole::new().colour(color).name(&name))
            .await?;
    }
    //ok.. we need to create a role.
    let r = guild_id
        .create_role(ctx.http(), EditRole::new().colour(color).name(name))
        .await?;

    member.add_role(ctx.http(), r.id).await?;

    ctx.send(Reply::default().embed(utils::create_embed_builder(
        "Set Color",
        "Your color has been set!",
        EmbedType::Good,
    )))
    .await?;

    Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    log::warn!("Debug register command called!!!");
    Ok(())
}
