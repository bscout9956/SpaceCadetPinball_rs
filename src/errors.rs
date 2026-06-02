use std::ffi::FromBytesUntilNulError;
use std::io::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecordLoadError {
    #[error("File signature does not match!")]
    IncorrectFileSignatureError,
    #[error("Unable to parse data when reading bytes: `{0}`")]
    RecordParseError(#[from] FromBytesUntilNulError),
    #[error("Other error while loading records: `{0}`")]
    GenericError(#[from] Error),
    #[error("Incorrectly sized bitmap field")]
    BitmapFieldSizeError,
    #[error("Bitmap Resolution is out of bounds (<= 2)")]
    BitmapResolutionOobError,
    #[error("Unknown error...")]
    UnknownError,
}
