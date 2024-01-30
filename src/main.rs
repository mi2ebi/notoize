use std::fs;

use notoize::*;

fn main() {
    let the = notoize("ᵼഔᎇℴ↤ب𝄞").files();
    for f in the {
        fs::write("test/".to_string() + &f.filename, &f.bytes).expect("test doesn't exist")
    };
}
