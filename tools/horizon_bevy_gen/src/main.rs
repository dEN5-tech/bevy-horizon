use std::env;
use std::fs;
use std::path::PathBuf;
use horizon_bevy_gen::ast::parser::parse_rust_source;
use horizon_bevy_gen::generate_from_paths;

fn print_usage() {
    eprintln!(
        "Usage:\n\
         horizon-bevy-gen inspect <rust-file>          - Inspect & dump AST in JSON format (rusty-ast)\n\
         horizon-bevy-gen generate <in-path> <out-dir>  - Generate first-class Bevy crate from Horizon Rust sources\n"
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "inspect" => {
            if args.len() < 3 {
                eprintln!("Error: missing <rust-file> for inspect");
                std::process::exit(1);
            }
            let file_path = &args[2];
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {file_path}: {e}");
                    std::process::exit(1);
                }
            };
            let module = match parse_rust_source(&content, file_path) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("AST parse failed: {e}");
                    std::process::exit(1);
                }
            };
            let json = serde_json::to_string_pretty(&module).unwrap();
            println!("{json}");
        }
        "generate" => {
            if args.len() < 4 {
                eprintln!("Error: usage: horizon-bevy-gen generate <in-path> <out-dir>");
                std::process::exit(1);
            }
            let in_path = PathBuf::from(&args[2]);
            let out_dir = PathBuf::from(&args[3]);

            println!("Parsing Horizon Rust AST from: {}", in_path.display());
            println!("Target Bevy crate destination: {}", out_dir.display());

            match generate_from_paths(&[in_path.as_path()], out_dir.as_path()) {
                Ok(_) => {
                    println!("[SUCCESS] Generated complete `bevy_horizon` crate at: {}", out_dir.display());
                }
                Err(e) => {
                    eprintln!("[ERROR] Generation failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            print_usage();
            std::process::exit(1);
        }
    }
}
