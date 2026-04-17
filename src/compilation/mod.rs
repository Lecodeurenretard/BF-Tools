use crate::tokenization::Token;

pub struct Generator {
    cell_count : usize,
}

impl Generator {
    pub fn new(cell_count : usize) -> Generator {
        Generator { 
            cell_count,
        }
    }
    
    fn check_ptr(_offset : isize) -> String {
        // not implemented
        String::from(
            ""
        )
    }
    
    fn gen_mem_next(&self) -> String {
        Generator::check_ptr(1)
        + &String::from(
            "\tadd rbp, 1\n"
        )
    }
    
    fn gen_mem_prev(&self) -> String {
        Generator::check_ptr(-1) 
        + &String::from(
            "\tsub rbp, \n"
        )
    }
    
    pub fn gen_init(&self) -> String {
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
        ), self.cell_count))
    }
    
    pub fn gen_exit(&self) -> String {
        String::from(concat!(
            "\tmov rax, 60\n",
            "\tmov rdi, 0\n",
            "\tsyscall\n",
        ))
    }
    
    pub fn gen_token(&self, tok : Token) -> String {
        match tok {
            Token::MemNext => self.gen_mem_next(),
            Token::MemPrev => self.gen_mem_prev(),
            _ => String::new()
        }
    }
}