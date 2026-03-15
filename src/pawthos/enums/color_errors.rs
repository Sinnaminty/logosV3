//! Error type for colour parsing and image generation.

/// Errors that can occur in the `/color` command group.
#[derive(thiserror::Error, Debug)]
pub enum ColorError {
    /// The supplied colour string was not valid hexadecimal.
    ///
    /// Valid inputs are bare hex (`FF8800`) or `0x`-prefixed (`0xFF8800`).
    #[error("Color has incorrect format!")]
    IncorrectFormat,

    /// The [`image`] crate failed while encoding the preview PNG.
    ///
    /// Wraps the underlying [`image::ImageError`] for display.
    #[error("ImageError!: {0}")]
    ImageError(image::ImageError),
}
