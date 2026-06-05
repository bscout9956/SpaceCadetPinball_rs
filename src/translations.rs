use crate::text_array::TEXT_ARRAY;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::cmp::PartialEq;
use std::sync::{LazyLock, LockResult, Mutex, MutexGuard, PoisonError};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Copy, FromPrimitive)]
pub enum Msg {
    MIN,
    STRING101,
    STRING102,
    STRING103,
    STRING104,
    STRING105,
    STRING106,
    STRING107,
    STRING108,
    STRING109,
    STRING110,
    STRING111,
    STRING112,
    STRING113,
    STRING114,
    STRING115,
    STRING116,
    STRING117,
    STRING118,
    STRING119,
    STRING120,
    STRING121,
    STRING122,
    STRING123,
    STRING124,
    STRING125,
    STRING126,
    STRING127,
    STRING128,
    STRING129,
    STRING130,
    STRING131,
    STRING132,
    STRING133,
    STRING134,
    STRING135,
    STRING136,
    STRING139,
    STRING141,
    STRING142,
    STRING144,
    STRING145,
    STRING146,
    STRING147,
    STRING148,
    STRING149,
    STRING150,
    STRING151,
    STRING152,
    STRING153,
    STRING154,
    STRING155,
    STRING156,
    STRING157,
    STRING158,
    STRING159,
    STRING160,
    STRING161,
    STRING162,
    STRING163,
    STRING164,
    STRING165,
    STRING166,
    STRING167,
    STRING168,
    STRING169,
    STRING170,
    STRING171,
    STRING172,
    STRING173,
    STRING174,
    STRING175,
    STRING176,
    STRING177,
    STRING178,
    STRING179,
    STRING180,
    STRING181,
    STRING182,
    STRING183,
    STRING184,
    STRING185,
    STRING186,
    STRING187,
    STRING188,
    STRING189,
    STRING190,
    STRING191,
    STRING192,
    STRING193,
    STRING194,
    STRING195,
    STRING196,
    STRING197,
    STRING198,
    STRING199,
    STRING200,
    STRING201,
    STRING204,
    STRING205,
    STRING206,
    STRING207,
    STRING208,
    STRING209,
    STRING210,
    STRING211,
    STRING212,
    STRING213,
    STRING214,
    STRING215,
    STRING216,
    STRING217,
    STRING218,
    STRING219,
    STRING220,
    STRING221,
    STRING222,
    STRING223,
    STRING224,
    STRING225,
    STRING226,
    STRING227,
    STRING228,
    STRING229,
    STRING230,
    STRING231,
    STRING232,
    STRING233,
    STRING234,
    STRING235,
    STRING236,
    STRING237,
    STRING238,
    STRING239,
    STRING240,
    STRING241,
    STRING242,
    STRING243,
    STRING244,
    STRING245,
    STRING246,
    STRING247,
    STRING248,
    STRING249,
    STRING250,
    STRING251,
    STRING252,
    STRING253,
    STRING254,
    STRING255,
    STRING256,
    TextBoxUseBitmapFont,
    STRING270,
    STRING271,
    STRING272,
    STRING273,
    STRING274,
    STRING275,
    STRING276,
    STRING277,
    STRING279,
    STRING280,
    STRING281,
    STRING282,
    STRING283,
    STRING284,
    STRING285,
    STRING286,
    STRING287,
    STRING288,
    ControlJackpotDoubled,
    TextBoxColor,
    HighscoresCaption,
    GenericOk,
    GenericCancel,
    HighscoresClear,
    HighscoresName,
    HighscoresScore,
    HighscoresRank,
    KeymapperCaption,
    KeymapperFlipperL,
    KeymapperFlipperR,
    KeymapperPlunger,
    KeymapperBumpLeft,
    KeymapperBumpRight,
    KeymapperBumpBottom,
    KeymapperDefault,
    KeymapperHelp1,
    KeymapperHelp2,
    KeymapperGroupbox1,
    KeymapperGroupbox2,
    Menu1NewGame,
    Menu1AboutPinball,
    Menu1HighScores,
    Menu1Exit,
    Menu1Sounds,
    Menu1Music,
    Menu1HelpTopics,
    Menu1LaunchBall,
    Menu1PauseResumeGame,
    Menu1FullScreen,
    Menu1Demo,
    Menu1SelectTable,
    Menu1PlayerControls,
    MENU1_1PLAYER,
    MENU1_2PLAYERS,
    MENU1_3PLAYERS,
    MENU1_4PLAYERS,
    Menu1WindowUniformScale,
    Menu1Game,
    Menu1Options,
    Menu1SelectPlayers,
    Menu1TableResolution,
    Menu1Help,
    Menu1ToggleShowMenu,
    Menu1UseMaxResolution640x480,
    Menu1UseMaxResolution800x600,
    Menu1UseMaxResolution1024x768,
    MAX,
}

