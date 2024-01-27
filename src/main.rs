use notoize::{Serifness::*, *};

fn main() {
    let config = NotoizeConfig {
        lgc: vec![Serif, Sans],
        ..NotoizeConfig::new_sans()
    };
    println!("{:?}", notoize("ᵼഔᎇ䅺ℴ↤ب", config));
}
