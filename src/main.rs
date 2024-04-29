use notoize::*;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let mut client = NotoizeClient::new();
    let _ = client
        .notoize(&(0..0x110000).filter_map(char::from_u32).collect::<String>())
        .files();
    println!("{:?}", start.elapsed());
}
