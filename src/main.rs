use notoize::*;

fn main() {
    let _config = NotoizeConfig {
        // prefer_math: true,
        adlam: vec![AdlamNkoCfg::Unjoined],
        ..NotoizeConfig::new_sans()
    };
    println!("{:?}", FontStack(vec![]).files().iter().map(|x| x.filename.clone()).collect::<Vec<String>>());
}