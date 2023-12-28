use std::io::{self, Error, Write};

use rustpy::Interpreter;

fn repl() -> Result<(), Error> {
    let mut interpreter = Interpreter::new();
    loop {
        let mut buffer = String::new();
        print!("> ");
        io::stdout().flush()?;
        let bytes_read = io::stdin().read_line(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        let buffer = buffer.trim();
        if buffer == "quit" {
            break;
        }
        interpreter.run(buffer);
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
