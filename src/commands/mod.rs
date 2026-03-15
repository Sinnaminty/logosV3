//! Top-level command registry and miscellaneous slash/prefix commands.
//!
//! This module owns:
//! - [`return_commands`] — the list of all commands registered with the framework.
//! - Utility commands: `help`, `pfp`, `daily`, `balance`.
//! - The `/color` command group (`preview` + `set`).
//! - Admin prefix commands (`register`, `give_tabs`, `fix_color_role_names`).
//!
//! Feature-specific command groups live in their own sub-modules:
//! - [`mimic`] — webhook-based persona impersonation.
//! - [`schedule`] — timezone-aware event reminders.
//! - [`vox`] — DECtalk text-to-speech synthesis.

use crate::commands::{mimic::*, schedule::*, vox::*};
use crate::pawthos::consts::{COLOR_PREVIEW_SIZE, COLOR_ROLE_COST, DAILY_REWARD, FIZZ_ID, TAB_EMOJI};
use crate::pawthos::enums::color_errors::ColorError;
use crate::pawthos::{
    enums::embed_type::EmbedType,
    structs::data::Data,
    types::{Context, Error, Reply, Result},
};
use crate::utils::{self};
use image::ImageEncoder;
use poise::serenity_prelude::{self as serenity, EditRole, RoleId, User};
mod mimic;
mod schedule;
mod vox;

/// Register all commands with the Poise framework.
///
/// Slash commands are registered globally in [`crate::framework::setup_framework`].
/// To add a new command, create it in this file (or a sub-module) and append
/// it to the `vec!` here.
pub fn return_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        help(),
        daily(),
        balance(),
        pfp(),
        register(),
        give_tabs(),
        vox(),
        mimic(),
        schedule(),
        color(),
        fix_color_role_names(),
    ]
}

// ---------------------------------------------------------------------------
// General-purpose commands
// ---------------------------------------------------------------------------

/// Show help text for the bot or a specific command.
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

/// Display a user's profile picture as a large embed image.
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

/// Claim your daily tab reward (10 tabs, once every 24 hours).
///
/// The response is ephemeral so only you can see it. The daily window resets
/// at midnight local time; the cooldown message tells you exactly how long
/// remains if you've already claimed.
#[poise::command(slash_command)]
pub async fn daily(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;

    let balance = ctx.data().wallet_user_daily(user_id).await?;

    ctx.send(
                poise::CreateReply::default()
                    .content(format!("✅ You claimed **{DAILY_REWARD} {TAB_EMOJI}**! You now have **{balance} {TAB_EMOJI}**.",),)
                    .ephemeral(true),
            ).await?;
    Ok(())
}

