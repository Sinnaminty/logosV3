use crate::{
    dectalk::{Dectalk, WaveFormat},
    types::{Context, Error},
};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, subcommands("say"), subcommand_required)]
pub async fn vox(_: Context<'_>) -> Result<(), Error> {
    //lmao, again
    panic!();
}

#[poise::command(slash_command)]
pub async fn say(
    ctx: Context<'_>,
    #[description = "Text to synthesize"] text: String,
    #[description = "Speaker ID (numeric, optional)"] speaker: Option<u32>,
    #[description = "Language code (e.g. 1=US English)"] language: Option<u32>,
    #[description = "Words per minute"] rate: Option<u32>,
) -> Result<(), Error> {
    let path = tokio::task::spawn_blocking(move || -> Result<std::path::PathBuf, Error> {
        use std::{
            env,
            time::{SystemTime, UNIX_EPOCH},
        };

        // Create and use DECtalk entirely on this blocking thread.
        let tts = Dectalk::new()?;

        if let Some(l) = language {
            let _ = tts.set_language(l);
        }
        if let Some(s) = speaker {
            let _ = tts.set_speaker(s);
        }
        if let Some(r) = rate {
            let _ = tts.set_rate(r);
        }

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
