use std::{env, io::Write};

const SOURCE: &str = r#"
use crate::graphic::font::Font;
const FONT_RAW: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/font_data"));

/// get `Font` corresponding to `c`.
/// if `c` is non ascii char, rerutn `None`.
pub fn get_font(c: char) -> Option<Font> {
    let index = 16 * c as usize;
    if index + 16 > FONT_RAW.len() {
        None
    } else {
        FONT_RAW[index..(index + 16)].try_into().ok()
    }
}
"#;

fn main() {
    let font_data_path = "data/hankaku.txt";
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR is empty");
    let out_dir_path = std::path::Path::new(&out_dir);

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={}", font_data_path);

    let font_data_raw = std::fs::read_to_string(font_data_path).unwrap();

    write_font_data(font_data_raw.as_str(), out_dir_path);

    let p = out_dir_path.join("font_data.rs");
    let mut rs = std::fs::File::create(p).unwrap();
    rs.write_all(SOURCE.as_bytes()).unwrap();
}

fn write_font_data(font_data_raw: &str, out_dir: &std::path::Path) {
    let font_data_path = out_dir.join("font_data");
    let mut font_data_file = std::fs::File::create(font_data_path).unwrap();

    let mut bytes = Vec::new();
    for raw in font_data_raw.lines() {
        if is_data_raw(raw) {
            dbg!(raw);
            let byte: u8 = raw
                .chars()
                .map(|c| if c == '.' { 0 } else { 1 })
                .reduce(|acc, x| 2 * acc + x)
                .unwrap();
            bytes.push(byte);
        }
    }

    font_data_file.write_all(&bytes).unwrap();
}

fn is_data_raw(data_raw: &str) -> bool {
    if data_raw.is_empty() {
        return false;
    }
    data_raw.chars().all(|c| c == '.' || c == '@')
}
