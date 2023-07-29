use std::collections::VecDeque;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{error::Error, path::PathBuf};
use std::path::Path;
use std::fs::{ File, OpenOptions};
use walkdir::WalkDir;
use crate::config::Config;
use regex::Regex;

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
        fill_cache_multithread(String::from(root_dir), &cache_path, &config)?;
    } else {
        let result: Vec<String>;
        if cache_exists(&cache_path, &config).unwrap() {
            result = search_cache_file(&cache_path, &config);
        } else {
            result = fill_cache(String::from(root_dir), &cache_path, &config).unwrap();
        }
        result.clone().into_iter().for_each(| file |{
            println!("{}", file);
        });
        println!("{}", result.len());
    }
    Ok(())
}

fn fill_cache_multithread(path: String, cache_path: &PathBuf, conf: &Config)
    -> Result<Vec<String>, Box<dyn Error>>
{
    let return_vec: Vec<String> = vec![];
    let paths_q = Arc::new(Mutex::new(VecDeque::new()));
    let paths_vec = Arc::new(Mutex::new(Vec::new()));
    let mut counter = 0;
    let mut curr_dir: PathBuf = PathBuf::from(path);
    while counter < conf.thread_count {
        let mut found_dir: bool = false;
        for item in std::fs::read_dir(&curr_dir).unwrap() {
            let path_buf = item.unwrap().path().clone();
            if path_buf.is_dir() {
                if found_dir {
                    curr_dir = path_buf;
                    found_dir = true;
                } else {
                    paths_q.lock().unwrap().push_back(path_buf);
                    counter += 1;
                }
            }
        }
    }
    let mut handles = vec![];
    for thread_id in 0..conf.thread_count {
        let q_clone = paths_q.clone();
        let v_clone = paths_vec.clone();
        let st_clone = conf.search_term.clone();
        let verb = conf.verbose.clone();
        let handle = thread::spawn(move || {
            worker_function(q_clone, v_clone, st_clone, verb);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let mut counter = 0;
    for item in paths_vec.lock().unwrap().iter() {
        println!("{}", item);
        counter += 1;
    }
    println!("{}", counter);
    Ok(return_vec)
}

fn worker_function(safe_q: Arc<Mutex<VecDeque<PathBuf>>>, 
                   safe_vec: Arc<Mutex<Vec<String>>>,
                   search_term: String,
                   verbose: bool)
{
    let mut matching: Vec<String> = Vec::new();
    loop {
        let path: PathBuf;
        {
            let mut q = safe_q.lock().unwrap();
            if q.is_empty() {
                drop(q);
                let mut vec = safe_vec.lock().unwrap();
                matching.iter().for_each(|item| {
                    vec.push(item.to_owned());
                });
                if verbose {
                    println!("Thread done!");
                }
                break;
            }
            path = q.pop_front().unwrap();
            if verbose {
                println!("{}", q.len());
            }
        }
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let path_as_string = path.as_os_str().to_str().unwrap().to_string();
            if path_as_string.contains(&search_term) {
                matching.push(path_as_string);
            }
        }
    }
}

/// Checks if cache file exists.
///
/// Returns true if it exists, false if it doesn't (or flag --fresh was user)
fn cache_exists(cache_path: &PathBuf, conf: &Config)
    -> Result<bool, Box<dyn Error>> 
{
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
    std::fs::read_to_string(cache_path)
        .expect("somethign went wrong reading file")
        .lines()
        .for_each(|line| {
            if line.contains(&conf.search_term) {
                return_vec.push(String::from(line));
            }
        });
    return_vec
}

/// Fills the cache file with all files found recursively starting `cache_path`.
/// If a file matching the search string, it is added to the return `Vec`.
///
/// # Returns:
/// A vector of matching strings.
fn fill_cache(path: String, cache_path: &PathBuf, conf: &Config) 
    -> Result<Vec<String>, Box<dyn Error>> 
{
    let cache_file_path = cache_path;

    // Error handling please!~!!!!
    let mut cache_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&cache_file_path)?;

    let mut return_vec: Vec<String> = Vec::new();
    let rgx: Regex = match Regex::new(&conf.search_term) {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("failed to create regex: {e}");
            std::process::exit(0);
        },
    };
    // Recursively iterate over directory contents
    println!("Filling cache...");
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let path_as_str = path.as_os_str().to_str().unwrap();
        writeln!(cache_file, "{}", path_as_str)?;
        // if path_as_string.contains(&conf.search_term) {
        //     return_vec.push(path_as_string);
        // }
        if rgx.is_match(path_as_str){
             return_vec.push(path_as_str.to_string());
        }
    }
    Ok(return_vec)
}
