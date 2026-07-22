use crate::constants::{VERSION, help_message, invalid_option_error, invalid_path_error};

use std::path::PathBuf;
use std::process::exit;

pub fn parse_args() -> PathBuf {
    let args: Vec<String> = std::env::args().collect();

    let vault = args
        .iter()
        .skip(1)
        .find(|arg| !arg.starts_with('-'))
        .map(PathBuf::from);

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("{}", help_message(&args[0]));
                exit(0);
            }
            "--version" | "-v" => {
                println!("{}", VERSION);
                exit(0);
            }
            other if other.starts_with('-') => {
                eprintln!("{}", invalid_option_error(&other, &args[0]));
                exit(1);
            }
            _ => {}
        }
    }

    let vault = vault.unwrap_or_default();

    if !vault.as_os_str().is_empty() && !vault.exists() {
        eprintln!("{}", invalid_path_error(&args[0]));
        exit(1);
    }

    vault
}
