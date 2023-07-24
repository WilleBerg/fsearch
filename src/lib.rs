use std::io::Write;
use std::thread;
use std::sync::{Mutex, Arc};
use std::{error::Error, path::PathBuf};
use std::path::Path;
use walkdir::WalkDir;
use std::fs::{ File, OpenOptions, DirEntry };
use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};

const CACHE_FILE_NAME: &str = "cache";
const CACHE_SAVE_PATH: &str = "C:\\Users\\willi\\Documents\\GitHub\\fsearch";

pub struct Config {
    pub search_term: String,
    pub nightly: bool,
    pub fresh: bool,
    pub thread_count: usize,
}

impl Config {
    pub fn build(args: &Vec<String>) -> Result<Config, &'static str> {
        let mut nightly: bool = false;
        let mut thread_count: usize = 10;
        let mut fresh: bool = false;
        if args.len() < 2 {
            return Err("Not enough arguments");
        }
        if args.len() >= 3 {
            let mut args_iter = args.into_iter();
            args_iter.next();
            args_iter.next();
            for arg in args_iter {
                if arg.contains("--") {
                    match arg.as_str() {
                        "--nightly" => nightly = true,
                        "--fresh" => fresh = true,
                        _ => println!("Unknown flag {}", arg),
                    }
                }
            }
        }
        let search_term: String = args[1].clone();
        Ok(Config { search_term, nightly, thread_count, fresh})
    }
}

#[derive(Debug, Eq, Clone)]
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

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // let root_dir = "C:\\"; // Replace with your desired directory

    let file_to_find = &config.search_term;
    let nightly = config.nightly;
    println!("File to find: {}\nNightly: {}", file_to_find, nightly);


    let root_dir = "C:\\Users\\willi";
    // let root_dir = "C:\\Users\\willi\\Documents\\GitHub\\fsearch";
    let cache_path_part = Path::new(CACHE_SAVE_PATH);
    let cache_path = cache_path_part.join(CACHE_FILE_NAME);
    if nightly {
        fill_cache_multi_threaded(String::from(root_dir), &cache_path, config.thread_count, &config)?;
    } else {
        let cache: HashSet<CachedFile>;
        if cache_exists(&cache_path, &config).unwrap() {
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
                    println!("Files found:");
                    result.into_iter().for_each(| file |{
                        println!("{}", file);
                    });
                }
            },
            None => println!("No file found"),
        }
    }
    Ok(())
}

// Currently does not work
fn get_hash_cache(cache_path: &PathBuf) -> Result<HashSet<CachedFile>, Box<dyn Error>> {
    let mut set: HashSet<CachedFile> = HashSet::new();
    std::fs::read_to_string(cache_path)?
        .lines()
        .for_each(|line| {
            let cached_file = CachedFile::create(line);
            if set.contains(&cached_file) {
                // println!("Already found: {}\nThis time at: {}", cached_file.name, line);
                let mut taken = set.take(&cached_file).unwrap();
                let tup = split_path(line);
                if !taken.paths.contains(&tup.0) {
                    taken.paths.push(tup.0);
                }
                set.insert(taken);
            } else {
                set.insert(cached_file);
            }
        });
    Ok(set)
}

fn search_cache(file_to_find: &String, cache: &HashSet<CachedFile>) -> Option<Vec<String>> {
    let file_search_struct = CachedFile { name: file_to_find.clone(), paths: vec![] };
    let real_file = cache.get(&file_search_struct)?;
    Some(real_file.full_paths())
}

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
    let cache_file_path = cache_path;

    // Error handling please!~!!!!
    let mut cache_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&cache_file_path)?;

    let mut cache_set: HashSet<CachedFile> = HashSet::new();
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
            let mut taken = cache_set.take(&cached_file)
                .expect("Something went wrong getting the cached file");
            let tup = split_path(&path_as_str);
            if !taken.paths.contains(&tup.0) {
                writeln!(cache_file, "{}", path_as_string)?;
                taken.paths.push(tup.0);
            }
            cache_set.insert(taken);
        }
    }
    Ok(cache_set)
}

