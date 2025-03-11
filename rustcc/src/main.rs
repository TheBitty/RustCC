mod parser;
mod compiler;
mod cli;

use cli::CLI;

fn main() {
    let cli = CLI::new();
    match cli.run() {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
