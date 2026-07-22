// USER INTERFACE //

pub const TITLE: &str = "NeoNote";
pub const DESCRIPTION: &str = "NNote is a keyboard-first note taking app in your terminal\n\
             Local Markdown notes, simple, quick and lightweight\n\n\
             Start by opening a vault";

// COMMAND-LINE ARGUMENTS //

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn help_message(bin: &str) -> String {
    format!(
        "Note taking application\n\n\
         Usage: {} [OPTIONS] VAULT\n\n\
         Options:\n\
         -h, --help       Print this message\n\
         -v, --version    Print version information",
        bin
    )
}

pub fn invalid_option_error(opt: &str, bin: &str) -> String {
    format!(
        "error: no such option '{}'\n\
         use the option '-h' or '--help' for help\n\n\
         Usage: {} [OPTIONS] VAULT",
        opt, bin
    )
}

pub fn invalid_path_error(bin: &str) -> String {
    format!(
        "error: path does not exist or is not accessible\n\
         use the option '-h' or '--help' for help\n\n\
         Usage: {} [OPTIONS] VAULT",
        bin
    )
}
