use std::io::Write;
use std::{error::Error, path::PathBuf};
use std::path::Path;
use std::fs::{ File, OpenOptions};
use walkdir::WalkDir;
use crate::config::Config;

pub mod config;
pub mod cached_file;

// const CACHE_FILE_NAME: &str = "cache";
// const CACHE_SAVE_PATH: &str = "C:\\Users\\willi\\Documents\\GitHub\\fsearch";
const CACHE_FILE_NAME: &str = "cache";
const CACHE_SAVE_PATH: &str = "./";

/// Runs the actual program.
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // let root_dir = "C:\\"; // Replace with your desired directory

    let file_to_find = &config.search_term;
    let nightly = config.nightly;
    println!("File to find: {}\nNightly: {}", file_to_find, nightly);

    let root_dir = "/";
    #[cfg(target_os = "windows")]
    {
        // root_dir = "C:\\Users\\willi";
        root_dir = "C:\\Users\\willi\\Documents\\GitHub\\fsearch";
    }
    let cache_path_part = Path::new(CACHE_SAVE_PATH);
    let cache_path = cache_path_part.join(CACHE_FILE_NAME);
    if nightly {
        // fill_cache_multi_threaded(String::from(root_dir), &cache_path, config.thread_count, &config)?;
    } else {
        let result: Vec<String>;
        if cache_exists(&cache_path, &config).unwrap() {
            result = search_cache_file(&cache_path, &config);
        } else {
            result = fill_cache(String::from(root_dir), &cache_path, &config).unwrap();
        }
        result.into_iter().for_each(| file |{
            println!("{}", file);
        });
    }
    Ok(())
}

/// Checks if cache file exists.
///
/// Returns true if it exists, false if it doesn't (or flag --fresh was user)
fn cache_exists(cache_path: &PathBuf, conf: &Config) -> Result<bool, Box<dyn Error>> {
    if conf.fresh || !cache_path.exists() || !cache_path.is_file() {
        match File::create(cache_path) {
            Ok(_) => return Ok(false),
            Err(e) => return Err(Box::new(e)),
        }
    } else {
        return Ok(true);
    }
}

fn search_cache_file(cache_path: &PathBuf, conf: &Config) -> Vec<String>{
    let mut return_vec: Vec<String> = Vec::new();
    std::fs::read_to_string(cache_path).expect("somethign went wrong reading file")
        .lines()
        .for_each(|line| {
            if line.contains(&conf.search_term) {
                return_vec.push(String::from(line));
            }
        });
    return_vec
}

/// Fills the cache file with all files found recursively starting `cache_path`.
fn fill_cache(path: String, cache_path: &PathBuf, conf: &Config) -> Result<Vec<String>, Box<dyn Error>> {
    let cache_file_path = cache_path;

    // Error handling please!~!!!!
    let mut cache_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&cache_file_path)?;

    let mut return_vec: Vec<String> = Vec::new();
    // Recursively iterate over directory contents
    println!("Filling cache...");
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Skip if the path is not a file
        if !path.is_file() {
            continue;
        }

        // Writes to cache if file isn't already there
        let path_as_string = path.as_os_str().to_str().unwrap().to_string();
        writeln!(cache_file, "{}", path_as_string)?;
        if path_as_string.contains(&conf.search_term) {
            return_vec.push(path_as_string);
        }
    }
    Ok(return_vec)
}
