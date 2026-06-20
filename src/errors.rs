use std::ffi::{FromBytesUntilNulError, FromBytesWithNulError, NulError};
use std::io::Error;
use std::sync::{Arc, MutexGuard, PoisonError};
use thiserror::Error;

use crate::fullscrn::ResolutionInfo;
use crate::group_data::DatFile;
use crate::loader::SoundListStruct;
use crate::render::RenderError;
use crate::score::ScoreMessageFontType;
use crate::sound::SoundError;
use crate::t_pinball_table::PinballTableError;
use crate::timer::TimerError;
use crate::translations::TranslationError;

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
pub enum ScoreError {
    #[error("Failed to lock RecordTable from PB: `{0}`")]
    RecordTableLock(#[from] PoisonError<MutexGuard<'static, Option<Arc<DatFile>>>>),
    #[error("Failed to lock MSG_FONTP from Score: `{0}`")]
    MsgFontLock(#[from] PoisonError<MutexGuard<'static, Option<ScoreMessageFontType>>>),
}

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Failed to lock LOADER_TABLE")]
    TableLock(#[from] PoisonError<MutexGuard<'static, Option<Arc<DatFile>>>>),
    #[error("Failed to lock SOUND_LIST")]
    SoundListLock(#[from] PoisonError<MutexGuard<'static, [SoundListStruct; 65]>>),
    #[error("Failed to lock SOUND_COUNT")]
    SoundCountLock(#[from] PoisonError<MutexGuard<'static, i32>>),
    #[error(transparent)]
    FromBytesWithNul(#[from] FromBytesWithNulError),
    #[error(transparent)]
    SoundError(#[from] SoundError),
}

#[derive(Error, Debug)]
pub enum PbError {
    #[error(transparent)]
    RecordLoadError(#[from] RecordLoadError),
    #[error(transparent)]
    LoaderError(#[from] LoaderError),
    #[error("Failed to convert string: `{0}`")]
    FailedStrConversion(#[from] NulError),
    #[error(transparent)]
    ScoreError(#[from] ScoreError),
    #[error(transparent)]
    RenderError(#[from] RenderError),
    #[error(transparent)]
    TimerError(#[from] TimerError),
    #[error("No textbox found...")]
    NoTextBox,
    #[error(transparent)]
    TranslationError(#[from] TranslationError),
    #[error("Error creating PinballTable: `{0}`")]
    PinballTableError(#[from] PinballTableError),
    #[error("We could find the pinball table")]
    NoTable,
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
    #[error("Renderer is missing (possibly none)")]
    MissingRenderer,
    #[error("Failed to lock Scale value: `{0}`")]
    FloatLock(#[from] PoisonError<MutexGuard<'static, f32>>),
}

#[derive(Debug, Error)]
pub enum TTextBoxError {
    #[error("Failure creating new TTextBox")]
    New,
    #[error("Failure to load dimensions from loader `{0}`")]
    DimensionLoading(#[from] LoaderError),
}

#[derive(Error, Debug)]
pub enum MainLoopError {
    #[error("Failed to lock Mutex")]
    MutexLock,
    #[error(transparent)]
    NulError(#[from] NulError),
    #[error("There is no MainWindow to attach to...")]
    NullWindow,
    #[error(transparent)]
    FullScreen(#[from] FullscreenError),
    #[error(transparent)]
    Translation(#[from] TranslationError),
    #[error(transparent)]
    PbError(#[from] PbError),
}
