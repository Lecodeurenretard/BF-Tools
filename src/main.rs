use std::process::{Output, Command};
use std::error::Error;
use clap::Parser;

use crate::parameters::Parameters;
use crate::tokenization::{Token, simplify_token_list};
use crate::compilation::Generator;
use crate::other::{write_assembly, check_cmd_output};

mod tokenization;
mod parsing;
mod compilation;
mod parameters;
mod other;

fn main() -> Result<(), Box<dyn Error>>{
    let arguments = Parameters::parse();
    
    let contents = std::fs::read_to_string(arguments.get_input_file())     // same representation as in C, 0th element is the executable's name
        .expect("Please provide a correct filename");
    
    let tokens = tokenization::Token::tokenize(&contents);
    let mut parser = parsing::Parser::new(
        tokens.clone(),
    );
    
    let simplified_tokens : Vec<Token>;
    if arguments.get_disable_simplification() {
        simplified_tokens = tokens;
    } else {
        simplified_tokens = simplify_token_list(tokens)
    }
    
    parser.parse();
    
    let output_file_asm = format!("{}.asm", arguments.get_output_file());
    let output_file_o   = format!("{}.o"  , arguments.get_output_file());
    let output_file     = arguments.get_output_file();
    write_assembly(
        Generator::new(arguments.get_cell_count()),
        simplified_tokens,
        &output_file_asm,
    )?;
    
    if arguments.get_compile_only() {
        return Ok(());
    }
    let mut pgrm_output : Output;
    
    // assemble the assembly file with as
    pgrm_output = Command::new("as")
        .arg(if arguments.get_dbg_enabled() {"-g"} else {""})
        .arg(&output_file_asm)
        .arg("-o")
        .arg(&output_file_o) //file in [output].o
        .output()
        .expect("The execution of the 'as' command has failed.");
    check_cmd_output(pgrm_output, "as");
    
    
    if arguments.get_assemble_only() {
        return Ok(());
    }
    // link the object file with ld
    pgrm_output = Command::new("ld")
        .arg(&output_file_o)
        .arg("-o")
        .arg(output_file)
        .output()
        .expect("The execution of the 'ld' command has failed.");
    check_cmd_output(pgrm_output, "ld");
    
    // delete the geneated files
    Command::new("rm")
        .arg(&output_file_o)
        .output()
        .expect("Can't delete the object file.");
    
    if !arguments.get_dbg_enabled() {
        Command::new("rm")
            .arg(&output_file_asm)
            .output()
            .expect("Can't delete the object file.");
    }
    
    println!(
        "Finished compiling `{}`, the binary executable can be found at `{}`.",
        arguments.get_input_file(),
        arguments.get_output_file(),
    );
    Ok(())
}
