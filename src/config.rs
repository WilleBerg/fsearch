use std::path::PathBuf;
//use whoami;
use std::env;

pub struct Config {
    pub search_term: String,
    pub nightly: bool,
    pub fresh: bool,
    pub verbose: bool,
    pub thread_count: usize,
    pub search_path: PathBuf,
    pub max_results: u32,
    pub cache_path: String
}

impl Config {
    pub fn build(args: &Vec<String>) -> Result<Config, &'static str> {
        let mut nightly: bool = false;
        let mut thread_count: usize = 10;
        let mut fresh: bool = false;
        let mut verbose: bool = false;
        let mut max_results: u32 = 25;
        let mut search_path: PathBuf = std::env::current_dir().unwrap();

        let user = match env::var("USER") {
            Ok(usr) => usr,
            Err(e) => {
                eprintln!("{}",e);
                return Err("Error getting username");
            },
        };
        let cache_path: String;

        // TODO: FIX THIS
        #[cfg(target_os = "windows")]
        {
            match username() {
                Ok(user) => {
                    // Get the system drive (e.g., "C:") from the environment
                    let drive = env::var("SystemDrive").unwrap_or("C:".to_string());

                    // Construct the full path with drive letter
                    let path = format!("{}\\Users\\{}\\.fsearch", drive, user);
                    cache_path = path;
                }
                Err(e) => return Err("Error getting username: {e}"),
            }
        }
        cache_path = format!("/home/{}/.fsearch", user);
        if args.len() < 2 {
            return Err("Not enough arguments");
        }
        if args.len() >= 3 {
            let mut args_iter = args.into_iter();
            args_iter.next();
            args_iter.next();
            let mut own_path: bool = true;
            let mut own_thread_count: bool = false;
            let mut arg = args_iter.next().unwrap();
            loop {
                if own_path {
                    // Might not work
                    if !arg.contains("--") {
                        own_path = false;
                        if arg.chars().nth(0).unwrap() == '\"' {
                            let mut tmp = "".to_string();
                            loop {
                                tmp += arg;
                                arg = match args_iter.next() {
                                    Some(val) => {
                                        if val.contains('\"') {
                                            search_path = PathBuf::from(arg.trim_matches('\"'));
                                            break;
                                        }
                                        val
                                    },
                                    None => {
                                        search_path = PathBuf::from(arg.trim_matches('\"'));
                                        break;
                                    }
                                }
                            }
                        } else {
                            search_path = PathBuf::from(arg);
                        }
                    }
                }
                if own_thread_count {
                    thread_count = match arg.parse() {
                        Ok(val) => val,
                        Err(e) => {
                            eprintln!("error parsing: {e}");
                            std::process::exit(0);
                        }
                    };
                    own_thread_count = false;
                }
                if arg.contains("--") {
                    match arg.as_str() {
                        "--nightly" => nightly = true,
                        "--fresh" => fresh = true,
                        "--verbose" => verbose = true,
                        "--thread-count" => {
                            own_thread_count = true;
                            continue;
                        }
                        "--max-results" => {
                            max_results = match args_iter.next() {
                                Some(val) => val.parse().expect("Enter a number for max results."),
                                None => break,
                            }
                        }
                        "--help" => {
                            print_help(max_results);
                        }
                        "-h" => {
                            print_help(max_results);
                        }
                        _ => println!("Unknown flag {}", arg),
                    }
                }

                arg = match args_iter.next() {
                    Some(val) => val,
                    None => break,
                }
            }
        }
        let search_term: String = args[1].clone();
        Ok(Config {
            search_term,
            nightly,
            thread_count,
            fresh,
            verbose,
            search_path,
            max_results,
            cache_path,
        })
    }
}
// TODO: Fix this and make sure it prints on -h
fn print_help(max_results: u32) {
    println!("\tfsearch <search_term> [optional flags]\n
              fsearch \"multi word search_term\" [optional flag]\n
              -h | --help - Prints this help text\n
              --max-results <number> - Sets the number of printed results to
              the number that was passed. (Default: {})\n
              --fresh - Recreates the cache by scanning all files on your computer.
              Do this intermittently.\n
              --thread-count <number> - Sets the number of threads used by program to
              the number that was passed.\n
              --verbose - Program will print information while running. Used
              for debugging.\n
             ", max_results);
    std::process::exit(0);
}
