use std::env;

mod tokenization;

fn main() {
    let args : Vec<String> = env::args().collect();  //reads command line arguments
    
    let contents = std::fs::read_to_string(&args[1])     // same representation as in C, 0th element is the executable's name
        .expect("Please provide a correct filename");
    
    let tokens = tokenization::Token::tokenize(&contents);
    for tok in tokens {
        print!("{}, ", tok);
    }
}
