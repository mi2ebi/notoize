use itertools::Itertools;
use serde::Deserialize;
use std::{collections::HashMap, fs};

pub struct NotoizeConfig {
    pub prefer_ui: bool,
    pub prefer_cjk: bool,
    pub prefer_math: bool,
    //
    pub lgc: Vec<Serifness>,
    pub armenian: Vec<Serifness>,
    pub balinese: Vec<Serifness>,
    pub bengali: Vec<Serifness>,
    pub devanagari: Vec<Serifness>,
    pub ethiopic: Vec<Serifness>,
    pub georgian: Vec<Serifness>,
    pub grantha: Vec<Serifness>,
    pub gujarati: Vec<Serifness>,
    pub gurmukhi: Vec<Serifness>,
    pub kannada: Vec<Serifness>,
    pub khmer: Vec<Serifness>,
    pub khojki: Vec<Serifness>,
    pub malayalam: Vec<Serifness>,
    pub myanmar: Vec<Serifness>,
    pub oriya: Vec<Serifness>,
    pub sinhala: Vec<Serifness>,
    pub tamil: Vec<Serifness>,
    pub telugu: Vec<Serifness>,
    pub vithkuqi: Vec<Serifness>,
    // not just sans/serif
    pub adlam: Vec<AdlamNkoCfg>,
    pub nko: Vec<AdlamNkoCfg>,
    pub arabic: Vec<ArabicCfg>,
    pub hebrew: Vec<HebrewCfg>,
    pub khitan: Vec<KhitanCfg>,
    pub nushu: Vec<NushuCfg>,
    pub syriac: Vec<SyriacCfg>,
    pub thai: Vec<ThaiLaoCfg>,
    pub lao: Vec<ThaiLaoCfg>,
    // cjk
    pub cjk: Vec<(Serifness, CjkVariant)>,
}
impl NotoizeConfig {
    pub fn new_sans() -> Self {
        Self {
            prefer_ui: false,
            prefer_cjk: false,
            prefer_math: false,
            lgc: vec![Serifness::Sans],
            armenian: vec![Serifness::Sans],
            balinese: vec![Serifness::Sans],
            bengali: vec![Serifness::Sans],
            devanagari: vec![Serifness::Sans],
            ethiopic: vec![Serifness::Sans],
            georgian: vec![Serifness::Sans],
            grantha: vec![Serifness::Sans],
            gujarati: vec![Serifness::Sans],
            gurmukhi: vec![Serifness::Sans],
            kannada: vec![Serifness::Sans],
            khmer: vec![Serifness::Sans],
            khojki: vec![Serifness::Sans],
            malayalam: vec![Serifness::Sans],
            myanmar: vec![Serifness::Sans],
            oriya: vec![Serifness::Sans],
            sinhala: vec![Serifness::Sans],
            tamil: vec![Serifness::Sans],
            telugu: vec![Serifness::Sans],
            vithkuqi: vec![Serifness::Sans],
            //
            adlam: vec![AdlamNkoCfg::Sans],
            nko: vec![AdlamNkoCfg::Sans],
            arabic: vec![ArabicCfg::Sans],
            hebrew: vec![HebrewCfg::Sans],
            khitan: vec![KhitanCfg::Serif],
            nushu: vec![NushuCfg::Sans],
            syriac: vec![SyriacCfg::Sans],
            thai: vec![ThaiLaoCfg::SansUnlooped],
            lao: vec![ThaiLaoCfg::SansUnlooped],
            //
            cjk: vec![(Serifness::Sans, CjkVariant::Sc)],
        }
    }
    pub fn prefer_serif() -> Self {
        Self {
            lgc: vec![Serifness::Serif],
            armenian: vec![Serifness::Serif],
            balinese: vec![Serifness::Serif],
            bengali: vec![Serifness::Serif],
            devanagari: vec![Serifness::Serif],
            ethiopic: vec![Serifness::Serif],
            georgian: vec![Serifness::Serif],
            grantha: vec![Serifness::Serif],
            gujarati: vec![Serifness::Serif],
            gurmukhi: vec![Serifness::Serif],
            kannada: vec![Serifness::Serif],
            khmer: vec![Serifness::Serif],
            khojki: vec![Serifness::Serif],
            malayalam: vec![Serifness::Serif],
            myanmar: vec![Serifness::Serif],
            oriya: vec![Serifness::Serif],
            sinhala: vec![Serifness::Serif],
            tamil: vec![Serifness::Serif],
            telugu: vec![Serifness::Serif],
            vithkuqi: vec![Serifness::Serif],
            //
            arabic: vec![ArabicCfg::Naskh],
            hebrew: vec![HebrewCfg::Serif],
            khitan: vec![KhitanCfg::Serif],
            thai: vec![ThaiLaoCfg::Serif],
            lao: vec![ThaiLaoCfg::Serif],
            //
            cjk: vec![(Serifness::Serif, CjkVariant::Sc)],
            ..Self::new_sans()
        }
    }
}

