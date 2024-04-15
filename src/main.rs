use notoize::*;
use std::{fs, time::Instant};

fn main() {
    let start = Instant::now();
    fs::remove_dir_all("test").unwrap_or_default();
    fs::remove_dir_all(".notoize").unwrap_or_default();
    let mut client = NotoizeClient::new();
    let the = client
        .notoize(
            &(0..0x110000)
                .map(|i| char::from_u32(i).unwrap_or_default())
                .collect::<String>(),
        )
        .files();
    fs::create_dir("test").unwrap();
    for f in the {
        fs::write("test/".to_string() + &f.filename, &f.bytes).unwrap();
    }
    let _ = client.notoize("e");
    println!("{:?}", start.elapsed());
}
