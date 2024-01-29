use notoize::{Serifness::*, *};

fn main() {
    let config = NotoizeConfig {
        prefer_ui: true,
        lgc: vec![Serif, Sans],
        arabic: vec![ArabicCfg::Naskh],
        ..NotoizeConfig::new_sans()
    };
    println!("{:?}", notoize("ᵼഔᎇ䅺ℴ↤ب", config));
}