#[derive(PartialEq)]
pub enum Serifness {
    Sans,
    Serif,
}
#[derive(PartialEq)]
pub enum AdlamNkoCfg {
    Sans,
    Unjoined,
}
#[derive(PartialEq)]
pub enum ArabicCfg {
    Sans,
    Kufi,
    Naskh,
    Nastaliq,
}
#[derive(PartialEq)]
pub enum HebrewCfg {
    Sans,
    Serif,
    Rashi,
}
#[derive(PartialEq)]
pub enum KhitanCfg {
    Serif,
    Vertical,
    Rotated,
}
#[derive(PartialEq)]
pub enum NushuCfg {
    Sans,
    Traditional,
}
#[derive(PartialEq)]
pub enum SyriacCfg {
    Sans,
    Western,
    Eastern,
}
#[derive(PartialEq)]
pub enum ThaiLaoCfg {
    SansLooped,
    SansUnlooped,
    Serif,
}
#[derive(PartialEq)]
pub enum CjkVariant {
    Sc,
    Tc,
    Hk,
    Jp,
    Kr,
}

pub struct FontStack(pub Vec<String>);

#[derive(Debug)]
pub struct Font {
    pub filename: String,
    pub fontname: String,
    pub bytes: Vec<u8>,
}

impl FontStack {
    pub fn files(&self) -> Vec<Font> {
        self.0
            .clone()
            .iter()
            .map(|x| Font {
                filename: "NotoSans-Regular.otf".to_string(),
                fontname: x.to_string(),
                bytes: fs::read("notofonts.github.io/fonts/NotoSans/full/otf/NotoSans-Regular.otf")
                    .unwrap(),
            })
            .collect()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlockData {
    cps: HashMap<String, CodepointFontSupport>,
    fonts: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CodepointFontSupport {
    fonts: Option<Vec<String>>,
}

fn preferred(p: bool, a: bool, b: bool) -> bool {
    (!a || p) && (!b || !p)
}

/// Returns a minimal font stack for rendering `text`
pub fn notoize(text: &str, config: NotoizeConfig) -> Vec<String> {
    let font_support = (0..=323)
        .map(|i| {
            serde_json::from_str::<BlockData>(
                &fs::read_to_string(format!("overview/blocks/block-{i:03}.json")).unwrap(),
            )
            .unwrap()
        })
        .flat_map(move |e| {
            e.cps
                .iter()
                .map(move |(k, v)| {
                    (
                        k.clone(),
                        match e.fonts.clone() {
                            None => v.fonts.clone().unwrap_or(vec![]),
                            Some(f) => f,
                        },
                    )
                })
                .collect::<HashMap<_, _>>()
        })
        .map(|(k, v)| (k.parse::<u32>().unwrap(), v.clone()))
        .sorted_by_key(|&(k, _)| k)
        .collect_vec();
    let mut fonts = Vec::new();
    for c in text.chars() {
        let codepoint = c as u32;
        let hex = format!("{codepoint:04x}");
        let mut f = font_support
            .iter()
            .find(|(n, _)| n == &codepoint)
            .unwrap_or(&(codepoint, vec![]))
            .1
            .clone();
        for e in &f {
            if !fonts.contains(&format!("Noto {e}"))
                && e != "Sans Mono"
                && !e.contains("Display")
                && (config.lgc.iter().any(|s| match s {
                    Serifness::Sans => e.ends_with("Sans"),
                    Serifness::Serif => e.ends_with("Serif"),
                }) || config.arabic.iter().any(|s| match s {
                    ArabicCfg::Kufi => e.contains("Kufi"),
                    ArabicCfg::Naskh => e.contains("Naskh"),
                    ArabicCfg::Nastaliq => e.contains("Nastaliq"),
                    ArabicCfg::Sans => e.contains("Sans Arabic"),
                }))
                && preferred(
                    config.prefer_ui,
                    e.ends_with(" UI"),
                    f.iter().any(|x| x == &format!("{e} UI")),
                )
                && preferred(
                    config.prefer_math,                             // p
                    e == "Sans Math",                               // a
                    f.iter().any(|x| x == "Sans Symbols"), // b
                )
            {
                fonts.push(format!("Noto {e}"));
            }
        }
        println!("{hex} {f:?}");
    }
    fonts
}
