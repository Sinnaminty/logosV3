use crate::types::Data;
use crate::types::EmbedType;
use crate::types::Error;
use crate::types::Reply;
use crate::utils;
use poise::FrameworkError;
use std::pin::Pin;

pub fn error_handler(
    error: FrameworkError<'_, Data, Error>,
) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
    Box::pin(async move {
        match error {
            poise::FrameworkError::Command { error, ctx, .. } => {
                let embed = utils::create_embed_builder(
                    "ERROR",
                    format!("Error in command: {error}"),
                    EmbedType::Bad,
                );

                let _ = ctx.send(Reply::default().embed(embed)).await;
            }
            other => {
                log::error!("Framework error: {other:#?}",);
            }
        }
    })
}
