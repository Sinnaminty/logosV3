use crate::dectalk::DectalkError;

#[derive(thiserror::Error, Debug)]
pub enum LogosErrors {
    #[error("SerenityError: {0}")]
    Serenity(#[from] poise::serenity_prelude::Error),

    #[error("ffiError: {0}")]
    FfiNul(#[from] std::ffi::NulError),

    #[error("tokio::JoinError: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),

    #[error("tokio::SendError {0}")]
    TokioSend(
        #[from]
        tokio::sync::mpsc::error::SendError<
            crate::pawthos::enums::persistant_data::PersistantData,
        >,
    ),

    #[error("DectalkError: {0}")]
    Dectalk(#[from] DectalkError),

    #[error("std::io: {0}")]
    StdIo(#[from] std::io::Error),
}
