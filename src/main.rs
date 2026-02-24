use std::path::PathBuf;

use chrono::Local;
use clap::Parser;
use lang::{Interpreter, Program};
use log::info;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    search_paths: Vec<PathBuf>,
    #[arg(short, long)]
    ast: bool,
    #[arg(short, long)]
    log: bool,
    #[arg(short, long)]
    debug: bool,
    path: PathBuf,
}

fn run(
    program: &Program,
    search_paths: Vec<PathBuf>,
    path: PathBuf,
    debug: bool,
) -> anyhow::Result<()> {
    let mut interpreter = Interpreter::new(Some(path), search_paths);
    interpreter.debug(debug);
    info!("====================================");
    info!("RUNTIME...");
    info!("====================================");
    match interpreter.run(program) {
        Ok(result) => {
            println!("Result: {}", result.to_string());
            Ok(())
        }
        Err(err) => Err(anyhow::anyhow!(err)),
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    std::fs::create_dir_all("logs")?;
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(format!(
            "logs/{}.log",
            Local::now().format("%Y-%m-%d_%H_%M_%S")
        ))?)
        .apply()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let Cli {
        search_paths,
        ast,
        debug,
        log,
        path,
    } = Cli::parse();
    if log {
        setup_logger()?;
    }
    let code = std::fs::read_to_string(&path)?;

    let mut parser = lang::parser::Parser::new(&code);

    info!("====================================");
    info!("PARSING...");
    info!("====================================");
    match parser.parse() {
        Ok(program) => {
            if ast {
                info!("Program: {:#?}", program);
            }
            run(&program, search_paths, path, debug)?;
        }
        Err(err) => {
            return Err(anyhow::anyhow!("Parse failed: {err}"));
        }
    }

    Ok(())
}
