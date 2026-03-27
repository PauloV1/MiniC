use std::{env, fs, process};

use mini_c::{interpreter::interpret, parser::program, semantic::type_check};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: minic <file.minic>");
        process::exit(1);
    }

    let source = match fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", args[1], e);
            process::exit(1);
        }
    };

    let unchecked = match program(&source) {
        Ok((_, prog)) => prog,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            process::exit(1);
        }
    };

    let checked = match type_check(&unchecked) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Type error: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = interpret(&checked) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
