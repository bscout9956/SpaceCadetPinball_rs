use crate::errors::TranslationError;
use crate::text_array::TEXT_ARRAY;
use num_derive::FromPrimitive;
use std::cmp::PartialEq;
use std::sync::{LazyLock, Mutex};

#[derive(Debug, Clone, PartialEq, Eq, Copy, FromPrimitive)]
pub enum Msg {
    Min,
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
    Max,
}

#[derive(Copy, Clone, PartialEq, Default)]
pub enum Lang {
    Min,
    Arabic,
    Czech,
    Danish,
    German,
    Greek,
    #[default]
    English,
    Spanish,
    Finnish,
    French,
    Hebrew,
    Hungarian,
    Italian,
    Japanese,
    Korean,
    Norwegian,
    Dutch,
    Polish,
    BrazilianPortuguese,
    Portuguese,
    Russian,
    Swedish,
    Turkish,
    SimplifiedChinese,
    TraditionalChinese,
    Max,
}

#[derive(Copy, Clone, PartialEq)]
pub struct LanguageInfo {
    pub short_name: &'static str,
    pub display_name: &'static str,
    pub language: Lang,
}

pub(crate) static LANGUAGES: [LanguageInfo; Lang::Max as usize] = [
    LanguageInfo {
        short_name: "ar",
        display_name: "Arabic",
        language: Lang::Arabic,
    },
    LanguageInfo {
        short_name: "cz",
        display_name: "Czech",
        language: Lang::Czech,
    },
    LanguageInfo {
        short_name: "da",
        display_name: "Danish",
        language: Lang::Danish,
    },
    LanguageInfo {
        short_name: "de",
        display_name: "German",
        language: Lang::German,
    },
    LanguageInfo {
        short_name: "el",
        display_name: "Greek",
        language: Lang::Greek,
    },
    LanguageInfo {
        short_name: "en",
        display_name: "English",
        language: Lang::English,
    },
    LanguageInfo {
        short_name: "es",
        display_name: "Spanish",
        language: Lang::Spanish,
    },
    LanguageInfo {
        short_name: "fi",
        display_name: "Finnish",
        language: Lang::Finnish,
    },
    LanguageInfo {
        short_name: "fr",
        display_name: "French",
        language: Lang::French,
    },
    LanguageInfo {
        short_name: "he",
        display_name: "Hebrew",
        language: Lang::Hebrew,
    },
    LanguageInfo {
        short_name: "hu",
        display_name: "Hungarian",
        language: Lang::Hungarian,
    },
    LanguageInfo {
        short_name: "it",
        display_name: "Italian",
        language: Lang::Italian,
    },
    LanguageInfo {
        short_name: "ja",
        display_name: "Japanese",
        language: Lang::Japanese,
    },
    LanguageInfo {
        short_name: "ko",
        display_name: "Korean",
        language: Lang::Korean,
    },
    LanguageInfo {
        short_name: "nb",
        display_name: "Norwegian",
        language: Lang::Norwegian,
    },
    LanguageInfo {
        short_name: "nl",
        display_name: "Dutch",
        language: Lang::Dutch,
    },
    LanguageInfo {
        short_name: "pl",
        display_name: "Polish",
        language: Lang::Polish,
    },
    LanguageInfo {
        short_name: "pt_BR",
        display_name: "Brazilian Portuguese",
        language: Lang::BrazilianPortuguese,
    },
    LanguageInfo {
        short_name: "pt_PT",
        display_name: "Portuguese",
        language: Lang::Portuguese,
    },
    LanguageInfo {
        short_name: "ru",
        display_name: "Russian",
        language: Lang::Russian,
    },
    LanguageInfo {
        short_name: "sv",
        display_name: "Swedish",
        language: Lang::Swedish,
    },
    LanguageInfo {
        short_name: "tr",
        display_name: "Turkish",
        language: Lang::Turkish,
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
        language: Lang::Min,
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
    if short_name.is_empty() {
        let mut lang = CURRENT_LANGUAGE.lock().unwrap();
        *lang = Lang::English;
        return;
    }

    for lang_info in LANGUAGES.iter() {
        if lang_info.short_name == short_name {
            let mut curr_lang = CURRENT_LANGUAGE.lock().unwrap();
            *curr_lang = lang_info.language;
            return;
        }
    }

    panic!("Language not available: {}", short_name);
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
                text = get(id, Lang::English);
                if text.is_err() {
                    Err(TranslationError::MissingEnglishText)
                } else {
                    Ok(text?)
                }
            } else {
                Ok(text?)
            }
        }
        Err(_) => Err(TranslationError::FailedToLockLanguage),
    }
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
