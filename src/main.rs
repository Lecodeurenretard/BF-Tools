use std::env;

mod tokenization;
mod parsing;

fn main() {
    let args : Vec<String> = env::args().collect();  //reads command line arguments
    if args.len() == 0 {
        panic!("Missing file argument.");
    }
    
    let contents = std::fs::read_to_string(&args[1])     // same representation as in C, 0th element is the executable's name
        .expect("Please provide a correct filename");
    
    let mut parser = parsing::Parser::new(
        tokenization::Token::tokenize(&contents)
    );
    
    parser.parse();
    println!("This file is syntaticly correct.")
}
