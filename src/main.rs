use std::fs;

use notoize::*;

fn main() {
    let the = notoize("ᵼഔᎇℴ↤ب𝄞").files().iter().map(|f| {
        fs::write("test/".to_string() + &f.filename, &f.bytes).unwrap()
    });
}
