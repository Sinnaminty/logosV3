use crate::dectalk::{Dectalk, WaveFormat};
use crate::pawthos::types::{Context, Result};
use poise::serenity_prelude as serenity;

/// Vox: A suite of commands around Dectalk voice synthesis.
#[poise::command(slash_command, subcommands("say"), subcommand_required)]
pub async fn vox(_: Context<'_>) -> Result {
    //lmao, again
    panic!();
}

/// Genereates a sound file with the Dectalk API.
#[poise::command(slash_command)]
pub async fn say(ctx: Context<'_>, #[description = "Text to synthesize"] text: String) -> Result {
    let path = tokio::task::spawn_blocking(move || -> Result<std::path::PathBuf> {
        use std::{
            env,
            time::{SystemTime, UNIX_EPOCH},
        };

        // Create and use DECtalk entirely on this blocking thread.
        let tts = Dectalk::new()?;

        // Pick a temp path
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let p = env::temp_dir().join(format!("dectalk_{ts}.wav"));

        // Synthesize to WAV via DECtalk API

        // NOTE: 4 = mono 16 11kHz
        tts.speak_to_wav(&text, &p, WaveFormat::DT_1M16)?;

        Ok(p)
    })
    .await??;

    let attachment = serenity::CreateAttachment::path(path.clone()).await?;
    ctx.send(poise::CreateReply::default().attachment(attachment))
        .await?;

    let _ = std::fs::remove_file(path);
    Ok(())
}
