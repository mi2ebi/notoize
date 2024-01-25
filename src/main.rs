use notoize::*;

fn main() {
    let config = NotoizeConfig {
        // prefer_math: true,
        adlam: vec![AdlamNkoCfg::Unjoined],
        ..NotoizeConfig::new_sans()
    };
    println!("{:?}", notoize("ᵼഔᎇ䅺ℴ↤", config));
}