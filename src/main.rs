use std::time::Instant;

use notoize::*;

fn main() {
    let start = Instant::now();
    let config = NotoizeConfig {
        // prefer_math: true,
        adlam: vec![AdlamNkoCfg::Unjoined],
        ..NotoizeConfig::new_sans()
    };
    println!("{:?}", notoize("ᵼഔᎇ䅺ℴ↤", config));
    println!("{}", (Instant::now() - start).as_secs_f64());
}