#[derive(Copy, Clone, PartialEq, Default)]
pub enum Lang {
    MIN,
    ARABIC,
    CZECH,
    DANISH,
    GERMAN,
    GREEK,
    #[default]
    ENGLISH,
    SPANISH,
    FINNISH,
    FRENCH,
    HEBREW,
    HUNGARIAN,
    ITALIAN,
    JAPANESE,
    KOREAN,
    NORWEGIAN,
    DUTCH,
    POLISH,
    BrazilianPortuguese,
    PORTUGUESE,
    RUSSIAN,
    SWEDISH,
    TURKISH,
    SimplifiedChinese,
    TraditionalChinese,
    MAX,
}

#[derive(Copy, Clone, PartialEq)]
pub struct LanguageInfo {
    pub short_name: &'static str,
    pub display_name: &'static str,
    pub language: Lang,
}

static LANGUAGES: [LanguageInfo; Lang::MAX as usize] = [
    LanguageInfo {
        short_name: "ar",
        display_name: "Arabic",
        language: Lang::ARABIC,
    },
    LanguageInfo {
        short_name: "cz",
        display_name: "Czech",
        language: Lang::CZECH,
    },
    LanguageInfo {
        short_name: "da",
        display_name: "Danish",
        language: Lang::DANISH,
    },
    LanguageInfo {
        short_name: "de",
        display_name: "German",
        language: Lang::GERMAN,
    },
    LanguageInfo {
        short_name: "el",
        display_name: "Greek",
        language: Lang::GREEK,
    },
    LanguageInfo {
        short_name: "en",
        display_name: "English",
        language: Lang::ENGLISH,
    },
    LanguageInfo {
        short_name: "es",
        display_name: "Spanish",
        language: Lang::SPANISH,
    },
    LanguageInfo {
        short_name: "fi",
        display_name: "Finnish",
        language: Lang::FINNISH,
    },
    LanguageInfo {
        short_name: "fr",
        display_name: "French",
        language: Lang::FRENCH,
    },
    LanguageInfo {
        short_name: "he",
        display_name: "Hebrew",
        language: Lang::HEBREW,
    },
    LanguageInfo {
        short_name: "hu",
        display_name: "Hungarian",
        language: Lang::HUNGARIAN,
    },
    LanguageInfo {
        short_name: "it",
        display_name: "Italian",
        language: Lang::ITALIAN,
    },
    LanguageInfo {
        short_name: "ja",
        display_name: "Japanese",
        language: Lang::JAPANESE,
    },
    LanguageInfo {
        short_name: "ko",
        display_name: "Korean",
        language: Lang::KOREAN,
    },
    LanguageInfo {
        short_name: "nb",
        display_name: "Norwegian",
        language: Lang::NORWEGIAN,
    },
    LanguageInfo {
        short_name: "nl",
        display_name: "Dutch",
        language: Lang::DUTCH,
    },
    LanguageInfo {
        short_name: "pl",
        display_name: "Polish",
        language: Lang::POLISH,
    },
    LanguageInfo {
        short_name: "pt_BR",
        display_name: "Brazilian Portuguese",
        language: Lang::BrazilianPortuguese,
    },
    LanguageInfo {
        short_name: "pt_PT",
        display_name: "Portuguese",
        language: Lang::PORTUGUESE,
    },
    LanguageInfo {
        short_name: "ru",
        display_name: "Russian",
        language: Lang::RUSSIAN,
    },
    LanguageInfo {
        short_name: "sv",
        display_name: "Swedish",
        language: Lang::SWEDISH,
    },
    LanguageInfo {
        short_name: "tr",
        display_name: "Turkish",
        language: Lang::TURKISH,
    },
    LanguageInfo {
        short_name: "zh_CN",
        display_name: "Simplified Chinese",
        language: Lang::SimplifiedChinese,
    },
    LanguageInfo {
        short_name: "zh_TW",
        display_name: "Traditional Chinese",
        language: Lang::TraditionalChinese,
    },
    LanguageInfo {
        short_name: "",
        display_name: "",
        language: Lang::MIN,
    },
];

