use notoize::*;

fn main() {
    let config = NotoizeConfig::prefer_serif();
    println!("{:?}", notoize("ᵼഔᎇ䅺ℴ↤", config));
}
