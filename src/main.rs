mod hex_display;

use hex_display::HexDisplay;

use sha2::Sha256;
use sha2::Digest;

use walkdir::WalkDir;

use clap::Parser;

use indicatif::ProgressIterator;

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Parser, Debug)]
struct Arg {
    path : Option<PathBuf>,

    #[arg(short, long)]
    delete : bool,
}

fn main() {
    let arg = Arg::parse();

    let dirpath = arg.path.or_else(|| env::current_dir().ok()).unwrap();
    let filepaths = WalkDir::new(dirpath)
        .into_iter()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.metadata().unwrap().is_file())
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();

    let mut result = HashMap::<_, Vec<_>>::new();
    for filepath in filepaths.iter().progress() {
        let mut file = fs::File::open(&filepath).unwrap();
        let mut digest = Sha256::new();
        io::copy(&mut file, &mut digest).unwrap();

        let hash            = digest.finalize();
        let hash : [u8; 32] = hash.as_slice().try_into().unwrap();

        result.entry(hash).or_default().push(filepath);
    }

    for (hash, filenames) in result {
        if filenames.len() > 1 {
            println!("Duplicates found:");
            println!("  Hash: {}", HexDisplay::new(&hash));
            for filename in filenames {
                println!("  Filename: {}", filename.display());
                if arg.delete {
                    fs::remove_file(filename).unwrap();
                }
            }
        }
    }

    if arg.delete {
        println!("All Duplicates Deleted");
    }
}
