use clap::Parser;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Check only filenames
    #[arg(short, long, default_value_t = false)]
    name_only: bool,
    /// Root path to walk through
    root: Option<std::path::PathBuf>,
}

fn main() {
    let args = Args::parse();

    let mut fn_paths: HashMap<String, Vec<std::path::PathBuf>> = HashMap::new();

    let root = if let Some(path) = args.root {
        path
    } else {
        PathBuf::from(".")
    };

    if !root.exists() {
        panic!("\"{}\" directory does not exist", root.to_string_lossy());
    }

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let f_name = String::from(entry.file_name().to_string_lossy());
        let f_path = entry.path().to_path_buf();
        fn_paths
            .entry(f_name.clone())
            .and_modify(|paths| paths.push(f_path.clone()))
            .or_insert(vec![f_path.clone()]);
    }

    if args.name_only {
        for (f_name, f_paths) in fn_paths.into_iter() {
            if f_paths.len() > 1 {
                println!("{}: {:?}", f_name, f_paths);
            }
        }
    } else {
        let mut fn_hashes: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for (_, f_paths) in fn_paths.into_iter() {
            for path in f_paths {
                let mut file = fs::File::open(&path).unwrap();
                let mut hasher = Sha1::new();
                io::copy(&mut file, &mut hasher).unwrap();
                let hash_result = hasher.finalize();
                let hash = format!("{:x}", hash_result);
                fn_hashes
                    .entry(hash.clone())
                    .and_modify(|paths| paths.push(path.clone()))
                    .or_insert(vec![path.clone()]);
            }
        }

        for (hash, f_paths) in fn_hashes.into_iter() {
            if f_paths.len() > 1 {
                println!("{}: {:?}", hash, f_paths);
            }
        }
    }
}
