use std::ffi::FromBytesUntilNulError;
use std::io::Error;
use std::sync::{MutexGuard, PoisonError};
use thiserror::Error;

use crate::fullscrn::ResolutionInfo;
use crate::options::OptionsStruct;
use crate::translations::TranslationError;
use crate::MainError;

#[derive(Error, Debug)]
pub enum RecordLoadError {
    #[error("File signature does not match!")]
    IncorrectFileSignature,
    #[error("Unable to parse data when reading bytes: `{0}`")]
    RecordParse(#[from] FromBytesUntilNulError),
    #[error("Other error while loading records: `{0}`")]
    Generic(#[from] Error),
    #[error("Incorrectly sized bitmap field")]
    BitmapFieldSize,
    #[error("Bitmap Resolution is out of bounds (<= 2)")]
    BitmapResolutionOob,
    #[error("Unknown error...")]
    Unknown,
    #[error("Field type is out of bounds or is not recognized")]
    InvalidFieldType,
}

#[derive(Error, Debug)]
pub enum PbInitError {
    #[error(transparent)]
    RecordLoadError(#[from] RecordLoadError),
    #[error("Failed to get rc: `{0}`")]
    GetRcError(#[from] TranslationError)
}

#[derive(Error, Debug)]
pub enum GroupDataError {
    #[error("Failed to split spliced bitmap: `{0}`")]
    Split(#[from] PoisonError<MutexGuard<'static, [ResolutionInfo; 3]>>),
    #[error("There was a mismatch between the font widths")]
    FontWidthMismatch,
    #[error("Buffer length is not the correct size")]
    InvalidBufferLength,
}

#[derive(Error, Debug)]
pub enum FullscreenError {
    #[error("Could not find main window to turn into fullscreen")]
    MainWindowMissing,
    #[error("Resolution is out of bounds")]
    ResolutionOutOfBounds,
    #[error("Renderer is missing (possibly none) `{0}`")]
    MissingRenderer(MainError),
    #[error("Faild to lock OPTIONS: `{0}`")]
    OptionsLock(#[from] PoisonError<MutexGuard<'static, OptionsStruct>>),
    #[error("Failed to lock ResolutionArray: `{0}`")]
    ResolutionArrayLock(#[from] PoisonError<MutexGuard<'static, [ResolutionInfo; 3]>>),
    #[error("Failed to lock Scale value: `{0}`")]
    FloatLock(#[from] PoisonError<MutexGuard<'static, f32>>),
}
