pub struct Config {
    pub search_term: String,
    pub nightly: bool,
    pub fresh: bool,
    pub verbose: bool,
    pub thread_count: usize,
}

impl Config {
    pub fn build(args: &Vec<String>) -> Result<Config, &'static str> {
        let mut nightly: bool = false;
        let mut thread_count: usize = 10;
        let mut fresh: bool = false;
        let mut verbose: bool = false;
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
                        "--verbose" => verbose = true,
                        _ => println!("Unknown flag {}", arg),
                    }
                }
            }
        }
        let search_term: String = args[1].clone();
        Ok(Config { search_term, nightly, thread_count, fresh, verbose })
    }
}
