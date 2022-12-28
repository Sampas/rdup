use clap::{Parser, ValueEnum};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fmt, fs, io};
use walkdir::WalkDir;

#[derive(Clone, Debug, PartialEq, ValueEnum)]
enum OutputFormat {
    Console,
    Csv,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Console)]
    format: OutputFormat,
    /// Check only filenames and file sizes
    #[arg(short, long, default_value_t = false)]
    name_only: bool,
    /// Hash only files with duplicate names
    #[arg(short, long)]
    hash_only_dup_names: bool,
    /// List empty files
    #[arg(long, action=clap::ArgAction::Set, default_value_t = true)]
    list_empty: bool,
    /// Check only files with size greater or equal to value
    #[arg(short, long, default_value_t = 0)]
    min_size: u64,
    /// Root path to walk through
    root: Option<std::path::PathBuf>,
}

struct Paths<'a>(pub &'a Vec<PathBuf>);

impl<'a> fmt::Display for Paths<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut iter = self.0.iter();
        if let Some(result) = iter.next() {
            write!(f, "{}", result.to_string_lossy())?;
            iter.fold(Ok(()), |result, path| {
                result.and_then(|_| write!(f, ",{}", path.to_string_lossy()))
            })
        } else {
            Ok(())
        }
    }
}

fn main() {
    let args = Args::parse();

    let mut fn_paths: HashMap<(String, u64), Vec<PathBuf>> = HashMap::new();
    let mut empty_files = vec![];

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
        .filter(|e| e.file_type().is_file())
    {
        let f_name = String::from(entry.file_name().to_string_lossy());
        let f_path = entry.path().to_path_buf();
        let f_size = entry.metadata().unwrap().len();
        let is_empty = f_size == 0;

        if is_empty {
            empty_files.push(f_path.clone());
        } else if f_size >= args.min_size {
            fn_paths
                .entry((f_name.clone(), f_size))
                .and_modify(|paths| {
                    paths.push(f_path.strip_prefix("./").unwrap_or(&f_path).to_path_buf())
                })
                .or_insert(vec![f_path
                    .strip_prefix("./")
                    .unwrap_or(&f_path)
                    .to_path_buf()]);
        }
    }

    if args.name_only {
        if args.format == OutputFormat::Console {
            println!("{} issues found:", fn_paths.len());
        }
        for (f_name_size, f_paths) in fn_paths.into_iter() {
            if f_paths.len() > 1 {
                if args.format == OutputFormat::Csv {
                    println!("{},{},{}", f_name_size.0, f_name_size.1, Paths(&f_paths));
                } else {
                    println!("{} ({} B):", f_name_size.0, f_name_size.1);
                    for path in f_paths {
                        println!("    {}", path.to_string_lossy());
                    }
                    println!("");
                }
            }
        }
    } else {
        let mut fn_hashes: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for (_, f_paths) in fn_paths.into_iter() {
            if args.hash_only_dup_names && f_paths.len() <= 1 {
                continue;
            }
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
                if args.format == OutputFormat::Csv {
                    println!("{},{}", hash, Paths(&f_paths));
                } else {
                    println!("{}:", hash);
                    for path in f_paths {
                        println!("    {}", path.display());
                    }
                    println!("");
                }
            }
        }
    }

    if args.list_empty && args.format == OutputFormat::Console {
        println!("{} empty files:", empty_files.len());
        for path in empty_files {
            println!("    {}", path.strip_prefix("./").unwrap_or(&path).display());
        }
    }
}
