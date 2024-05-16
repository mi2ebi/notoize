use notoize::*;
use std::{fs, time::Instant};

fn main() {
    let start = Instant::now();
    let mut client = NotoizeClient::new();
    let the = client
        .notoize(&(0..0x110000).filter_map(char::from_u32).collect::<String>())
        .map_string();
    fs::write(".notoize/mapping.txt", the.all).unwrap();
    fs::write(".notoize/script_conflicts.txt", the.conflicts).unwrap();
    println!("{:?}", start.elapsed());
}
