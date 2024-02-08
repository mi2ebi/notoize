use std::fs;

use notoize::*;

fn main() {
    let the = notoize("ᵼഔᎇℴ↤ب𝄞").files();
    for f in the {
        fs::write("test/".to_string() + &f.filename, &f.bytes).expect("test doesn't exist")
    }
    // no one will know >:3
    fs::remove_dir_all("overview").unwrap();
    fs::remove_dir_all("notofonts.github.io").unwrap();
    fs::remove_dir_all("noto-emoji").unwrap();
    fs::remove_dir_all("noto-cjk").unwrap();
}
