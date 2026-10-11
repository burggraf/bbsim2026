#[path = "support/demo.rs"]
mod demo;

use std::io;

fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let autoplay = match args.as_slice() {
        [] => false,
        [arg] if arg == "--autoplay" => true,
        [arg] if arg == "--help" => {
            println!("Scripted engine demo (not a simulated matchup).\nUsage: scripted_game [--autoplay]\nDefault: Enter advances one play; q or EOF stops. Rerun to restart.");
            return Ok(());
        }
        _ => {
            eprintln!("Usage: scripted_game [--autoplay] (or --help)");
            std::process::exit(2);
        }
    };
    demo::run(&mut io::stdin().lock(), &mut io::stdout().lock(), autoplay)
}
