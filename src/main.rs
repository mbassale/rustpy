use std::io::{self, Error, Write};

use rustpy::Interpreter;

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

fn repl() -> Result<(), Error> {
    let mut interpreter = Interpreter::new();
    loop {
        print!("> ");
        io::stdout().flush()?;
        let source = read_source()?;
        println!("Source:\n\"{}\"\n", source);
        if source == "quit" {
            break;
        }

        match interpreter.run(&source) {
            Ok(value) => {
                dbg!(value);
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        };
    }
    Ok(())
}

fn main() {
    println!("Rust Python Interpreter");
    match repl() {
        Ok(_) => (),
        Err(err) => eprintln!("Err: {:?}", err),
    };
}
