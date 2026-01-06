mod compile;

mod error;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, exit},
};

use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    file: String,

    #[arg(long)]
    lex: bool,

    #[arg(long)]
    parse: bool,

    #[arg(long)]
    codegen: bool,

    #[arg(long)]
    S: bool,
}

//driver
fn main() {
    let cli = Cli::parse();

    let stem = Path::new(&cli.file)
        .file_stem()
        .expect("failed to extract file stem");

    //use preprocesser, emit intermediate file
    let preprocess = Command::new("gcc")
        .arg("-E")
        .arg("-P")
        .arg(&cli.file)
        .arg("-o")
        .arg(PathBuf::from(stem).with_extension("i"))
        .output()
        .expect("failed to execute preprocessor");
    if !preprocess.status.success() {
        exit(1);
    }

    //actually call compiler
    let compile = compile::compile(
        PathBuf::from(stem)
            .with_extension("i")
            .to_str()
            .expect("not sure how we failed to add a file extension lmfao"),
        cli.lex,
        cli.parse,
        cli.codegen,
    );
    //dont care if removing fails
    let _ = fs::remove_file(PathBuf::from(stem).with_extension("i"));
    match compile {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{}", e.to_string());
            exit(1)
        }
    }

    //assembler and linker
    if !cli.S && !(cli.lex || cli.parse || cli.codegen) {
        let assemble = Command::new("gcc")
            .arg(PathBuf::from(stem).with_extension("s"))
            .arg("-o")
            .arg(PathBuf::from(stem))
            .output()
            .expect("failed to execute assembler and linker");
        let _ = fs::remove_file(PathBuf::from(stem).with_extension("s"));
        if !assemble.status.success() {
            exit(1);
        }
    }
}
