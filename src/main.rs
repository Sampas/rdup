use std::collections::HashMap;
use walkdir::WalkDir;

fn main() {
    let mut fn_counts = HashMap::new();
    let mut fn_paths: HashMap<String, Vec<String>> = HashMap::new();

    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let f_name = String::from(entry.file_name().to_string_lossy());
        let f_path = String::from(entry.path().to_string_lossy());
        fn_paths
            .entry(f_name.clone())
            .and_modify(|paths| paths.push(f_path.clone()))
            .or_insert(vec![f_path.clone()]);

        let counter = fn_counts.entry(f_name.clone()).or_insert(0);
        *counter += 1;
    }

    for (f_name, counter) in fn_counts.into_iter() {
        if counter > 1 {
            println!("{}: {:?}", f_name, fn_paths.get(&f_name).unwrap());
        }
    }
}