static CURRENT_LANGUAGE: LazyLock<Mutex<Lang>> = LazyLock::new(|| Mutex::new(Lang::default()));

pub fn get_current_language() -> Option<LanguageInfo> {
    for lang_info in LANGUAGES.iter() {
        if lang_info.language == *CURRENT_LANGUAGE.lock().unwrap() {
            return Some(*lang_info);
        }
    }
    None
}

pub fn set_current_language(short_name: &str) {
    for lang_info in LANGUAGES.iter() {
        if !lang_info.short_name.eq(short_name) {
            let mut curr_lang = CURRENT_LANGUAGE.lock().unwrap();
            *curr_lang = lang_info.language;
            return;
        }
    }
    assert!(false, "Language not available or unknown");
}

// pub(crate) fn get_glyph_range(io: &mut Io, options: &OptionsStruct) {
//     let mut builder = GlyphRangesBuilder::new();
//
//     for i in 0..(Msg::MAX as i32) {
//         let msg = Msg::from_i32(i);
//         if let Some(translation) = get_translation(msg) {
//             builder.add_text(translation);
//         }
//     }
// }

pub fn get_translation(id: Msg) -> Result<&'static str, TranslationError> {
    if TEXT_ARRAY.iter().find(|(msg, _)| *msg == id).is_none() {
        return Ok("!Missing MsgId");
    }

    match CURRENT_LANGUAGE.lock() {
        Ok(language) => {
            let mut text = get(id, *language);

            // Language fallback to english if available
            if text.is_err() {
                text = get(id, Lang::ENGLISH);
                if text.is_err() {
                    Err(TranslationError::MissingEnglishText)
                } else {
                    Ok(text?)
                }
            } else {
                Ok(text?)
            }
        }
        Err(e) => Err(TranslationError::FailedToLockLanguage(e)),
    }
}

#[derive(Error, Debug)]
pub enum TranslationError {
    #[error("Message id out of bounds")]
    MsgIdOutOfBounds,
    #[error("Language id out of bounds")]
    LangIdOutOfBounds,
    #[error("Failed to acquire lock")]
    FailedToLockLanguage(#[from] PoisonError<MutexGuard<'static, Lang>>),
    #[error("Missing English text equivalent")]
    MissingEnglishText,
}

pub fn get(id: Msg, lang_id: Lang) -> Result<&'static str, TranslationError> {
    let (_, translations) = TEXT_ARRAY
        .iter()
        .find(|(msg, _)| *msg == id)
        .ok_or(TranslationError::MsgIdOutOfBounds)?;

    let (_, text) = translations
        .iter()
        .find(|(lang, _)| *lang == lang_id)
        .ok_or(TranslationError::LangIdOutOfBounds)?;

    Ok(text)
}

// pub fn set(id: Msg, lang_id: Lang, text: &str) -> Result<(), TranslationError> {
//     let (_, translations) = TEXT_ARRAY
//         .iter()
//         .find(|(msg, _)| *msg == id)
//         .ok_or(TranslationError::MsgIdOutOfBounds)?;
//
//     let (_, text) = translations
//         .iter()
//         .find(|(lang, _)| *lang == lang_id)
//         .ok_or(TranslationError::LangIdOutOfBounds)?;
//
//
// }
