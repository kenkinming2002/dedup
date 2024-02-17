mod hex_display;

use hex_display::HexDisplay;

use sha2::Sha256;
use sha2::Digest;

use walkdir::WalkDir;

use clap::Parser;
use clap::ValueEnum;

use indicatif::ProgressIterator;

use time::OffsetDateTime;
use time::UtcOffset;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(ValueEnum, Debug, Clone, Copy)]
enum Action {
    /// Delete all version of files excluding latest
    Prune,
    /// Delete all version of files including latest
    Delete,
}

#[derive(Parser, Debug)]
struct Arg {
    path : Option<PathBuf>,

    /// Delete all version of files excluding latest
    #[arg(short, long)]
    action  : Option<Action>,
}

struct Record {
    filepath : PathBuf,
    ctime : SystemTime,
}

fn main() {
    let arg = Arg::parse();

    let dirpath = arg.path.or_else(|| env::current_dir().ok()).unwrap();
    let entries = WalkDir::new(dirpath)
        .into_iter()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.metadata().unwrap().is_file())
        .collect::<Vec<_>>();

    let mut result = HashMap::<_, Vec<_>>::new();
    for entry in entries.into_iter().progress() {
        let ctime    = entry.metadata().unwrap().created().unwrap();
        let filepath = entry.into_path();

        let mut file = fs::File::open(&filepath).unwrap();
        let mut digest = Sha256::new();
        io::copy(&mut file, &mut digest).unwrap();

        let hash            = digest.finalize();
        let hash : [u8; 32] = hash.as_slice().try_into().unwrap();

        result.entry(hash).or_default().push(Record { filepath, ctime });
    }

    for (hash, mut records) in result {
        if records.len() > 1 {
            println!("Duplicates found:");
            println!("  Hash: {}", HexDisplay::new(&hash));
            println!("  Filepath in ascending order of creation time");

            records.sort_by_key(|record| record.ctime);
            for record in &records {
                let filepath = record.filepath.display();
                let utc_offset = UtcOffset::current_local_offset().unwrap();
                let ctime = OffsetDateTime::from(record.ctime).to_offset(utc_offset);
                println!("    Filename: {filepath}, Creation Time {ctime}");
            }

            if let Some(action) = arg.action {
                match action {
                    Action::Prune => { records.pop(); },
                    Action::Delete => {},
                };

                for record in &records {
                    fs::remove_file(&record.filepath).unwrap();
                }

            }
        }
    }

    if let Some(action) = arg.action {
        match action {
            Action::Prune  => println!("All duplicates excluding latest deleted"),
            Action::Delete => println!("All duplicates including latest deleted"),
        }
    }
}
