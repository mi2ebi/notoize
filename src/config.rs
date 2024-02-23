use crate::NotoizeClient;

#[derive(Clone)]
pub struct NotoizeConfig {
    pub font_ext: Vec<FontExt>,
    pub lgc: Vec<Lgc>,
    pub join_adlam: Vec<bool>,
    pub arabic: Vec<Arabic>,
}

impl NotoizeConfig {
    pub fn new_sans(ext: Vec<FontExt>) -> Self {
        Self {
            font_ext: ext,
            lgc: vec![Lgc::Sans],
            join_adlam: vec![false],
            arabic: vec![Arabic::Sans],
        }
    }
}
impl NotoizeClient {
    pub fn lgc(self, lgc: Vec<Lgc>) -> Self {
        let clone = self.clone();
        Self {
            config: NotoizeConfig {
                lgc,
                ..clone.config
            },
            ..clone
        }
    }
    pub fn join_adlam(self, join_adlam: Vec<bool>) -> Self {
        let clone = self.clone();
        Self {
            config: NotoizeConfig {
                join_adlam,
                ..clone.config
            },
            ..clone
        }
    }
    pub fn arabic(self, arabic: Vec<Arabic>) -> Self {
        let clone = self.clone();
        Self {
            config: NotoizeConfig {
                arabic,
                ..clone.config
            },
            ..clone
        }
    }
}

#[derive(Clone)]
pub enum FontExt {
    Ttf,
    Otf,
}

#[derive(Clone, PartialEq)]
pub enum Lgc {
    Sans,
    Serif,
    Mono,
}

#[derive(Clone, PartialEq)]
pub enum Arabic {
    Sans,
    Kufi,
    Naskh,
    Nastaliq,
}
