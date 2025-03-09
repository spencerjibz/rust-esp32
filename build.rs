use std::{collections::HashMap, fs, io::Write, path::Path};
fn main() {
    linker_be_nice();
    create_display_symbols();
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    // make sure linkall.x is the last linker script (otherwise might cause problems with flip-link)
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}

fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let kind = &args[1];
        let what = &args[2];

        match kind.as_str() {
            "undefined-symbol" => match what.as_str() {
                "_defmt_timestamp" => {
                    eprintln!();
                    eprintln!("💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`");
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                _ => (),
            },
            // we don't have anything helpful for "missing-lib" yet
            _ => {
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    println!(
        "cargo:rustc-link-arg=-Wl,--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}

use postcard::to_stdvec;
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct CharBitMap<'a> {
    #[serde(borrow)]
    pub map: HashMap<&'a str, [u8; 8]>,
}
fn create_display_symbols() {
    // check for the symbols.json files;
    let symbols_file_path = "./display_symbols.json";
    let symbols_file_postcard = "./display_symbols.txt";
    let symbols_json =
        fs::read_to_string(symbols_file_path).expect("can't find display_symbols files");
    let symbols: HashMap<&str, [u8; 8]> = serde_json::from_str(&symbols_json).unwrap();
    let char_map = CharBitMap { map: symbols };
    let mut bytes = to_stdvec(&char_map).unwrap();

    if Path::new(symbols_file_postcard).exists() {
        fs::remove_file(symbols_file_postcard).unwrap();
    }
    let mut file = fs::File::create_new(symbols_file_postcard)
        .expect("failed to create symbols in postcode format");
    file.write_all(&mut bytes)
        .expect("failed to write symbols in postcard format");
}
