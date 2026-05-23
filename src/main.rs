use std::process::{Output, Command};
use std::error::Error;
use clap::Parser;

use crate::parameters::Parameters;
use crate::compilation::Generator;
use crate::interpretation::Interpreter;
use crate::other::*;
use crate::parsing::Instruction;
use crate::tokenization::Token;

mod tokenization;
mod parsing;
mod compilation;
mod interpretation;
mod parameters;
mod other;

fn compilation(cmd_args : Parameters, instructions : Vec<Instruction>) ->  Result<(), Box<dyn Error>> {
    let output_file_asm = format!("{}.asm", cmd_args.get_output_file().unwrap());
    let output_file_o   = format!("{}.o"  , cmd_args.get_output_file().unwrap());
    let output_file     = cmd_args.get_output_file().unwrap();
    write_assembly(
        Generator::new(!cmd_args.get_disable_bound_checking()),
        instructions,
        &output_file_asm,
    )?;
    
    if cmd_args.get_compile_only() {
        println!(
            "Finished compiling `{}`, the generated assembly can be found at `{}`.",
            cmd_args.get_input_file(),
            output_file_asm,
        );
        return Ok(());
    }
    let mut pgrm_output : Output;
    
    // assemble the assembly file with as
    pgrm_output = Command::new("as")
        .arg(if cmd_args.get_dbg_enabled() {"-g"} else {""})
        .arg(&output_file_asm)
        .arg("-o")
        .arg(&output_file_o) //file in [output].o
        .output()
        .expect("The execution of the 'as' command has failed.");
    check_cmd_output(pgrm_output, "as");
    
    
    if cmd_args.get_assemble_only() {
        println!(
            "Finished compiling and assembling `{}`, the generated object file can be found at `{}`.",
            cmd_args.get_input_file(),
            output_file_o,
        );
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
    
    // delete the generated files
    Command::new("rm")
        .arg(&output_file_o)
        .output()
        .expect("Can't delete the object file.");
    
    if !cmd_args.get_dbg_enabled() {
        Command::new("rm")
            .arg(&output_file_asm)
            .output()
            .expect("Can't delete the assembly file.");
    }
    
    println!(
        "Finished compilation of `{}`, the binary executable can be found at `{}`.",
        cmd_args.get_input_file(),
        cmd_args.get_output_file().unwrap(),
    );
    
    Ok(())
}

fn execution(cmd_args : Parameters, instructions : Vec<Instruction>) ->  Result<(), Box<dyn Error>> {
    let mut interpreter = Interpreter::new(!cmd_args.get_disable_bound_checking());
    
    interpreter.exec_config_functions(&instructions)?;
    for instr in instructions {
        interpreter.exec_instr(&instr)?;
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = Parameters::parse();
    if arguments.get_disable_ebf() && arguments.get_input_file().ends_with(".ebf") {
        panic!("Disabled extended brainfuck on an extended brainfuck file.");
    }
    
    let contents = std::fs::read_to_string(arguments.get_input_file())
        .expect("Please provide a correct filename. You may also not have the necessary permissions.");
    
    let mut tokens = tokenization::Token::tokenize(contents, !arguments.get_disable_ebf());
    if !arguments.get_disable_reordering() {
        Token::reorder_opposites(&mut tokens);
    }
    
    let mut instructions = Instruction::parse(tokens);
    if !arguments.get_disable_simplification() {
        let mut reducer = parsing::Reducer::new(instructions);
        reducer.reduce();
        instructions = reducer.clone_instructions();
    }
    
    if arguments.is_compilation() {
        compilation(arguments, instructions)
    } else if arguments.is_interpretation() {
        execution(arguments, instructions)
    } else {
        unreachable!()
    }
}
