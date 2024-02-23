use gh_file_curler::{fetch, wrapped_first};
use itertools::Itertools;
use serde::Deserialize;
use std::{collections::HashMap, fs};
pub mod config;
use config::*;

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
            .clone()
            .iter()
            .map(|x| {
                let cjkfile = format!(
                    "{}/OTF/SimplifiedChinese/Noto{0}CJKsc-Regular.otf",
                    x.split_ascii_whitespace().collect::<Vec<_>>()[1]
                );
                let f = if x.contains("CJK") {
                    cjkfile.split('/').last().unwrap().to_string()
                } else if x == "Noto Color Emoji" {
                    "NotoColorEmoji.ttf".to_string()
                } else {
                    format!("{}-Regular.otf", x.replace(' ', ""))
                };
                eprintln!("fetching {x} ({f})");
                Font {
                    filename: f.clone(),
                    fontname: x.to_string(),
                    bytes: {
                        let path =
                            format!("fonts/{}/full/otf/{f}", f.split('-').collect::<Vec<_>>()[0]);
                        wrapped_first(fetch("notofonts", "notofonts.github.io", vec![&path]))
                    }
                    .unwrap_or_else(|| {
                        wrapped_first(fetch("notofonts", "noto-cjk", vec![&cjkfile]))
                            .unwrap_or_else(|| {
                                wrapped_first(fetch("googlefonts", "noto-emoji", vec!["fonts"]))
                                    .unwrap_or_default()
                            })
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

fn drain_before(f: Vec<String>, index: Option<usize>) -> Vec<String> {
    let mut f = f;
    if let Some(i) = index {
        f.drain(..i);
    }
    f
}

#[derive(Clone)]
pub struct NotoizeClient {
    font_support: Vec<(u32, Vec<String>)>,
    config: NotoizeConfig,
}

impl Default for NotoizeClient {
    fn default() -> Self {
        Self::new(NotoizeConfig::new_sans(vec![FontExt::Ttf]))
    }
}

impl NotoizeClient {
    pub fn new(config: NotoizeConfig) -> Self {
        Self {
            font_support: (0..=323)
                .map(|i| {
                    fetch(
                        "notofonts",
                        "overview",
                        vec![&format!("blocks/block-{i:03}.json")],
                    )
                    .unwrap()
                    .write_to(".notoize");
                    serde_json::from_str::<BlockData>(
                        &fs::read_to_string(format!(".notoize/blocks/block-{i:03}.json")).unwrap(),
                    )
                    .unwrap()
                })
                .flat_map(|e| {
                    e.cps
                        .iter()
                        .map(|(k, v)| {
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
                .collect_vec(),
            config,
        }
    }

    /// Returns a minimal font stack for rendering `text`
    pub fn notoize(self, text: &str) -> FontStack {
        let mut out = vec![];
        let text = text.chars().sorted().dedup();
        let mut hold = vec![];
        for c in text {
            let codepoint = c as u32;
            let fonts = self
                .font_support
                .iter()
                .find(|(n, _)| n == &codepoint)
                .unwrap_or(&(codepoint, vec![]))
                .1
                .clone();
            // let f = drain_before(f.clone(), f.iter().position(|x| x == "Sans"));
            if !fonts.is_empty() {
                let mut sel = vec![];
                // is this char supported by Sans/Serif?
                if fonts
                    .iter()
                    .any(|x| ["Sans", "Sans Mono", "Serif"].contains(&x.as_str()))
                {
                    // anything else?
                    if !fonts
                        .iter()
                        .filter(|x| {
                            !["Sans", "Sans Mono", "Serif", "Serif Display"].contains(&x.as_str())
                        })
                        .collect::<Vec<_>>()
                        .is_empty()
                    {
                        hold.push(c);
                        continue;
                    }
                    // no
                    // add the   cf config
                    let mut s = vec![];
                    for option in self.config {
                        
                    }
                    sel.extend(s);
                } else {
                    // pick variant
                    for font in fonts {
                        let mut s = vec![2];
                        // get the script, fallback from config

                        sel.extend(s);
                    }
                }
                if !sel.is_empty() {
                    for s in sel {
                        eprintln!("need {s} for u+{codepoint:04x}");
                        if !out.contains(&format!("Noto {s}")) {
                            out.push(format!("Noto {s}"));
                        }
                    }
                }
            } else {
                // eprintln!("no fonts support u+{codepoint:04x}")
            }
        }
        FontStack(out)
    }
    pub fn bury_the_evidence(&mut self) {
        // fs::remove_dir_all(".notoize").unwrap_or(());
    }
}
