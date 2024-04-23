use notoize::*;
use std::{fs, time::Instant};

fn main() {
    let start = Instant::now();
    let mut client = NotoizeClient::new();
    let the = client
        .notoize(&(0..0x110000).filter_map(char::from_u32).collect::<String>())
        .files();
    fs::remove_dir_all("test").unwrap_or_default();
    fs::create_dir("test").unwrap();
    for f in the {
        fs::write(format!("test/{}", f.filename), &f.bytes).unwrap();
    }
    println!("{:?}", start.elapsed());
}
