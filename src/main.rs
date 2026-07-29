use std::ffi::OsString;

use clap::Parser;

mod cli;
mod commands;
mod config;
mod error;
mod util;

use cli::Cli;
use config::resolve_home;
use error::{MaehError, Result};

fn main() {
    if let Err(err) = run(std::env::args_os().skip(1).collect()) {
        match err {
            MaehError::Clap(err) => {
                let _ = err.print();
                std::process::exit(err.exit_code());
            }
            err => {
                eprintln!("maeh error: {err}");
                std::process::exit(1);
            }
        }
    }
}

fn run(args: Vec<OsString>) -> Result<()> {
    let cli = Cli::try_parse_from(std::iter::once(OsString::from("maeh")).chain(args))?;
    let home = match cli.home {
        Some(home) => home,
        None => resolve_home()?,
    };
    match cli.command {
        Some(command) => command.dispatch(&home),
        None => {
            print_concise_help();
            Ok(())
        }
    }
}

fn print_concise_help() {
    println!("Typed orchestration CLI for hmph and Herdr agents");
    println!();
    println!("Usage: maeh [--home PATH] <command>");
    println!();
    println!("Examples:");
    println!("  maeh init");
    println!("  maeh doctor");
    println!();
    println!(
        "Run `maeh --help` for the full command list and `maeh <command> --help` for details."
    );
}
