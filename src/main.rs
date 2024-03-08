use notoize::*;
use std::{fs, time::Instant};

fn main() {
    let start = Instant::now();
    let client = NotoizeClient::new();
    let the = client.notoize("ᵼaഔᎇℴ↤ب𝄞\u{e800}𛰵⻤🥺").files();
    for f in the {
        fs::write("test/".to_string() + &f.filename, &f.bytes).expect("test doesn't exist");
    }
    println!("{:?}", start.elapsed());
}
