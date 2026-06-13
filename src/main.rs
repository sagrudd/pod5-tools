use clap::Parser;
use pod5_tools::{Cli, run};

fn main() {
    let cli = Cli::parse();
    match run(cli) {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}
