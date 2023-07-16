use fsearch::Config;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(| err | {
        println!("Problem parsing arguments: {err}");
        std::process::exit(0);
    });
    fsearch::run(&config.search_term).unwrap();
}
