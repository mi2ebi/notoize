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
        let mut all = String::new();
        let mut conflicts = String::new();
        for (c, fonts) in self.map.iter().filter(|m| !m.1.is_empty()) {
            let fonts_str = fonts
                .iter()
                .sorted_by(|a, b| script(a).cmp(&script(b)))
                .group_by(|f| script(f))
                .into_iter()
                .map(|(_, mut g)| g.join(", "))
                .join("\r\n    ");
            all += &format!("{c:04x}\r\n    {fonts_str}\r\n");
            if scripts(fonts).len() > 1 {
                conflicts += &format!("{c:04x}\r\n    {fonts_str}\r\n");
            }
        }
        MapString { all, conflicts }
        // todo - check for all variants
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
                self.font_support.push(
                    [{
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
                        // eprint!("{c:04x}\r");
                        &data
                    }]
                    .iter()
                    .flat_map(|e| {
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
                            .collect::<HashMap<_, _>>()
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
                    .find(|(k, v)| {
                        scripts(v);
                        *k == c
                    })
                    .unwrap_or((c, vec![])),
                );
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
            } else {
                // eprintln!("no fonts support u+{codepoint:04x}");
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

#[allow(clippy::ptr_arg)] // &String is deliberate here
pub fn script(name: &String) -> String {
    match name.as_str() {
        // no _ for ease of future modifications
        // check via / ((?!Sans|Serif)[a-zA-Z]+)([ ,]|$).*\n.* \1([ ,]|$)/
        "Sans" | "Serif" | "Sans Mono" => String::new(),
        "Sans Adlam" | "Sans Adlam Unjoined" => "Adlam".to_string(),
        "Serif Ahom" => "Ahom".to_string(),
        "Sans AnatoHiero" => "Anatolian Hieroglyphs".to_string(),
        "Sans Arabic" | "Kufi Arabic" | "Naskh Arabic" | "Nastaliq Urdu" => "Arabic".to_string(),
        "Sans Armenian" | "Serif Armenian" => "Armenian".to_string(),
        "Sans Avestan" => "Avestan".to_string(),
        "Sans Bengali" | "Serif Bengali" => "Bengali".to_string(),
        "Sans Balinese" | "Serif Balinese" => "Balinese".to_string(),
        "Sans Bamum" => "Bamum".to_string(),
        "Sans Bassa Vah" => "Bassa Vah".to_string(),
        "Sans Batak" => "Batak".to_string(),
        "Sans Bhaiksuki" => "Bhaiksuki".to_string(),
        "Sans Brahmi" => "Brahmi".to_string(),
        "Sans Buginese" => "Buginese".to_string(),
        "Sans Buhid" => "Buhid".to_string(),
        "Sans Canadian Aboriginal" => "Canadian Aboriginal".to_string(),
        "Sans Carian" => "Carian".to_string(),
        "Sans Caucasian Albanian" => "Caucasian Albanian".to_string(),
        "Sans Chakma" => "Chakma".to_string(),
        "Sans Cham" => "Cham".to_string(),
        "Sans Cherokee" => "Cherokee".to_string(),
        "Sans Chorasmian" => "Chorasmian".to_string(),
        "Sans CJK HK" | "Sans CJK JP" | "Sans CJK KR" | "Sans CJK SC" | "Sans CJK TC" => {
            "CJK".to_string()
        }
        "Sans Coptic" => "Coptic".to_string(),
        "Sans Cuneiform" => "Cuneiform".to_string(),
        "Sans Cypriot" => "Cypriot".to_string(),
        "Sans Cypro Minoan" => "Cypro Minoan".to_string(),
        "Sans Deseret" => "Deseret".to_string(),
        "Sans Devanagari" | "Serif Devanagari" => "Devanagari".to_string(),
        "Serif Dives Akuru" => "Dives Akuru".to_string(),
        "Serif Dogra" => "Dogra".to_string(),
        "Sans Duployan" => "Duployan".to_string(),
        "Sans EgyptHiero" => "Egyptian Hieroglyphs".to_string(),
        "Color Emoji" => "Emoji".to_string(),
        "Sans Elbasan" => "Elbasan".to_string(),
        "Sans Elymaic" => "Elymaic".to_string(),
        "Sans Ethiopic" | "Serif Ethiopic" => "Ethiopic".to_string(),
        "Sans Georgian" | "Serif Georgian" => "Georgian".to_string(),
        "Sans Glagolitic" => "Glagolitic".to_string(),
        "Sans Gothic" => "Gothic".to_string(),
        "Sans Grantha" | "Serif Grantha" => "Grantha".to_string(),
        "Sans Gujarati" | "Serif Gujarati" => "Gujarati".to_string(),
        "Sans Gunjala Gondi" => "Gunjala Gondi".to_string(),
        "Sans Gurmukhi" | "Serif Gurmukhi" => "Gurmukhi".to_string(),
        "Sans Hanifi Rohingya" => "Hanifi Rohingya".to_string(),
        "Sans Hanunoo" => "Hanunoo".to_string(),
        "Sans Hatran" => "Hatran".to_string(),
        "Sans Hebrew" | "Serif Hebrew" | "Rashi Hebrew" => "Hebrew".to_string(),
        "Sans ImpAramaic" => "Imperial Aramaic".to_string(),
        "Sans Indic Siyaq Numbers" => "Indic Siyaq Numbers".to_string(),
        "Sans InsPahlavi" => "Inscriptional Pahlavi".to_string(),
        "Sans Inscriptional Parthian" => "Inscriptional Parthian".to_string(),
        "Sans Javanese" => "Javanese".to_string(),
        "Sans Kaithi" => "Kaithi".to_string(),
        "Sans Kannada" | "Serif Kannada" => "Kannada".to_string(),
        "Sans Kawi" => "Kawi".to_string(),
        "Serif Khitan Small Script" | "Fangsong KSS Rotated" | "Fangsong KSS Vertical" => {
            "Khitan".to_string()
        }
        "Sans Kayah Li" => "Kayah Li".to_string(),
        "Sans Kharoshthi" => "Kharoshthi".to_string(),
        "Sans Khmer" | "Serif Khmer" => "Khmer".to_string(),
        "Sans Khojki" | "Serif Khojki" => "Khojki".to_string(),
        "Sans Khudawadi" => "Khudawadi".to_string(),
        "Sans Lao" | "Sans Lao Looped" | "Serif Lao" => "Lao".to_string(),
        "Sans Lepcha" => "Lepcha".to_string(),
        "Sans Limbu" => "Limbu".to_string(),
        "Sans Linear A" => "Linear A".to_string(),
        "Sans Linear B" => "Linear B".to_string(),
        "Sans Lisu" => "Lisu".to_string(),
        "Sans Lycian" => "Lycian".to_string(),
        "Sans Lydian" => "Lydian".to_string(),
        "Sans Mahajani" => "Mahajani".to_string(),
        "Sans Malayalam" | "Serif Malayalam" => "Malayalam".to_string(),
        "Sans Mandaic" => "Mandaic".to_string(),
        "Serif Makasar" => "Makasar".to_string(),
        "Sans Manichaean" => "Manichaean".to_string(),
        "Sans Masaram Gondi" => "Masaram Gondi".to_string(),
        "Sans Marchen" => "Marchen".to_string(),
        "Sans Math" => "Math".to_string(),
        "Sans Mayan Numerals" => "Mayan Numerals".to_string(),
        "Sans Medefaidrin" => "Medefaidrin".to_string(),
        "Sans Meetei Mayek" => "Meetei Mayek".to_string(),
        "Sans Mende Kikakui" => "Mende Kikakui".to_string(),
        "Sans Meroitic" => "Meroitic".to_string(),
        "Sans Miao" => "Miao".to_string(),
        "Sans Modi" => "Modi".to_string(),
        "Sans Mongolian" => "Mongolian".to_string(),
        "Sans Mro" => "Mro".to_string(),
        "Sans Multani" => "Multani".to_string(),
        "Music" => "Music".to_string(),
        "Sans Myanmar" | "Serif Myanmar" => "Myanmar".to_string(),
        "Sans Nabataean" => "Nabataean".to_string(),
        "Sans Nag Mundari" => "Nag Mundari".to_string(),
        "Sans Nandinagari" => "Nandinagari".to_string(),
        "Sans New Tai Lue" => "New Tai Lue".to_string(),
        "Sans Newa" => "Newa".to_string(),
        "Sans NKo" | "Sans NKo Unjoined" => "NKo".to_string(),
        "Serif NP Hmong" => "Nyiakeng Puachue Hmong".to_string(),
        "Sans Nushu" | "Traditional Nushu" => "Nushu".to_string(),
        "Sans Ogham" => "Ogham".to_string(),
        "Sans Ol Chiki" => "Ol Chiki".to_string(),
        "Sans OldHung" => "Old Hungarian".to_string(),
        "Sans Old Italic" => "Old Italic".to_string(),
        "Sans OldNorArab" => "Old North Arabian".to_string(),
        "Sans Old Permic" => "Old Permic".to_string(),
        "Sans OldPersian" => "Old Persian".to_string(),
        "Sans OldSogdian" => "Old Sogdian".to_string(),
        "Sans OldSouArab" => "Old South Arabian".to_string(),
        "Sans Old Turkic" => "Old Turkic".to_string(),
        "Serif Old Uyghur" => "Old Uyghur".to_string(),
        "Sans Oriya" | "Serif Oriya" => "Oriya".to_string(),
        "Sans Osage" => "Osage".to_string(),
        "Sans Osmanya" => "Osmanya".to_string(),
        "Serif Ottoman Siyaq" => "Ottoman Siyaq".to_string(),
        "Sans Pahawh Hmong" => "Pahawh Hmong".to_string(),
        "Sans Palmyrene" => "Palmyrene".to_string(),
        "Sans PauCinHau" => "Pau Cin Hau".to_string(),
        "Sans PhagsPa" => "Phags-Pa".to_string(),
        "Sans Phoenician" => "Phoenician".to_string(),
        "Sans PsaPahlavi" => "Psalter Pahlavi".to_string(),
        "Sans Rejang" => "Rejang".to_string(),
        "Sans Runic" => "Runic".to_string(),
        "Sans Samaritan" => "Samaritan".to_string(),
        "Sans Saurashtra" => "Saurashtra".to_string(),
        "Sans Sharada" => "Sharada".to_string(),
        "Sans Siddham" => "Siddham".to_string(),
        "Sans Shavian" => "Shavian".to_string(),
        "Sans SignWriting" => "SignWriting".to_string(),
        "Sans Sinhala" | "Serif Sinhala" => "Sinhala".to_string(),
        "Sans Sogdian" => "Sogdian".to_string(),
        "Sans Sora Sompeng" => "Sora Sompeng".to_string(),
        "Sans Soyombo" => "Soyombo".to_string(),
        "Sans Sundanese" => "Sundanese".to_string(),
        "Sans Syloti Nagri" => "Syloti Nagri".to_string(),
        "Sans Symbols" | "Sans Symbols 2" => "Symbols".to_string(),
        "Sans Syriac" | "Sans Syriac Eastern" | "Sans Syriac Western" => "Syriac".to_string(),
        "Sans Tamil" | "Serif Tamil" => "Tamil".to_string(),
        "Sans Tamil Supplement" => "Tamil Supplement".to_string(),
        "Sans Tagalog" => "Tagalog".to_string(),
        "Sans Tagbanwa" => "Tagbanwa".to_string(),
        "Sans Tai Le" => "Tai Le".to_string(),
        "Sans Tai Tham" => "Tai Tham".to_string(),
        "Sans Tai Viet" => "Tai Viet".to_string(),
        "Sans Takri" => "Takri".to_string(),
        "Sans Tangsa" => "Tangsa".to_string(),
        "Serif Tangut" => "Tangut".to_string(),
        "Sans Telugu" | "Serif Telugu" => "Telugu".to_string(),
        "Sans Thaana" => "Thaana".to_string(),
        "Sans Thai" | "Sans Thai Looped Regular" | "Serif Thai" => "Thai".to_string(),
        "Sans Tirhuta" => "Tirhuta".to_string(),
        "Serif Tibetan" => "Tibetan".to_string(),
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
        | "Sans Tifinagh Tawellemmet" => "Tifinagh".to_string(),
        "Serif Toto" => "Toto".to_string(),
        "Sans Ugaritic" => "Ugaritic".to_string(),
        "Sans Vai" => "Vai".to_string(),
        "Sans Vithkuqi" | "Serif Vithkuqi" => "Vithkuqi".to_string(),
        "Sans Wancho" => "Wancho".to_string(),
        "Sans WarangCiti" => "Warang Citi".to_string(),
        "Serif Yezidi" => "Yezidi".to_string(),
        "Sans Yi" => "Yi".to_string(),
        "Sans Zanabazar" => "Zanabazar".to_string(),
        _ => panic!(
            "unknown font name `{name}` - please file an issue on the github repo or i'll catch \
             it in up to three months"
        ),
    }
}

pub fn scripts(fonts: &[String]) -> Vec<String> {
    fonts.iter().map(script).sorted().dedup().collect_vec()
}
