//! `/vox` command suite — DECtalk text-to-speech synthesis.
//!
//! DECtalk runs synchronous blocking C calls that must not execute on the
//! async executor. Each invocation creates a fresh [`Dectalk`] instance inside
//! [`tokio::task::spawn_blocking`], synthesises audio to a timestamped temp
//! file, and uploads the resulting WAV to Discord before cleaning up.

use crate::dectalk::{Dectalk, WaveFormat};
use crate::pawthos::types::{Context, Result};
use poise::serenity_prelude as serenity;

/// Voice synthesis commands powered by DECtalk.
///
/// This is a parent command; use `/vox say` to synthesise speech.
#[poise::command(slash_command, subcommands("say"), subcommand_required)]
pub async fn vox(_: Context<'_>) -> Result {
    //lmao, again
    panic!();
}

/// Synthesise text as speech using the DECtalk TTS engine and post the WAV.
///
/// The audio is generated on a blocking thread (via [`tokio::task::spawn_blocking`])
/// to avoid stalling the async executor during the synchronous DECtalk calls.
/// The resulting WAV file is attached to the reply and then deleted from disk.
///
/// DECtalk supports its own markup language for controlling prosody, pitch,
/// and speaking rate — e.g. `[:rate 200]` sets the words-per-minute.
#[poise::command(slash_command)]
pub async fn say(ctx: Context<'_>, #[description = "Text to synthesize"] text: String) -> Result {
    let path = tokio::task::spawn_blocking(move || -> Result<std::path::PathBuf> {
        use std::{
            env,
            time::{SystemTime, UNIX_EPOCH},
        };

        // Create and use DECtalk entirely on this blocking thread.
        let tts = Dectalk::new()?;

        // Pick a temp path using a millisecond timestamp to avoid collisions
        // when multiple users invoke /vox say concurrently.
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

    // Best-effort cleanup — ignore errors (the OS will reclaim the file on exit).
    let _ = std::fs::remove_file(path);
    Ok(())
}
