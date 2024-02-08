use itertools::Itertools;
use serde::Deserialize;
use std::{collections::HashMap, fs, process::Command};

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
                    "noto-cjk/{}/OTF/SimplifiedChinese/Noto{0}CJKsc-Regular.otf",
                    x.split_ascii_whitespace().collect::<Vec<_>>()[1]
                );
                let f = if x.contains("CJK") {
                    cjkfile.split('/').last().unwrap().to_string()
                } else if x == "Noto Color Emoji" {
                    "NotoColorEmoji.ttf".to_string()
                } else {
                    format!("{}-Regular.otf", x.replace(' ', ""))
                };
                Font {
                    filename: f.clone(),
                    fontname: x.to_string(),
                    bytes: fs::read(format!(
                        "notofonts.github.io/fonts/{}/full/otf/{f}",
                        f.split('-').collect::<Vec<_>>()[0]
                    ))
                    .unwrap_or(
                        fs::read(cjkfile)
                            .unwrap_or(fs::read("noto-emoji/fonts/NotoColorEmoji.ttf").unwrap()),
                    ),
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

fn drain_before(mut f: Vec<String>, index: Option<usize>) {
    if let Some(i) = index {
        f.drain(..i);
    }
}

fn clone_folders(origin: &str, dest: &str, folders: Vec<&str>) {
    // cursed
    Command::new("git")
        .args([
            "clone",
            "--depth=1",
            "--single-branch",
            "--filter=blob:none",
            "--no-checkout",
            "--sparse",
            origin,
        ])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(dest)
        .args(["sparse-checkout", "init", "--no-cone"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(dest)
        .args(["sparse-checkout", "set"])
        .args(&folders)
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(dest)
        .arg("checkout")
        .status()
        .unwrap();
}

/// Returns a minimal font stack for rendering `text`
pub fn notoize(text: &str) -> FontStack {
    // grab the repos
    clone_folders(
        "https://github.com/notofonts/overview",
        "overview",
        vec!["blocks"],
    );
    clone_folders(
        "https://github.com/notofonts/notofonts.github.io",
        "notofonts.github.io",
        vec!["fonts"],
    );
    clone_folders(
        "https://github.com/googlefonts/noto-emoji",
        "noto-emoji",
        vec!["fonts"],
    );
    clone_folders(
        "https://github.com/notofonts/noto-cjk",
        "noto-cjk",
        vec!["Sans", "Serif"],
    );
    // parse data
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
        println!("{c} {f:?}");
        drain_before(f.clone(), f.iter().position(|x| x == "Sans"));
        let sel = &f[0];
        if !fonts.contains(&format!("Noto {}", sel)) {
            fonts.push(format!("Noto {}", sel));
        }
    }
    FontStack(fonts)
}
