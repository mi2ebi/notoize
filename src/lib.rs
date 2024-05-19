use gh_file_curler::{fetch, wrapped_first};
use itertools::Itertools;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Clone)]
pub struct FontStack {
    pub names: Vec<String>,
    pub map: Vec<(u32, Vec<String>)>,
}

#[derive(Debug, Clone)]
pub struct Font {
    pub filename: String,
    pub fontname: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct MapString {
    pub all: String,
    pub conflicts: String,
    pub missing: String,
}

impl FontStack {
    pub fn files(&self) -> Vec<Font> {
        self.names
            .iter()
            .map(|x| {
                let f = if x.contains("CJK") {
                    format!(
                        "Noto{}CJK{}-Regular.otf",
                        x.split_ascii_whitespace().nth(1).unwrap(),
                        x.split_ascii_whitespace().nth(3).unwrap().to_lowercase()
                    )
                } else if [
                    "Noto Color Emoji",
                    "Noto Sans ImpAramaic",
                    "Noto Sans OldSouArab",
                    "Noto Sans OldNorArab",
                    "Noto Sans InsPahlavi",
                    "Noto Sans PsaPahlavi",
                    "Noto Sans OldHung",
                    "Noto Sans Zanabazar",
                    "Noto Sans EgyptHiero",
                    "Noto Sans AnatoHiero",
                ]
                .contains(&x.as_str())
                {
                    match x.as_str() {
                        "Noto Color Emoji" => "NotoColorEmoji.ttf",
                        "Noto Sans ImpAramaic" => "NotoSansImperialAramaic-Regular.ttf",
                        "Noto Sans OldSouArab" => "NotoSansOldSouthArabian-Regular.ttf",
                        "Noto Sans OldNorArab" => "NotoSansOldNorthArabian-Regular.ttf",
                        "Noto Sans InsPahlavi" => "NotoSansInscriptionalPahlavi-Regular.ttf",
                        "Noto Sans PsaPahlavi" => "NotoSansPsalterPahlavi-Regular.ttf",
                        "Noto Sans OldHung" => "NotoSansOldHungarian-Regular.ttf",
                        "Noto Sans Zanabazar" => "NotoSansZanabazarSquare-Regular.ttf",
                        "Noto Sans EgyptHiero" => "NotoSansEgyptianHieroglyphs-Regular.ttf",
                        "Noto Sans AnatoHiero" => "NotoSansAnatolianHieroglyphs-Regular.ttf",
                        _ => panic!("the universe broke, sorry"),
                    }
                    .to_string()
                } else {
                    format!("{}-Regular.ttf", x.replace([' ', '-'], ""))
                };
                eprintln!("fetching {x} ({f})");
                Font {
                    filename: f.clone(),
                    fontname: x.to_string(),
                    bytes: {
                        let path = format!("fonts/{}/hinted/ttf/{f}", f.split('-').next().unwrap());
                        wrapped_first(fetch("notofonts", "notofonts.github.io", &[&path]))
                    }
                    .unwrap_or_else(|e| {
                        if x.contains("CJK") {
                            wrapped_first(fetch(
                                "notofonts",
                                "noto-cjk",
                                &[&format!(
                                    "{}/OTF/{}/{f}",
                                    x.split_ascii_whitespace().nth(1).unwrap(),
                                    {
                                        let var = x
                                            .split_ascii_whitespace()
                                            .nth(3)
                                            .unwrap()
                                            .to_lowercase();
                                        match var.as_str() {
                                            "jp" => "Japanese",
                                            "kr" => "Korean",
                                            "sc" => "SimplifiedChinese",
                                            "tc" => "TraditionalChinese",
                                            "hk" => "TraditionalChineseHK",
                                            _ => panic!("unknown CJK variety `{var}`"),
                                        }
                                    }
                                )],
                            ))
                            .unwrap()
                        } else if x.contains("Emoji") {
                            wrapped_first(fetch(
                                "googlefonts",
                                "noto-emoji",
                                &["fonts/NotoColorEmoji.ttf"],
                            ))
                            .unwrap()
                        } else {
                            panic!("could not find {x}. The err from gh-file-curler is:\n    {e}");
                        }
                    }),
                }
            })
            .collect()
    }

