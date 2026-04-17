use std::{env, error::Error, io::Write};

use crate::tokenization::Token;
use crate::compilation::Generator;

mod tokenization;
mod parsing;
mod compilation;
mod parameters;



fn write_assembly(tokens : Vec<Token>) -> Result<(), std::io::Error> {
    std::fs::create_dir("build")?;
    let mut output = std::fs::File::create("build/output.asm")?;
    
    output.write(
        Generator::gen_init().as_bytes()
    )?;
    
    for tok in tokens {
        output.write(
            Generator::gen_token(tok).as_bytes()
        )?;
    }
    
    output.write(
        Generator::gen_exit().as_bytes()
    )?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>>{
    let args : Vec<String> = env::args().collect();  //reads command line arguments
    if args.len() == 1 {    // can't be 0
        panic!("Missing file argument.");
    }
    
    let contents = std::fs::read_to_string(&args[1])     // same representation as in C, 0th element is the executable's name
        .expect("Please provide a correct filename");
    
    let tokens = tokenization::Token::tokenize(&contents);
    let mut parser = parsing::Parser::new(
        tokens.clone()
    );
    
    parser.parse();
    
    write_assembly(tokens)?;
    Ok(())
}
