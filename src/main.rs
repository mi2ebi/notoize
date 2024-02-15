use notoize::*;
use std::fs;

fn main() {
    let the = notoize("ᵼഔᎇℴ↤ب𝄞").files();
    for f in the {
        fs::write("test/".to_string() + &f.filename, &f.bytes).expect("test doesn't exist");
    }
    // no one will know >:3
    // fs::remove_dir_all(".notoize").unwrap();
}
