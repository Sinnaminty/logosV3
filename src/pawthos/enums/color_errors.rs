#[derive(Debug)]

pub enum ColorError {
    IncorrectFormat,
    ImageError(image::ImageError),
}

impl std::fmt::Display for ColorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorError::IncorrectFormat => {
                write!(f, "Color has incorrect format!")
            }
            ColorError::ImageError(e) => {
                write!(f, "ImageError!: {e}")
            }
        }
    }
}

impl std::error::Error for ColorError {}
