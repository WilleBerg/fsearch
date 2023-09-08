use fsearch::config::Config;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        std::process::exit(0);
    });
    match fsearch::run(config) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("Application error {}", e);
            std::process::exit(1);
        }
    }
}
