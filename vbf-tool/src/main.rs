#[macro_use]
extern crate clap;

use clap::{App, Arg};
use std::io;

const _HEADER: &str = r#"
header {
    //**********************************************************
    //*
    //*                  Volvo Car Corporation
    //*
    //*     This file is generated by VBF CONVERT ver. 5.10.0
    //*
    //*                        DO NOT EDIT !
    //*
    //**********************************************************
"#;

fn run() -> io::Result<()> {
    /* get parameters from input */
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .arg(Arg::with_name("file").help("to do").required(true))
        .arg(
            Arg::with_name("VBB")
            .short("s")
            .takes_value(true)
            .value_name("VBB")
            .help("VBB scripts should be append")
        )
        .get_matches();

    let _filename = matches.value_of("file").unwrap();
    let _vbb_script = matches.value_of("VBB").unwrap();
    println!("input binary: {}", _filename);
    println!("input script: {}", _vbb_script);

    Ok(())
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
