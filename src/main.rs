use std::{error::Error, io::Write};
use clap::Parser;

use crate::parameters::Parameters;
use crate::tokenization::Token;
use crate::compilation::Generator;

mod tokenization;
mod parsing;
mod compilation;
mod parameters;



fn write_assembly(generator : Generator, tokens : Vec<Token>, output_file : String) -> Result<(), std::io::Error> {
    let mut output = std::fs::File::create(output_file)?;
    
    output.write(
        generator.gen_init().as_bytes()
    )?;
    
    for tok in tokens {
        output.write(
            generator.gen_token(tok).as_bytes()
        )?;
    }
    
    output.write(
        generator.gen_exit().as_bytes()
    )?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>>{
    let arguments = Parameters::parse();
    
    let contents = std::fs::read_to_string(arguments.get_input_file())     // same representation as in C, 0th element is the executable's name
        .expect("Please provide a correct filename");
    
    let tokens = tokenization::Token::tokenize(&contents);
    let mut parser = parsing::Parser::new(
        tokens.clone(),
    );
    
    parser.parse();
    
    write_assembly(
        Generator::new(arguments.get_cell_count()),
        tokens,
        arguments.get_output_file(),
    )?;
    
    Ok(())
}
