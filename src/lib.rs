use dotenv::dotenv;
use gh_file_curler::speedrun;
use itertools::Itertools;
use serde::Deserialize;
use std::{collections::HashMap, env, fs, path::Path};

#[derive(Debug)]
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
            .map(|x| {
                let cjkfile = format!(
                    ".notoize/noto-cjk/{}/OTF/SimplifiedChinese/Noto{0}CJKsc-Regular.otf",
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
                            format!("fonts/{}/full/otf", f.split('-').collect::<Vec<_>>()[0]);
                        if Path::new(".notoize/noto-cjk").exists() || clone_folders("notofonts", "notofonts.github.io", vec![&path]).is_ok() {
                            fs::read(format!(".notoize/notofonts.github.io/{path}/{f}"))
                        } else {
                            fs::read(".")
                        }
                    }
                    .unwrap_or_else(|_| {
                        {
                            if !Path::new(".notoize/noto-cjk").exists() {
                                clone_folders("notofonts", "noto-cjk", vec!["Sans", "Serif"])
                                    .unwrap();
                            }
                            fs::read(cjkfile)
                        }
                        .unwrap_or_else(|_| {
                            if !Path::new(".notoize/noto-emoji").exists() {
                                clone_folders("googlefonts", "noto-emoji", vec!["fonts"]).unwrap();
                            }
                            fs::read(".notoize/noto-emoji/fonts/NotoColorEmoji.ttf")
                                .unwrap_or_else(|_| vec![])
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

fn clone_folders(
    author: &str,
    repo: &str,
    folders: Vec<&str>,
) -> Result<gh_file_curler::Files, String> {
    let gh_token = env::var("GITHUB_API_TOKEN").unwrap();
    let gh_token = &gh_token;
    speedrun(
        author,
        repo,
        &format!(".notoize/{repo}"),
        folders,
        true,
        gh_token,
    )
}
/// Returns a minimal font stack for rendering `text`
pub fn notoize(text: &str) -> FontStack {
    dotenv().ok();
    clone_folders("notofonts", "overview", vec!["blocks"]).unwrap();
    // parse data
    let font_support = (0..=323)
        .map(|i| {
            serde_json::from_str::<BlockData>(
                &fs::read_to_string(format!(".notoize/overview/blocks/block-{i:03}.json")).unwrap(),
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
    // actually do things
    let mut fonts = vec![];
    for c in text.chars() {
        let codepoint = c as u32;
        let f = font_support
            .iter()
            .find(|(n, _)| n == &codepoint)
            .unwrap_or(&(codepoint, vec![]))
            .1
            .clone();
        let f = drain_before(f.clone(), f.iter().position(|x| x == "Sans"));
        if f.len() > 0 {
            let sel = &f[0];
            if !fonts.contains(&format!("Noto {}", sel)) {
                eprintln!("need {sel} for u+{codepoint:04x}");
                fonts.push(format!("Noto {}", sel));
            }
        } else {
            eprintln!("no fonts support u+{codepoint:04x}")
        }
    }
    FontStack(fonts)
}
