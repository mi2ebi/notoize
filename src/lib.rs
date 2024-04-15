use gh_file_curler::{fetch, wrapped_first};
use itertools::Itertools;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug)]
pub struct FontStack(pub Vec<String>);

#[derive(Debug, Clone)]
pub struct Font {
    pub filename: String,
    pub fontname: String,
    pub bytes: Vec<u8>,
}

impl FontStack {
    pub fn files(&self) -> Vec<Font> {
        self.0
            .iter()
            .map(|x| {
                let f = if x.contains("CJK") {
                    format!(
                        "Noto{}CJK{}-Regular.otf",
                        x.split_ascii_whitespace().collect::<Vec<_>>()[1],
                        x.split_ascii_whitespace().collect::<Vec<_>>()[3].to_lowercase()
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
                        _ => "the universe broke, sorry",
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
                        let path = format!(
                            "fonts/{}/hinted/ttf/{f}",
                            f.split('-').collect::<Vec<_>>()[0]
                        );
                        wrapped_first(fetch("notofonts", "notofonts.github.io", &[&path]))
                    }
                    .unwrap_or_else(|e| {
                        if x.contains("CJK") {
                            wrapped_first(fetch(
                                "notofonts",
                                "noto-cjk",
                                &[&format!(
                                    "{}/OTF/{}/{f}",
                                    x.split_ascii_whitespace().collect::<Vec<_>>()[1],
                                    {
                                        let var = x.split_ascii_whitespace().collect::<Vec<_>>()[3]
                                            .to_lowercase();
                                        match var.as_str() {
                                            "jp" => "Japanese",
                                            "kr" => "Korean",
                                            "sc" => "SimplifiedChinese",
                                            "tc" => "TraditionalChinese",
                                            "hk" => "TraditionalChineseHK",
                                            _ => panic!("unknown CJK variety \"{var}\""),
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
        fs::create_dir_all(".notoize").unwrap_or_default();
        let mut fonts = vec![];
        let text = text.chars().sorted().dedup();
        let codepoints = text.clone().map(|c| c as u32);
        let mut data = BlockData {
            cps: HashMap::new(),
            fonts: None,
        };
        let mut map = vec![];
        for c in codepoints {
            if let Some(i) = self
                .blocks
                .iter()
                .find(|b| b.start <= c && c <= b.end)
                .map(|b| b.ix)
            {
                self.font_support.push(
                    [{
                        let path = format!("blocks/block-{i:03}.json");
                        if !Path::new(&format!(".notoize/{path}")).exists() {
                            fs::remove_dir_all(".notoize").unwrap_or_default();
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
                    .map(|(k, v)| (k.parse::<u32>().unwrap(), v))
                    .find(|(k, _)| *k == c)
                    .unwrap_or((c, vec![])),
                );
            }
        }
        for c in text {
            let codepoint = c as u32;
            let missing = &(codepoint, vec![]);
            let f = &self
                .font_support
                .iter()
                .find(|(n, _)| n == &codepoint)
                .unwrap_or(missing)
                .1;
            let f = drain_before(f, f.iter().position(|x| x == "Sans"));
            if !f.is_empty() {
                let sel = &f[0];
                if !fonts.contains(&format!("Noto {}", sel)) {
                    eprintln!("need {sel} for u+{codepoint:04x}");
                    fonts.push(format!("Noto {}", sel));
                }
                map.push((codepoint, f.clone()));
            } else {
                // eprintln!("no fonts support u+{codepoint:04x}");
            }
        }
        let mut mapstring = String::new();
        for (c, fonts) in map {
            let fonts_str = fonts.join("\r\n    ");
            mapstring += &format!("{c:04x}\r\n    {}\r\n", fonts_str);
        }
        fs::write(".notoize/mapping.txt", mapstring).unwrap();
        fs::remove_dir_all(".notoize/blocks").unwrap();
        FontStack(fonts)
    }
}
