use std::fs::{self};

use makemkv_core::read_disc_properties;

fn main() {
    let info = read_disc_properties("./data/makemkvcon_movie");
    // let info = read_disc_properties("/Applications/MakeMKV.app/Contents/MacOS/makemkvcon");

    let test = serde_json::to_string_pretty(&info.unwrap()).unwrap();
    fs::write("parsed.json", test).expect("written file");

    // let start = std::time::Instant::now();
    // let info = read_disc_properties("/Applications/MakeMKV.app/Contents/MacOS/makemkvcon");
    // println!("elapsed: {:?} for {:#?}", start.elapsed(), info.unwrap());
    // println!("elapsed: {:?}", start.elapsed());
}
