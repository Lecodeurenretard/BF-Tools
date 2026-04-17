use crate::tokenization::Token;
use crate::parameters::Parameters;

pub struct Generator {}

impl Generator {
    fn check_ptr(_offset : isize) -> String {
        // not implemented
        String::from(
            ""
        )
    }
    
    fn gen_mem_next() -> String {
        Generator::check_ptr(1)
        + &format!(
            "\tadd rbp, {}\n"
        , Parameters::CELL_SIZE)
    }
    
    fn gen_mem_prev() -> String {
        Generator::check_ptr(-1) 
        + &format!(
            "\tsub rbp, {}\n"
        , Parameters::CELL_SIZE)
    }
    
    pub fn gen_init() -> String {
        const BAND_SIZE : usize = Parameters::CELL_COUNT * (Parameters::CELL_SIZE as usize);
        String::from(format!(concat!(
            ".global _start\n",
            ".intel_syntax noprefix\n\n",
            
            ".section .data\n",
            // Error messages go here
            
            ".section .text\n",
            "_start:\n",
            
            // get starting heap address
            "\tmov rax, 12\n",
            "\tmov rdi, 0\n",
            "\tsyscall\n\n",

            // new end of heap ptr in rdi
            "\tadd rax, {}\n",
            "\tmov rdi, rax\n\n",

            // allocate memory
            "\tmov rax, 12\n",
            "\tsyscall\n\n",

            "\tmov rbp, rdi\n\n",
        ), BAND_SIZE))
    }
    
    pub fn gen_exit() -> String {
        String::from(format!(concat!(
            "\tmov rax, 60\n",
            "\tmov rdi, 0\n",
            "\tsyscall\n",
        )))
    }
    
    pub fn gen_token(tok : Token) -> String {
        match tok {
            Token::MemNext => Generator::gen_mem_next(),
            Token::MemPrev => Generator::gen_mem_prev(),
            _ => String::new()
        }
    }
}