use crate::types::Data;
use crate::types::Error;
use poise::FrameworkError;
use std::pin::Pin;

pub fn error_handler(
    error: FrameworkError<'_, Data, Error>,
) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
    Box::pin(async move {
        match error {
            poise::FrameworkError::Command { error, ctx, .. } => {
                let _ = ctx.say(format!("Error in command: {error}")).await;
            }
            other => {
                log::error!("Framework error: {other:#?}",);
            }
        }
    })
}
