use crate::commands::{mimic::*, schedule::*, vox::*};
use crate::pawthos::consts::{DAILY_REWARD, FIZZ_ID};
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
mod oot;
mod schedule;
mod vox;

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
    let user_id = ctx.author().id;

    let balance = ctx.data().wallet_user_daily(user_id).await?;

    ctx.send(
                poise::CreateReply::default()
                    .content(format!("✅ You claimed **{} <:tab:1459045305084547123>**! You now have **{balance} <:tab:1459045305084547123>**.", DAILY_REWARD),)
                    .ephemeral(true),
            ).await?;
    Ok(())
}

/// Shows how many tabs you have in your wallet.
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
                "You have **{balance} <:tab:1459045305084547123>!**"
            ))
            .ephemeral(true),
    )
    .await?;

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
    #[description = "Hex code of the color you want to preview."] color: String,
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
#[poise::command(slash_command, guild_only)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "Name of your role."] name: String,
    #[description = "Color of your role."] color: String,
) -> Result {
    const COST: i64 = 10;
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
        .with_wallet_user_write(user_id, |user| user.remove_tabs(COST))
        .await?;

    ctx.send(Reply::default().embed(utils::create_embed_builder(
        "Set Color",
        format!("Your color has been set! You now have **{tabs} <:tab:1459045305084547123>!**"),
        EmbedType::Good,
    )))
    .await?;

    Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result {
    if ctx.author().id != FIZZ_ID {
        return Ok(());
    }

    poise::builtins::register_application_commands_buttons(ctx).await?;
    log::warn!("Debug register command called!!!");
    Ok(())
}

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
