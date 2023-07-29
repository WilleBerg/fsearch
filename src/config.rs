use std::path::PathBuf;

pub struct Config {
    pub search_term: String,
    pub nightly: bool,
    pub fresh: bool,
    pub verbose: bool,
    pub thread_count: usize,
    pub search_path: PathBuf,
}

impl Config {
    pub fn build(args: &Vec<String>) -> Result<Config, &'static str> {
        let mut nightly: bool = false;
        let mut thread_count: usize = 10;
        let mut fresh: bool = false;
        let mut verbose: bool = false;
        let mut search_path: PathBuf = std::env::current_dir().unwrap();
        if args.len() < 2 {
            return Err("Not enough arguments");
        }
        if args.len() >= 3 {
            let mut args_iter = args.into_iter();
            args_iter.next();
            args_iter.next();
            let mut own_path: bool = true;
            let mut own_thread_count: bool = false;
            for arg in args_iter {
                if own_path {
                    // Might not work
                    if !arg.contains("--") {
                        own_path = false;
                        search_path = PathBuf::from(arg);
                    }
                }
                if own_thread_count {
                    thread_count = match arg.parse() {
                        Ok(val) => val,
                        Err(e) => {
                            eprintln!("error parsing: {e}");
                            std::process::exit(0);
                        },
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
                        },
                        _ => println!("Unknown flag {}", arg),
                    }
                }
            }
        }
        let search_term: String = args[1].clone();
        Ok(Config { search_term, nightly, thread_count, fresh, verbose, search_path })
    }
}
