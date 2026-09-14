use std::process::exit;

mod cli;
mod config;
mod discovery;
mod error;
mod model;
mod parser;
mod runner;

fn main() {
    match cli::run(cli::parse()) {
        Ok(code) => exit(code),
        Err(error) => {
            eprintln!("error[{}]: {error}", error.code());
            exit(2);
        }
    }
}