/// Check your current tab balance (ephemeral — only you can see it).
#[poise::command(slash_command)]
pub async fn balance(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;
    let balance = ctx
        .data()
        .with_wallet_user_read(user_id, |user| Ok(user.tabs))
        .await?;

    ctx.send(
        poise::CreateReply::default()
            .content(format!(
                "You have **{balance} {TAB_EMOJI}!**"
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Color commands
// ---------------------------------------------------------------------------

/// Colour-related commands: preview a hex colour, or purchase a custom role.
#[poise::command(slash_command, subcommands("preview", "set"))]
pub async fn color(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// Preview what a hex colour looks like as a 256×256 PNG swatch.
///
/// Accepts bare hex (`FF8800`) or `0x`-prefixed (`0xFF8800`). The image is
/// attached directly to the response so you can see the exact colour before
/// committing to buying a role with it.
#[poise::command(slash_command)]
pub async fn preview(
    ctx: Context<'_>,
    #[description = "Hex code of the color you want to preview."] color: String,
) -> Result {
    let trimmed = color.strip_prefix("0x").unwrap_or(&color);

    let color = poise::serenity_prelude::Colour::new(
        u32::from_str_radix(trimmed, 16).map_err(|_| ColorError::IncorrectFormat)?,
    );
    let mut img = image::RgbaImage::new(COLOR_PREVIEW_SIZE, COLOR_PREVIEW_SIZE);
    for px in img.pixels_mut() {
        *px = image::Rgba([color.r(), color.g(), color.b(), 255]);
    }

    // Encode as PNG into bytes
    let mut png_bytes = Vec::new();
    {
        use image::ColorType;
        use image::codecs::png::PngEncoder;

        let encoder = PngEncoder::new(&mut png_bytes);

        encoder.write_image(&img, COLOR_PREVIEW_SIZE, COLOR_PREVIEW_SIZE, ColorType::Rgba8.into())?;
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

/// Set your custom colour role name and colour for [`COLOR_ROLE_COST`] tabs.
///
/// Custom roles are identified by a leading zero-width space (`\u{200B}`) in
/// their name, which keeps them distinct from normal server roles. If you
/// already have a colour role it is updated in-place; otherwise a new role is
/// created and assigned to you.
///
/// The tab charge only happens *after* the Discord API calls succeed, so a
/// failed role creation never costs you tabs.
///
/// **Special case:** a colour value of `#000000` (pure black) is silently
/// converted to `rgb(1, 1, 1)` because Discord treats role colour `0` as
/// "no colour" and renders it as the default text colour instead of black.
#[poise::command(slash_command, guild_only)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "Name of your role."] name: String,
    #[description = "Color of your role."] color: String,
) -> Result {
    let user_id = ctx.author().id;
    let guild_id = ctx.guild_id().unwrap();
    // rebinding name with zero-width
    let name = '\u{200B}'.to_string() + &name;

    log::debug!("inputted name: {name}");

    let trimmed = color.strip_prefix("0x").unwrap_or(&color);

    let color_integer =
        u32::from_str_radix(trimmed, 16).map_err(|_| ColorError::IncorrectFormat)?;

    let color = if color_integer == 0 {
        poise::serenity_prelude::Colour::from_rgb(1, 1, 1)
    } else {
        poise::serenity_prelude::Colour::new(color_integer)
    };

    // Do all guild API work first — only charge tabs on success.
    let member = guild_id.member(ctx.http(), user_id).await?;
    let member_role_ids = member.roles.clone();
    let guild_roles = guild_id.roles(ctx.http()).await?;
    let member_roles = member_role_ids
        .iter()
        .filter_map(|r| guild_roles.get(r))
        .collect::<Vec<_>>();

    // right.. so let's try to use a zero-width space to determine if this is a color role or not.
    if let Some(mut r) = member_roles
        .into_iter()
        .filter(|r| r.name.starts_with('\u{200B}'))
        .cloned()
        .next_back()
    {
        log::debug!("role already exists! {}", r.name);
        r.edit(ctx.http(), EditRole::new().colour(color).name(&name))
            .await?;
    } else {
        //ok.. we need to create a role.
        let r = guild_id
            .create_role(ctx.http(), EditRole::new().colour(color).name(name))
            .await?;
        log::debug!("makin new role! {}", r.name);

        member.add_role(ctx.http(), r.id).await?;
    }

    let tabs = ctx
        .data()
        .with_wallet_user_write(user_id, |user| user.remove_tabs(COLOR_ROLE_COST))
        .await?;

    ctx.send(utils::reply_ok(
        "Set Color",
        format!("Your color has been set! You now have **{tabs} {TAB_EMOJI}!**"),
    ))
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Admin prefix commands (owner-only)
// ---------------------------------------------------------------------------

/// Register slash commands globally (owner-only, prefix command).
///
/// Opens the interactive Poise registration UI. Only works for `FIZZ_ID`;
/// silently does nothing for anyone else.
#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result {
    if ctx.author().id != FIZZ_ID {
        return Ok(());
    }

    poise::builtins::register_application_commands_buttons(ctx).await?;
    log::warn!("Debug register command called!!!");
    Ok(())
}

/// Give tabs to any user (owner-only, prefix command).
///
/// Usage: `!give_tabs @user 50`
#[poise::command(prefix_command)]
pub async fn give_tabs(ctx: Context<'_>, user: User, tabs: i64) -> Result {
    if ctx.author().id != FIZZ_ID {
        return Ok(());
    }
    let user_id = user.id;
    ctx.data()
        .with_wallet_user_write(user_id, |user| {
            user.add_tabs(tabs);
            Ok(())
        })
        .await?;

    log::warn!("Gave {} tabs to {}!", tabs, user.name);
    Ok(())
}

/// Retrofit old colour roles with the zero-width-space name prefix (owner-only).
///
/// Custom colour roles are identified by a leading `\u{200B}` in their name.
/// This one-off utility command adds that prefix to an existing role that was
/// created before the convention was introduced.
///
/// Usage: `!fix_color_role_names <role_id>`
#[poise::command(prefix_command)]
pub async fn fix_color_role_names(ctx: Context<'_>, role_id: u64) -> Result {
    if ctx.author().id != FIZZ_ID {
        return Ok(());
    }

    let mut role = ctx
        .guild_id()
        .unwrap()
        .role(ctx.http(), RoleId::new(role_id))
        .await?;

    if role.name.contains('\u{200B}') {
        log::warn!("BRO.. this shit already GOT a fuckin THING!!!");
        return Ok(());
    }
    let name = '\u{200B}'.to_string() + &role.name;

    role.edit(ctx.http(), EditRole::new().name(&name)).await?;
    log::warn!("fixed role name{name}");
    Ok(())
}
