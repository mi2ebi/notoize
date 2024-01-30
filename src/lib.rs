use itertools::Itertools;
use serde::Deserialize;
use std::{collections::HashMap, fs};

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
                let f = format!("{}-Regular.otf", x.replace(' ', ""));
                println!("{}", &f);
                Font {
                    filename: f.clone(),
                    fontname: x.to_string(),
                    bytes: fs::read(format!(
                        "notofonts.github.io/fonts/{}/full/otf/{f}",
                        f.split('-').collect::<Vec<_>>()[0]
                    ))
                    .unwrap(),
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

/// Returns a minimal font stack for rendering `text`
pub fn notoize(text: &str) -> FontStack {
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
    let mut fonts = vec![];
    for c in text.chars() {
        let codepoint = c as u32;
        let hex = format!("{codepoint:04x}");
        let f = font_support
            .iter()
            .find(|(n, _)| n == &codepoint)
            .unwrap_or(&(codepoint, vec![]))
            .1
            .clone();
        if !fonts.contains(&format!("Noto {}", f[0])) {
            fonts.push(format!("Noto {}", f[0]));
        }
        println!("{hex} {f:?}");
    }
    FontStack(fonts)
}
