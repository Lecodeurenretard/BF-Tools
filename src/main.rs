use std::process::Output;
use std::{error::Error, io::Write, process::Command};
use clap::Parser;

use crate::parameters::Parameters;
use crate::tokenization::Token;
use crate::compilation::Generator;

mod tokenization;
mod parsing;
mod compilation;
mod parameters;



fn write_assembly(generator : Generator, tokens : Vec<Token>, output_file : &str) -> Result<(), std::io::Error> {
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
        generator.gen_exit(0).as_bytes()
    )?;
    
    Ok(())
}

fn check_prgm_output(pgrm_output : Output, pgrm_name : &str) {
    if !pgrm_output.status.success() {
        let exit_status = pgrm_output.status.code()
            .expect(&format!("{pgrm_name} was terminated by a signal."));
        panic!(
            "{pgrm_name} ended with status {exit_status}, it printed the following errors:\n{}",
            String::from_utf8(pgrm_output.stderr)
                .expect(&format!("{pgrm_name} failed and while decoding its output in stderr, an UTF8 error was raised.")),
        );
    }
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
    
    let output_file_asm = format!("{}.asm", arguments.get_output_file());
    let output_file_o   = format!("{}.o"  , arguments.get_output_file());
    let output_file     = arguments.get_output_file();
    write_assembly(
        Generator::new(arguments.get_cell_count()),
        tokens,
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
    check_prgm_output(pgrm_output, "as");
    
    
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
    check_prgm_output(pgrm_output, "ld");
    
    // delete the geneated files
    Command::new("rm")
        .arg(output_file_o)
        .output()
        .expect("Can't delete the object file.");
    
    if !arguments.get_dbg_enabled() {
        Command::new("rm")
            .arg(output_file_asm)
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