fn fill_cache_multi_threaded(path: String, cache_path: &PathBuf, thread_count: usize, config: &Config)
    -> Result<HashSet<CachedFile>, Box<dyn Error>> 
{
    let dirs = std::fs::read_dir(path)?;
    let dir_vec: Vec<_> = dirs.collect();
    let mut dir_vecdeq: VecDeque<_> = dir_vec.into_iter()
                                             .map(|e| e.ok())
                                             .filter_map(|entry|{
                                                if let Some(en) = entry {
                                                    if en.path().is_dir() {
                                                        Some(en)
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                }
                                             }).collect::<VecDeque<_>>();
    while dir_vecdeq.len() < 10 {
        let curr = dir_vecdeq.pop_front().unwrap();
        let tmp = std::fs::read_dir(curr.path())?.collect::<Vec<_>>();
        tmp.into_iter().map(|e| e.ok())
                       .filter_map(|entry|{
                            if let Some(en) = entry {
                                if en.path().is_dir() {
                                    Some(en)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                            }).for_each(|a| dir_vecdeq.push_back(a));
    }
    let safe_q: Arc<Mutex<VecDeque<DirEntry>>> = Arc::new(Mutex::new(dir_vecdeq));
    let safe_set: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
    let mut handles = vec![];
    for thread_id in 0..thread_count {
        let work_q_clone = Arc::clone(&safe_q);
        let safe_set_clone = Arc::clone(&safe_set);
        let handle = thread::spawn(move || {
            spawn_worker(work_q_clone, thread_id, safe_set_clone);
        });
        println!("Thread spawned with ID: {}", thread_id);
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    // let r_set = safe_set.lock().unwrap().clone();
    let mut r_set = HashSet::new();
    for item in safe_set.lock().unwrap().iter() {
        if item.is_dir() {
            continue;
        }
        let path = item.to_str().unwrap();
        let cache_file = CachedFile::create(path);
        let s_path = split_path(path);
        if r_set.contains(&cache_file) {
            let mut taken: CachedFile = r_set.take(&cache_file).unwrap();
            if !taken.paths.contains(&s_path.0) {
                taken.paths.push(s_path.0);
            }
            r_set.insert(taken);
        } else {
            r_set.insert(cache_file);
        }
    }


    write_hash_to_cache(&r_set, cache_path, &config)?;
    Ok(r_set)
}

fn write_hash_to_cache(set: &HashSet<CachedFile>, cache_path: &PathBuf, config: &Config) -> Result<(), Box<dyn Error>> {
    let mut cache_file: File;
    let cache_exists: bool = cache_exists(cache_path, &config).unwrap();
    //BAD TEMPORARY CODE
    if cache_exists {
        cache_file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&cache_path)?;
    } else {
        cache_file = OpenOptions::new()
            .write(true)
            .append(false)
            .open(&cache_path)?;
    }
    for item in set {
        for path in item.full_paths() {
            writeln!(cache_file, "{path}")?;
        }
    }
    Ok(())
}

fn spawn_worker(work_queue: Arc<Mutex<VecDeque<DirEntry>>>, 
                thread_id: usize, 
                set: Arc<Mutex<HashSet<PathBuf>>>) 
{
    loop {
        let path: DirEntry;
        {
            let mut queue = work_queue.lock().unwrap();
            if queue.is_empty() {
                println!("Thread with ID {} done", thread_id);
                break;
            }
            path = queue.pop_front().unwrap();
        }
        // Should always be a dir.
        let it_dir = std::fs::read_dir(path.path()).unwrap();
        it_dir.for_each(|item| {
            match item {
                Ok(val) => {
                    if val.path().is_dir() {
                        let mut set = set.lock().unwrap();
                        if !set.contains(&val.path()) {
                            let mut queue = work_queue.lock().unwrap();
                            set.insert(val.path().clone());
                            queue.push_back(val);
                            println!("Queue size: {}", queue.len());
                        }
                    } else {
                        let mut set = set.lock().unwrap();
                        set.insert(val.path());
                    }
                }
                Err(e) => eprintln!("error reading path: {e}"),
            }
        })
    }
}
