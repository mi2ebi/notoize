use notoize::*;
use std::{fs, sync::LazyLock, time::Instant};

fn main() {
    let start = Instant::now();
    let mut client = NotoizeClient::new();
    static ALL: LazyLock<String> =
        LazyLock::new(|| (0..0x110000).filter_map(char::from_u32).collect::<String>());
    let the = client.notoize(&ALL);
    let map = the.map_string();
    let _ = fs::remove_dir_all("out/data");
    fs::create_dir_all("out/data").unwrap();
    fs::write("out/data/mapping.txt", map.all).unwrap();
    fs::write("out/data/script_conflicts.txt", map.conflicts).unwrap();
    fs::write("out/data/missing_variants.txt", map.missing).unwrap();
    let _ = fs::remove_dir_all("out/fonts");
    fs::create_dir_all("out/fonts").unwrap();
    for font in the.files() {
        fs::write(format!("out/fonts/{}", font.filename), font.bytes).unwrap();
    }
    println!("\x1b[92m{:?}\x1b[m", start.elapsed());
}
