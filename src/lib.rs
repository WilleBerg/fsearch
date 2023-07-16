use std::io::{Write};
use std::{error::Error, path::PathBuf};
use std::path::Path;
use walkdir::WalkDir;
use std::fs::{ File, OpenOptions };
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
const CACHE_FILE_NAME: &str = "cache";
const CACHE_SAVE_PATH: &str = "C:\\Users\\willi\\Documents\\GitHub\\fsearch";


pub struct Config {
    pub search_term: String,
}

impl Config {
    pub fn build(args: &Vec<String>) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("Not enough arguments");
        }
        let search_term: String = args[1].clone();
        Ok(Config { search_term })
    }
}

#[derive(Debug, Eq)]
struct CachedFile {
    name: String,
    paths: Vec<String>,
}

impl CachedFile {
    fn create(path: &str) -> CachedFile {
        let tuple = split_path(path);
        CachedFile { name: tuple.1, paths: vec![tuple.0] }
    }

    fn full_paths(&self) -> Vec<String> {
        // self.path.clone() + &"\\" + &self.name
        self.paths.clone().into_iter()
                  .map(|path| path.clone() + &"\\" + &self.name)
                  .collect::<Vec<String>>()
    }
}

impl PartialEq for CachedFile {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for CachedFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

pub fn run(file_to_find: &String) -> Result<(), Box<dyn Error>> {
    let root_dir = "C:\\"; // Replace with your desired directory

    let cache_path_part = Path::new(CACHE_SAVE_PATH);
    let cache_path = cache_path_part.join(CACHE_FILE_NAME);

    let cache: HashSet<CachedFile>;
    if cache_exists(&cache_path).unwrap() {
        cache = match get_hash_cache(&cache_path) {
            Ok(set) => set,
            Err(_) => {
                panic!("Error while reading cache");
            }
        }
    } else {
        cache = fill_cache(String::from(root_dir), &cache_path).unwrap();
    }
    let search_result = search_cache(file_to_find, &cache);
    match search_result {
        Some(result) => {
            if result.len() == 1 {
                println!("File found: {}", result[0]);
            } else {
                println!("Files found: \n");
                result.into_iter().for_each(| file |{
                    println!("{}", file);
                });
            }
        },
        None => println!("No file found"),
    }
    Ok(())
}

fn get_hash_cache(cache_path: &PathBuf) -> Result<HashSet<CachedFile>, Box<dyn Error>> {
    let mut set: HashSet<CachedFile> = HashSet::new();
    std::fs::read_to_string(cache_path)?.lines()
                                       .for_each(|line| {
                                            set.insert(CachedFile::create(line));
                                       });
    Ok(set)
}

fn search_cache(file_to_find: &String, cache: &HashSet<CachedFile>) -> Option<Vec<String>> {
    let file_search_struct = CachedFile { name: file_to_find.clone(), paths: vec![] };
    let real_file = cache.get(&file_search_struct)?;
    Some(real_file.full_paths())
}

fn cache_exists(cache_path: &PathBuf) -> Result<bool, Box<dyn Error>> {

    if !cache_path.exists() || !cache_path.is_file() {
        match File::create(cache_path) {
            Ok(_) => return Ok(false),
            Err(e) => return Err(Box::new(e)),
        }
    } else {
        return Ok(true);
    }
}

// Written by chatgpt
fn split_path(path: &str) -> (String, String) {
    let path_buf = PathBuf::from(path);
    let directory = path_buf
        .parent()
        .map_or("", |parent| parent.to_str().unwrap_or(""));
    let file_name = path_buf
        .file_name()
        .map_or("", |file_name| file_name.to_str().unwrap_or(""));
    (directory.to_string(), file_name.to_string())
}

fn fill_cache(path: String, cache_path: &PathBuf) -> Result<HashSet<CachedFile>, Box<dyn Error>> {
    let cache_exists = cache_exists(&cache_path)?;

    let cache_file_path = cache_path;

    // Error handling please!~!!!!
    let mut cache_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&cache_file_path)?;

    let mut cache_set: HashSet<CachedFile> = HashSet::new();
    if cache_exists {
        std::fs::read_to_string(cache_file_path).unwrap()
                                                .lines()
                                                .for_each(| line | {
                                                    cache_set.insert(CachedFile::create(line));
                                                });
    }
    // Recursively iterate over directory contents
    println!("Filling cache...");
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Skip if the path is not a file
        if !path.is_file() {
            continue;
        }

        // Writes to cache if file isn't already there
        let path_as_str = path.as_os_str().to_str().unwrap();
        let path_as_string = String::from(path_as_str);
        let cached_file: CachedFile = CachedFile::create(path_as_str);

        if !cache_set.contains(&cached_file) {
            writeln!(cache_file, "{}", path_as_string)?;
            cache_set.insert(cached_file);
        } else {
            let mut taken = cache_set.take(&cached_file).expect("Something went wrong getting the cached file");
            if !taken.paths.contains(&path_as_string) {
                taken.paths.push(path_as_string);
            }
            cache_set.insert(taken);
        }
    }
    Ok(cache_set)
}