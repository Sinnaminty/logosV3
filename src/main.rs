use clap::Parser;
use poise::serenity_prelude as serenity;
use utils::ResultExt;
mod utils;

#[derive(Debug)]
struct Data {} // User data, which is stored and accessible in all command invocations

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(context_menu_command = "Echo", slash_command)]
pub async fn echo(
    ctx: Context<'_>,
    #[description = "Message to echo (enter a link or ID)"] msg: serenity::Message,
) -> Result<(), Error> {
    ctx.say(&msg.content).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    utils::setup_logging(utils::Args::parse().log_level);

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), echo()],
            on_error: |error| {
                Box::pin(async move {
                    match error {
                        poise::FrameworkError::Command { error, ctx, .. } => {
                            let _ = ctx.say(format!("Error in command: {error}")).await;
                        }
                        other => {
                            log::error!("Framework error: {:#?}", other);
                        }
                    }
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(utils::get_api_token(), utils::INTENTS)
        .framework(framework)
        .await
        .unwrap_or_log();

    client.start().await.unwrap_or_log();
}
