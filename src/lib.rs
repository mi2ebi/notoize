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
            .clone()
            .iter()
            .map(|x| {
                let cjkfile = format!(
                    "{}/TTF/SimplifiedChinese/Noto{0}CJKsc-Regular.ttf",
                    x.split_ascii_whitespace().collect::<Vec<_>>()[1]
                );
                let f = if x.contains("CJK") {
                    cjkfile.split('/').last().unwrap().to_string()
                } else if x == "Noto Color Emoji" {
                    "NotoColorEmoji.ttf".to_string()
                } else {
                    format!("{}-Regular.ttf", x.replace(' ', ""))
                };
                eprintln!("fetching {x} ({f})");
                Font {
                    filename: f.clone(),
                    fontname: x.to_string(),
                    bytes: {
                        let path =
                            format!("fonts/{}/hinted/ttf/{f}", f.split('-').collect::<Vec<_>>()[0]);
                        wrapped_first(fetch("notofonts", "notofonts.github.io", vec![&path]))
                    }
                    .unwrap_or_else(|| {
                        if x.contains("CJK") || x.contains("Emoji") {
                         wrapped_first(fetch("notofonts", "noto-cjk", vec![&cjkfile]))
                            .unwrap_or_else(|| {
                                wrapped_first(fetch("googlefonts", "noto-emoji", vec!["fonts"]))
                                    .unwrap_or_default()
                            })
                        } else {
                            panic!("could not find {x}");
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

fn drain_before(f: Vec<String>, index: Option<usize>) -> Vec<String> {
    let mut f = f;
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
                fetch("notofonts", "overview", vec!["blocks.json"])
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
    pub fn notoize(mut self, text: &str) -> FontStack {
        let mut fonts = vec![];
        let text = text.chars().sorted().dedup();
        let codepoints = text.clone().map(|c| c as u32);
        for c in codepoints {
            if let Some(i) = self
                .blocks
                .iter()
                .find(|b| b.start <= c && c <= b.end)
                .map(|b| b.ix)
            {
                self.font_support.extend(
                    [{
                        // if for some reason we already have some of them
                        let path = format!("blocks/block-{i:03}.json");
                        if !Path::new(&format!(".notoize/{path}")).exists() {
                            fetch("notofonts", "overview", vec![&path])
                                .unwrap()
                                .write_to(".notoize");
                        }
                        serde_json::from_str::<BlockData>(
                            &fs::read_to_string(format!(".notoize/{path}")).unwrap(),
                        )
                        .unwrap()
                    }]
                    .iter()
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
                    .collect_vec(),
                )
            }
        }
        for c in text {
            let codepoint = c as u32;
            let f = self
                .font_support
                .iter()
                .find(|(n, _)| n == &codepoint)
                .unwrap_or(&(codepoint, vec![]))
                .1
                .clone();
            let f = drain_before(f.clone(), f.iter().position(|x| x == "Sans"));
            if !f.is_empty() {
                let sel = &f[0];
                if !fonts.contains(&format!("Noto {}", sel)) {
                    eprintln!("need {sel} for u+{codepoint:04x}");
                    fonts.push(format!("Noto {}", sel));
                }
            } else {
                // eprintln!("no fonts support u+{codepoint:04x}")
            }
        }
        FontStack(fonts)
    }
}

impl Drop for NotoizeClient {
    fn drop(&mut self) {
        fs::remove_dir_all(".notoize").unwrap_or(());
    }
}
