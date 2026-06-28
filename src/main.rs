mod compile;
mod error;
use std::{
    fs::{File, read_to_string, remove_file},
    io::{BufWriter, Write, stderr},
    path::{Path, PathBuf},
    process::{Command, exit},
};

use clap::Parser;

use crate::compile::compile;

#[derive(Parser, Debug)]
struct Cli {
    files: Vec<String>,

    #[arg(long)]
    lex: bool,

    #[arg(long)]
    parse: bool,

    #[arg(long)]
    codegen: bool,

    #[arg(long)]
    s: bool,

    #[arg(long)]
    c: bool,

    #[arg(long)]
    tacky: bool,

    #[arg(long)]
    validate: bool,
}

//driver
fn main() {
    let cli = Cli::parse();
    let paths = cli.files.into_iter().map(PathBuf::from).collect::<Vec<_>>();

    for path in &paths {
        //use preprocesser, emit intermediate file
        let preprocess = Command::new("gcc")
            .arg("-E")
            .arg("-P")
            .arg(&path)
            .arg("-o")
            .arg(path.with_extension("i"))
            .output()
            .expect("failed to execute preprocessor");
        if !preprocess.status.success() {
            exit(1);
        }

        //actually call compiler
        let code = read_to_string(path.with_extension("i"))
            .expect("failed to read intermediate post processing file");

        //we can remove immedaitely since its in memory
        let _ = remove_file(path.with_extension("i"));

        let mut asm_file_writer = BufWriter::new(
            File::create(path.with_extension("s")).expect("failed to create asm file"),
        );
        match compile(
            &mut asm_file_writer,
            code,
            cli.lex,
            cli.parse,
            cli.codegen,
            cli.tacky,
            cli.validate,
        ) {
            Ok(()) => (),
            Err(e) => {
                for path in paths {
                    let _ = remove_file(path.with_extension("s"));
                }
                eprintln!("{}", e.to_string());
                exit(1);
            }
        }
    }

    //assembler and linker
    if !cli.s && !(cli.lex || cli.parse || cli.codegen || cli.tacky || cli.validate) {
        let mut assemble = Command::new("gcc");
        for path in &paths {
            assemble.arg(path.with_extension("s"));
        }

        let x = assemble
            .arg("-o")
            .arg(paths[0].with_extension(""))
            .output()
            .expect("failed to execute assembler and linker");
        for path in paths {
            let _ = remove_file(path.with_extension("s"));
        }
        if !x.status.success() {
            stderr()
                .write_all(&x.stderr)
                .expect("failed to write assembler stage error to stderr");
            exit(1);
        }
    } else if !cli.s {
        for path in paths {
            let _ = remove_file(path.with_extension("s"));
        }
    } else if cli.c {
        let mut assemble = Command::new("gcc");
        assemble.arg("-c");
        for path in &paths {
            assemble.arg(path.with_extension("s"));
        }

        let x = assemble
            .arg("-o")
            .arg(paths[0].with_extension(""))
            .output()
            .expect("failed to execute assembler and linker");
        for path in paths {
            let _ = remove_file(path.with_extension("s"));
        }
        if !x.status.success() {
            stderr()
                .write_all(&x.stderr)
                .expect("failed to write assembler stage error to stderr");
            exit(1);
        }
    }
}
