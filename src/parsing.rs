use std::collections::HashMap;

use crate::tokenization::{Token, TokenPairType};

pub struct Parser {
    tokens : Vec<Token>,
    positon : usize,
    paired_brackets : HashMap<usize, TokenPairType>,    // map associating the end position of brackets with their type.
}

impl Parser {
    fn parse_loop(&mut self) {
        for (i, tok) in self.tokens[self.positon..].iter().enumerate() {
            if *tok == Token::LoopEnd && !self.paired_brackets.contains_key(&(self.positon + i)) {
                self.paired_brackets.insert(
                    self.positon + i,   // i is the position in the slice
                    TokenPairType::Loop
                );
                return;
            }
        }
        panic!("A loop is opened but never closed.")
    }
    
    pub fn new(tok : Vec<Token>) -> Parser {
        Parser{
            tokens: tok,     //moved
            positon: 0,
            paired_brackets: HashMap::new()
        }
    }
    
    pub fn parse(&mut self) {
        while self.positon < self.tokens.len() {
            let token = &self.tokens[self.positon];
            
            if token.is_basic_instruction() {
                // Basic instructions are only one char
                // hence the fact they can't produce syntax errors
            }
            
            // doesn't work, will be fixed with a stack
            if token.is_loop() {
                match token {
                    Token::LoopStart => self.parse_loop(),
                    Token::LoopEnd   => {
                        if self.paired_brackets.get(&self.positon).is_none() {
                            panic!("A loop is closed but never opened.");
                        }
                    },
                    _ => panic!("Non implemented loop token in Parser.parse()"),     // unreachable
                }
            }
            
            self.positon += 1;
        }
    }
}