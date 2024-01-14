use clap::Parser;
use log::{error, trace, LevelFilter};
use rustpy::config::Config;
use simple_logger::SimpleLogger;
use std::fs;
use std::io::{self, Error, Write};

use rustpy::Interpreter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    // Load script
    path: Option<String>,

    // dump trace information, instructions, disassembly, etc.
    #[arg(short, long)]
    trace: bool,
}

fn read_source() -> Result<String, Error> {
    let mut source = String::new();
    loop {
        let mut buffer = String::new();
        let bytes_read = io::stdin().read_line(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        if buffer.ends_with("\\\n") {
            let line = format!("{}\n", buffer.trim_end_matches("\\\n"));
            source.push_str(&line);
            continue;
        }
        source.push_str(&buffer);
        break;
    }
    Ok(source.trim().to_string())
}

fn exec(path: String, config: Config) -> io::Result<()> {
    let source = fs::read_to_string(path)?;

    let mut interpreter = Interpreter::new(config.clone());
    match interpreter.run(&source) {
        Ok(value) => {
            trace!("Result: {:?}", value);
        }
        Err(err) => {
            error!("Error: {:?}", err);
        }
    }
    Ok(())
}

fn repl(config: Config) -> io::Result<()> {
    let mut interpreter = Interpreter::new(config.clone());
    loop {
        print!("> ");
        io::stdout().flush()?;
        let source = read_source()?;
        if source == "quit" {
            break;
        }

        match interpreter.run(&source) {
            Ok(value) => {
                trace!("Result: {:?}", value);
            }
            Err(err) => {
                error!("Error: {:?}", err);
            }
        };
    }
    Ok(())
}

fn main() -> io::Result<()> {
    println!("Rust Python Interpreter");

    SimpleLogger::new().init().unwrap();

    let cli = Args::parse();

    let config = Config { trace: cli.trace };
    if config.trace {
        log::set_max_level(LevelFilter::Trace);
    } else {
        log::set_max_level(LevelFilter::Info);
    }

    if let Some(path) = cli.path {
        exec(path, config)?;
    } else {
        repl(config)?;
    }
    Ok(())
}
