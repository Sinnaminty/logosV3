use crate::dectalk::DectalkError;
use crate::pawthos::enums::color_errors::ColorError;
use crate::pawthos::enums::mimic_errors::MimicError;
use crate::pawthos::enums::schedule_errors::ScheduleError;
use crate::pawthos::enums::wallet_errors::WalletError;

#[derive(thiserror::Error, Debug)]
pub enum PawthosErrors {
    #[error("SerenityError: {0}")]
    Serenity(#[from] poise::serenity_prelude::Error),

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

    #[error("MimicError: {0}")]
    Mimic(#[from] MimicError),

    #[error("ScheduleError: {0}")]
    Schedule(#[from] ScheduleError),

    #[error("WalletError: {0}")]
    Wallet(#[from] WalletError),

    #[error("ColorError: {0}")]
    Color(#[from] ColorError),
}

impl From<chrono::ParseError> for PawthosErrors {
    fn from(value: chrono::ParseError) -> Self {
        PawthosErrors::Schedule(ScheduleError::ParseError(value))
    }
}

impl From<image::ImageError> for PawthosErrors {
    fn from(value: image::ImageError) -> Self {
        PawthosErrors::Color(ColorError::ImageError(value))
    }
}
