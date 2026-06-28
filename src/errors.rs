use crate::sound::SoundError;
use crate::t_pinball_table::PinballTableError;
use crate::timer::TimerError;
use std::ffi::{FromBytesUntilNulError, FromBytesWithNulError, NulError};
use std::io::Error;
use thiserror::Error;

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
    #[error("Failed to lock RecordTable from PB")]
    RecordTableLock,
    #[error("Failed to lock MSG_FONTP from Score")]
    MsgFontLock,
}

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Failed to lock LOADER_TABLE")]
    TableLock,
    #[error("Failed to lock SOUND_LIST")]
    SoundListLock,
    #[error("Failed to lock SOUND_COUNT")]
    SoundCountLock,
    #[error(transparent)]
    FromBytesWithNul(#[from] FromBytesWithNulError),
    #[error(transparent)]
    SoundError(#[from] SoundError),
    #[error("Array Index out of bounds. Tried to reach for `{0}` but the array length is `{1}`")]
    ArrayIndexOutOfBounds(usize, usize),
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
    TimerError(#[from] TimerError),
    #[error("No textbox found...")]
    NoTextBox,
    #[error(transparent)]
    TranslationError(#[from] TranslationError),
    #[error("Error creating PinballTable: `{0}`")]
    PinballTableError(#[from] PinballTableError),
    #[error("We could find the pinball table")]
    NoTable,
    #[error("Failed set RwLock mode")]
    RwLockError
}

#[derive(Error, Debug)]
pub enum GroupDataError {
    #[error("Failed to split spliced bitmap")]
    Split,
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
    #[error("Failed to lock Scale value")]
    FloatLock,
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

#[derive(Error, Debug)]
pub enum TranslationError {
    #[error("Message id out of bounds")]
    MsgIdOutOfBounds,
    #[error("Language id out of bounds")]
    LangIdOutOfBounds,
    #[error("Failed to acquire lock")]
    FailedToLockLanguage,
    #[error("Missing English text equivalent")]
    MissingEnglishText,
    #[error("String is null: `{0}`")]
    Nul(#[from] NulError),
}
