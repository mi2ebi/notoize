use notoize::*;
use std::{fs, time::Instant};

fn main() {
    let start = Instant::now();
    let mut client = NotoizeClient::new();
    let the = client.notoize(&(0..0x110000).filter_map(char::from_u32).collect::<String>());
    let map = the.clone().map_string();
    fs::write("out/data/mapping.txt", map.all).unwrap();
    fs::write("out/data/script_conflicts.txt", map.conflicts).unwrap();
    fs::write("out/data/missing_variants.txt", map.missing).unwrap();
    for font in the.files() {
        fs::write(format!("out/fonts/{}", font.filename), font.bytes).unwrap();
    }
    println!("{:?}", start.elapsed());
}