    pub fn map_string(self) -> MapString {
        fn stringify(stuff: &[String]) -> String {
            stuff
                .iter()
                .sorted_by(|a, b| script(a).cmp(&script(b)))
                .group_by(|f| script(f))
                .into_iter()
                .map(|(_, mut g)| g.join(", "))
                .join("\r\n    ")
        }
        let mut all = String::new();
        let mut conflicts = String::new();
        let mut missing = String::new();
        for (c, fonts) in self.map.iter().filter(|m| !m.1.is_empty()) {
            let fonts_str = stringify(fonts);
            all += &format!("{c:04x}\r\n    {fonts_str}\r\n");
            if scripts(fonts).len() > 1 {
                conflicts += &format!("{c:04x}\r\n    {fonts_str}\r\n");
            }
            let bad = missing_variants(fonts);
            if !bad.is_empty() {
                missing += &format!("{c:04x}\r\n    {}\r\n", stringify(&bad));
            }
        }
        MapString {
            all,
            conflicts,
            missing,
        }
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

fn drain_before(f: &[String], index: Option<usize>) -> Vec<String> {
    let mut f = f.to_vec();
    if let Some(i) = index {
        f.drain(..i);
    }
    f
}

#[derive(Clone, Deserialize, Debug)]
struct BlockEndpoints {
    ix: usize,
    start: u32,
    end: u32,
    name: String,
}

#[derive(Clone)]
pub struct NotoizeClient {
    blocks: Vec<BlockEndpoints>,
    font_support: Vec<(u32, Vec<String>)>,
}

impl Default for NotoizeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl NotoizeClient {
    pub fn new() -> Self {
        Self {
            blocks: {
                eprintln!("fetching block list");
                fetch("notofonts", "overview", &["blocks.json"])
                    .unwrap()
                    .write_to(".notoize");
                serde_json::from_str::<Vec<BlockEndpoints>>(
                    &fs::read_to_string(".notoize/blocks.json").unwrap(),
                )
                .unwrap()
            },
            font_support: vec![],
        }
    }

    /// Returns a minimal font stack for rendering `text`
    pub fn notoize(&mut self, text: &str) -> FontStack {
        fs::remove_dir_all(".notoize").unwrap_or(());
        fs::create_dir(".notoize").unwrap_or(());
        let mut fonts = vec![];
        let codepoints = text
            .chars()
            .map(|c| c as u32)
            .sorted()
            .dedup()
            .collect_vec();
        let mut data = BlockData {
            cps: HashMap::new(),
            fonts: None,
        };
        for &c in &codepoints {
            if let Some(i) = self
                .blocks
                .iter()
                .find(|b| b.start <= c && c <= b.end)
                .map(|b| b.ix)
            {
                self.font_support.push({
                    let e = {
                        let path = format!("blocks/block-{i:03}.json");
                        if !Path::new(&format!(".notoize/{path}")).exists()
                            && !self.font_support.iter().any(|f| f.0 == c)
                        {
                            let block = self
                                .blocks
                                .iter()
                                .find(|b| b.start <= c && c <= b.end)
                                .unwrap();
                            eprintln!(
                                "loading support for {:04x}-{:04x} `{}`",
                                block.start, block.end, block.name
                            );
                            fetch("notofonts", "overview", &[&path])
                                .unwrap()
                                .write_to(".notoize");
                            data = serde_json::from_str::<BlockData>(
                                &fs::read_to_string(format!(".notoize/{path}")).unwrap(),
                            )
                            .unwrap();
                        }
                        &data
                    };
                    e.cps
                        .iter()
                        .map(|(k, v)| {
                            (
                                k,
                                match &e.fonts {
                                    None => v.fonts.clone().unwrap_or(vec![]),
                                    Some(f) => f.to_vec(),
                                },
                            )
                        })
                        .map(|(k, v)| {
                            (
                                k.parse::<u32>().unwrap(),
                                v.iter()
                                    .filter(|f| !["UI", "Display"].iter().any(|a| f.contains(a)))
                                    .cloned()
                                    .collect_vec(),
                            )
                        })
                        .find(|(k, _)| *k == c)
                        .unwrap_or((c, vec![]))
                })
            }
        }
        let font_support = &self.font_support;
        for c in codepoints {
            let f = font_support
                .iter()
                .find(|(n, _)| n == &c)
                .cloned()
                .unwrap_or((c, vec![]))
                .1;
            let f = f.iter().map(|e| e.to_string()).collect_vec();
            let f = drain_before(&f, f.iter().position(|x| x == "Sans"));
            if !f.is_empty() {
                let sel = &f[0];
                if !fonts.contains(&format!("Noto {sel}")) {
                    eprintln!("need {sel} for u+{c:04x}");
                    fonts.push(format!("Noto {sel}"));
                }
            }
        }
        fs::remove_dir_all(".notoize").unwrap_or(());
        fs::create_dir(".notoize").unwrap_or(());
        FontStack {
            names: fonts,
            map: font_support.to_vec(),
        }
    }
}

macro_rules! generate_script {
    ($($($font:literal)|* => $script:literal),* $(,)?) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub struct Script(u16);
        pub fn script(font: &str) -> Script {
            let mut n = 0;
            $(
            if let $($font)|* = font {
                return Script(n);
            }
            n += 1;
            )*
            _ = n;
            panic!(
                "unknown font name `{font}` - please file an issue on the github repo or i'll catch \
                 it in up to three months"
            )
        }
        pub fn all_variants() -> Vec<String> {
            let mut a = vec![];
            $(
            a.extend([$($font),*]);
            )*
            a.iter().map(|v| v.to_string()).collect_vec()
        }
    }
}

generate_script! {
    // check via / ((?!Sans|Serif)[a-zA-Z]+)([ ,]|$).*\n.* \1([ ,]|$)/
    "Sans" | "Serif" | "Sans Mono" => "",
    "Sans Adlam" | "Sans Adlam Unjoined" => "Adlam",
    "Serif Ahom" => "Ahom",
    "Sans AnatoHiero" => "Anatolian Hieroglyphs",
    "Sans Arabic" | "Kufi Arabic" | "Naskh Arabic" | "Nastaliq Urdu" => "Arabic",
    "Sans Armenian" | "Serif Armenian" => "Armenian",
    "Sans Avestan" => "Avestan",
    "Sans Bengali" | "Serif Bengali" => "Bengali",
    "Sans Balinese" | "Serif Balinese" => "Balinese",
    "Sans Bamum" => "Bamum",
    "Sans Bassa Vah" => "Bassa Vah",
    "Sans Batak" => "Batak",
    "Sans Bhaiksuki" => "Bhaiksuki",
    "Sans Brahmi" => "Brahmi",
    "Sans Buginese" => "Buginese",
    "Sans Buhid" => "Buhid",
    "Sans Canadian Aboriginal" => "Canadian Aboriginal",
    "Sans Carian" => "Carian",
    "Sans Caucasian Albanian" => "Caucasian Albanian",
    "Sans Chakma" => "Chakma",
    "Sans Cham" => "Cham",
    "Sans Cherokee" => "Cherokee",
    "Sans Chorasmian" => "Chorasmian",
    "Sans CJK HK" | "Sans CJK JP" | "Sans CJK KR" | "Sans CJK SC" | "Sans CJK TC" => "CJK",
    "Sans Coptic" => "Coptic",
    "Sans Cuneiform" => "Cuneiform",
    "Sans Cypriot" => "Cypriot",
    "Sans Cypro Minoan" => "Cypro Minoan",
    "Sans Deseret" => "Deseret",
    "Sans Devanagari" | "Serif Devanagari" => "Devanagari",
    "Serif Dives Akuru" => "Dives Akuru",
    "Serif Dogra" => "Dogra",
    "Sans Duployan" => "Duployan",
    "Sans EgyptHiero" => "Egyptian Hieroglyphs",
    "Color Emoji" => "Emoji",
    "Sans Elbasan" => "Elbasan",
    "Sans Elymaic" => "Elymaic",
    "Sans Ethiopic" | "Serif Ethiopic" => "Ethiopic",
    "Sans Georgian" | "Serif Georgian" => "Georgian",
    "Sans Glagolitic" => "Glagolitic",
    "Sans Gothic" => "Gothic",
    "Sans Grantha" | "Serif Grantha" => "Grantha",
    "Sans Gujarati" | "Serif Gujarati" => "Gujarati",
    "Sans Gunjala Gondi" => "Gunjala Gondi",
    "Sans Gurmukhi" | "Serif Gurmukhi" => "Gurmukhi",
    "Sans Hanifi Rohingya" => "Hanifi Rohingya",
    "Sans Hanunoo" => "Hanunoo",
    "Sans Hatran" => "Hatran",
    "Sans Hebrew" | "Serif Hebrew" | "Rashi Hebrew" => "Hebrew",
    "Sans ImpAramaic" => "Imperial Aramaic",
    "Sans Indic Siyaq Numbers" => "Indic Siyaq Numbers",
    "Sans InsPahlavi" => "Inscriptional Pahlavi",
    "Sans Inscriptional Parthian" => "Inscriptional Parthian",
    "Sans Javanese" => "Javanese",
    "Sans Kaithi" => "Kaithi",
    "Sans Kannada" | "Serif Kannada" => "Kannada",
    "Sans Kawi" => "Kawi",
    "Serif Khitan Small Script" | "Fangsong KSS Rotated" | "Fangsong KSS Vertical" => "Khitan",
    "Sans Kayah Li" => "Kayah Li",
    "Sans Kharoshthi" => "Kharoshthi",
    "Sans Khmer" | "Serif Khmer" => "Khmer",
    "Sans Khojki" | "Serif Khojki" => "Khojki",
    "Sans Khudawadi" => "Khudawadi",
    "Sans Lao" | "Sans Lao Looped" | "Serif Lao" => "Lao",
    "Sans Lepcha" => "Lepcha",
    "Sans Limbu" => "Limbu",
    "Sans Linear A" => "Linear A",
    "Sans Linear B" => "Linear B",
    "Sans Lisu" => "Lisu",
    "Sans Lycian" => "Lycian",
    "Sans Lydian" => "Lydian",
    "Sans Mahajani" => "Mahajani",
    "Sans Malayalam" | "Serif Malayalam" => "Malayalam",
    "Sans Mandaic" => "Mandaic",
    "Serif Makasar" => "Makasar",
    "Sans Manichaean" => "Manichaean",
    "Sans Masaram Gondi" => "Masaram Gondi",
    "Sans Marchen" => "Marchen",
    "Sans Math" => "Math",
    "Sans Mayan Numerals" => "Mayan Numerals",
    "Sans Medefaidrin" => "Medefaidrin",
    "Sans Meetei Mayek" => "Meetei Mayek",
    "Sans Mende Kikakui" => "Mende Kikakui",
    "Sans Meroitic" => "Meroitic",
    "Sans Miao" => "Miao",
    "Sans Modi" => "Modi",
    "Sans Mongolian" => "Mongolian",
    "Sans Mro" => "Mro",
    "Sans Multani" => "Multani",
    "Music" => "Music",
    "Sans Myanmar" | "Serif Myanmar" => "Myanmar",
    "Sans Nabataean" => "Nabataean",
    "Sans Nag Mundari" => "Nag Mundari",
    "Sans Nandinagari" => "Nandinagari",
    "Sans New Tai Lue" => "New Tai Lue",
    "Sans Newa" => "Newa",
    "Sans NKo" | "Sans NKo Unjoined" => "NKo",
    "Serif NP Hmong" => "Nyiakeng Puachue Hmong",
    "Sans Nushu" | "Traditional Nushu" => "Nushu",
    "Sans Ogham" => "Ogham",
    "Sans Ol Chiki" => "Ol Chiki",
    "Sans OldHung" => "Old Hungarian",
    "Sans Old Italic" => "Old Italic",
    "Sans OldNorArab" => "Old North Arabian",
    "Sans Old Permic" => "Old Permic",
    "Sans OldPersian" => "Old Persian",
    "Sans OldSogdian" => "Old Sogdian",
    "Sans OldSouArab" => "Old South Arabian",
    "Sans Old Turkic" => "Old Turkic",
    "Serif Old Uyghur" => "Old Uyghur",
    "Sans Oriya" | "Serif Oriya" => "Oriya",
    "Sans Osage" => "Osage",
    "Sans Osmanya" => "Osmanya",
    "Serif Ottoman Siyaq" => "Ottoman Siyaq",
    "Sans Pahawh Hmong" => "Pahawh Hmong",
    "Sans Palmyrene" => "Palmyrene",
    "Sans PauCinHau" => "Pau Cin Hau",
    "Sans PhagsPa" => "Phags-Pa",
    "Sans Phoenician" => "Phoenician",
    "Sans PsaPahlavi" => "Psalter Pahlavi",
    "Sans Rejang" => "Rejang",
    "Sans Runic" => "Runic",
    "Sans Samaritan" => "Samaritan",
    "Sans Saurashtra" => "Saurashtra",
    "Sans Sharada" => "Sharada",
    "Sans Siddham" => "Siddham",
    "Sans Shavian" => "Shavian",
    "Sans SignWriting" => "SignWriting",
    "Sans Sinhala" | "Serif Sinhala" => "Sinhala",
    "Sans Sogdian" => "Sogdian",
    "Sans Sora Sompeng" => "Sora Sompeng",
    "Sans Soyombo" => "Soyombo",
    "Sans Sundanese" => "Sundanese",
    "Sans Syloti Nagri" => "Syloti Nagri",
    "Sans Symbols" => "Symbols",
    "Sans Symbols 2" => "Symbols 2", // yeah
    "Sans Syriac" | "Sans Syriac Eastern" | "Sans Syriac Western" => "Syriac",
    "Sans Tamil" | "Serif Tamil" => "Tamil",
    "Sans Tamil Supplement" => "Tamil Supplement",
    "Sans Tagalog" => "Tagalog",
    "Sans Tagbanwa" => "Tagbanwa",
    "Sans Tai Le" => "Tai Le",
    "Sans Tai Tham" => "Tai Tham",
    "Sans Tai Viet" => "Tai Viet",
    "Sans Takri" => "Takri",
    "Sans Tangsa" => "Tangsa",
    "Serif Tangut" => "Tangut",
    "Sans Telugu" | "Serif Telugu" => "Telugu",
    "Sans Thaana" => "Thaana",
    "Sans Thai" | "Sans Thai Looped Regular" | "Serif Thai" => "Thai",
    "Sans Tirhuta" => "Tirhuta",
    "Serif Tibetan" => "Tibetan",
    // i have no clue what these variants are
    "Sans Tifinagh"
    | "Sans Tifinagh APT"
    | "Sans Tifinagh Adrar"
    | "Sans Tifinagh Agraw Imazighen"
    | "Sans Tifinagh Ahaggar"
    | "Sans Tifinagh Air"
    | "Sans Tifinagh Azawagh"
    | "Sans Tifinagh Ghat"
    | "Sans Tifinagh Hawad"
    | "Sans Tifinagh Rhissa Ixa"
    | "Sans Tifinagh SIL"
    | "Sans Tifinagh Tawellemmet" => "Tifinagh",
    "Serif Toto" => "Toto",
    "Sans Ugaritic" => "Ugaritic",
    "Sans Vai" => "Vai",
    "Sans Vithkuqi" | "Serif Vithkuqi" => "Vithkuqi",
    "Sans Wancho" => "Wancho",
    "Sans WarangCiti" => "Warang Citi",
    "Serif Yezidi" => "Yezidi",
    "Sans Yi" => "Yi",
    "Sans Zanabazar" => "Zanabazar",
}

pub fn scripts(fonts: &[String]) -> Vec<Script> {
    fonts
        .iter()
        .map(|f| f.as_str())
        .map(script)
        .sorted()
        .dedup()
        .collect_vec()
}

fn missing_variants(font_names: &[String]) -> Vec<String> {
    all_variants()
        .iter()
        .filter(|v| font_names.iter().any(|f| script(f) == script(v)) && !font_names.contains(v))
        .cloned()
        .collect_vec()
}
