#[derive(thiserror::Error, Debug)]
pub enum ColorError {
    #[error("Color has incorrect format!")]
    IncorrectFormat,
    #[error("ImageError!: {0}")]
    ImageError(image::ImageError),
}
