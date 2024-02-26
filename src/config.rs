use crate::NotoizeClient;

#[derive(Clone)]
pub struct NotoizeConfig {
    pub font_ext: Vec<FontExt>,
    pub lgc: Vec<Lgc>,
    pub adlam: Vec<Adlam>,
    pub arabic: Vec<Arabic>,
}

impl NotoizeConfig {
    pub fn new_sans(ext: Vec<FontExt>) -> Self {
        Self {
            font_ext: ext,
            lgc: vec![Lgc::Sans],
            adlam: vec![Adlam::Unjoined],
            arabic: vec![Arabic::Sans],
        }
    }
    pub fn to_string_vec(&self) -> Vec<String> {
        let mut v: Vec<String> = vec![];
        v.extend(self.lgc.iter().map(|x| x.to_string()));
        v.extend(self.adlam.iter().map(|x| x.to_string()));
        v
    }
}
impl NotoizeClient {
    pub fn lgc(self, lgc: Vec<Lgc>) -> Self {
        let clone = self.clone();
        drop(self);
        Self {
            config: NotoizeConfig {
                lgc,
                ..clone.config
            },
            ..clone
        }
    }
    // bleh gotta fix the rest too
    pub fn adlam(self, adlam: Vec<Adlam>) -> Self {
        Self {
            config: NotoizeConfig {
                adlam,
                ..self.config.clone()
            },
            ..self.clone()
        }
    }
    pub fn arabic(self, arabic: Vec<Arabic>) -> Self {
        Self {
            config: NotoizeConfig {
                arabic,
                ..self.config.clone()
            },
            ..self.clone()
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
impl ToString for Lgc {
    fn to_string(&self) -> String {
        match self {
            Self::Sans => "lgc_sans",
            Self::Serif => "lgc_serif",
            Self::Mono => "lgc_mono",
        }
        .to_string()
    }
}

#[derive(Clone, PartialEq)]
pub enum Adlam {
    Unjoined,
    Joined,
}
impl ToString for Adlam {
    fn to_string(&self) -> String {
        match self {
            Self::Unjoined => "adlam_unjoined",
            Self::Joined => "adlam_joined",
        }
        .to_string()
    }
}

#[derive(Clone, PartialEq)]
pub enum Arabic {
    Sans,
    Kufi,
    Naskh,
    Nastaliq,
}
