#[macro_use]
extern crate clap;

mod vbf_parser;
use clap::{App, Arg};
use std::io;

fn run() -> io::Result<()> {
    /* get parameters from input */
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .arg(
            Arg::with_name("VBB")
            .takes_value(true)
            .value_name("VBB")
            .help("VBB scripts should be append")
        )
        .get_matches();

    let vbb_path = matches.value_of("VBB").unwrap();
    let result = vbf_parser::VbfFt::new(vbb_path);
    match result {
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
        _ => Ok(())
    }
}

fn main() {
    let result = run();
    match result {
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
        Ok(()) => {}
    }
}
