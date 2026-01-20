mod compile;
mod error;
use std::{
    fs::{self, File},
    io::{BufWriter, Write, stderr},
    path::Path,
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
    s: bool,

    #[arg(long)]
    tacky: bool,

    #[arg(long)]
    validate: bool,
}

//driver
fn main() {
    let cli = Cli::parse();

    let path = Path::new(&cli.file);
    let stem = path.with_extension("");

    //use preprocesser, emit intermediate file
    let preprocess = Command::new("gcc")
        .arg("-E")
        .arg("-P")
        .arg(&cli.file)
        .arg("-o")
        .arg(stem.with_extension("i"))
        .output()
        .expect("failed to execute preprocessor");
    if !preprocess.status.success() {
        exit(1);
    }

    //actually call compiler
    let code = fs::read_to_string(stem.with_extension("i"))
        .expect("failed to read intermediate post processing file");
    //we can remove immedaitely since its in memory
    let _ = fs::remove_file(stem.with_extension("i"));
    let mut asm_file_writer =
        BufWriter::new(File::create(stem.with_extension("s")).expect("failed to create asm file"));
    let compile = compile::compile(
        &mut asm_file_writer,
        &code,
        cli.lex,
        cli.parse,
        cli.codegen,
        cli.tacky,
        cli.validate,
    );
    //dont care if removing fails
    match compile {
        Ok(_) => (),
        Err(e) => {
            let _ = fs::remove_file(stem.with_extension("s"));
            eprintln!("{}", e.to_string());
            exit(1)
        }
    }

    //assembler and linker
    if !cli.s && !(cli.lex || cli.parse || cli.codegen || cli.tacky || cli.validate) {
        let assemble = Command::new("gcc")
            .arg(stem.with_extension("s"))
            .arg("-o")
            .arg(&stem)
            .output()
            .expect("failed to execute assembler and linker");
        let _ = fs::remove_file(stem.with_extension("s"));
        if !assemble.status.success() {
            stderr()
                .write_all(&assemble.stderr)
                .expect("failed to write assembler stage error to stderr");
            exit(1);
        }
    } else if !cli.s {
        let _ = fs::remove_file(stem.with_extension("s"));
    }
}
