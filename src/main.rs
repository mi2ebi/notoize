use notoize::{config::*, *};
use std::{fs, time::Instant};

fn main() {
    let start = Instant::now();
    let client = NotoizeClient::default().lgc(vec![Lgc::Serif]);
    let the = client.notoize("ᵼഔᎇℴ↤ب𝄞").files();
    for f in the {
        fs::write("test/".to_string() + &f.filename, &f.bytes).expect("test doesn't exist");
    }
    println!("{:?}", start.elapsed());
}
