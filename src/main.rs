use std::process::ExitCode;

mod cli;
mod commands;
mod output;

fn main() -> ExitCode {
    match cli::run(std::env::args().skip(1).collect()) {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("owly: {message}");
            ExitCode::from(2)
        }
    }
}
