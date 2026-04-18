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
        // TODO: not implemented
        String::from(
            ""
        )
    }
    
    fn gen_mem_next(&self) -> String {
        Generator::check_ptr(1)
        + &String::from(
            "\tadd rbx, 1\n"
        )
    }
    
    fn gen_mem_prev(&self) -> String {
        Generator::check_ptr(-1) 
        + &String::from(
            "\tsub rbx, 1\n"
        )
    }
    
    fn gen_cell_inc(&self) -> String {
        String::from(
            "\tinc byte ptr [rbx]\n"
        )
    }

    fn gen_cell_dec(&self) -> String {
        String::from(
            "\tdec byte ptr [rbx]\n"
        )
    }
    
    fn gen_cell_read(&self) -> String {
        String::from(concat!(
            "\tmov rax, 0\n",
            "\tmov rdi, 0\n",
            "\tsub rsp, 8\n",    // reserve 8 byte on the stack (7 unused)
            "\tmov rsi, rsp\n",  // read() will write 1 byte on the stack
            "\tmov rdx, 1\n",
            "\tsyscall\n",
            "\tpop rdx\n",       // the byte written by read() is in dl
            "\tmov byte [rbx-1], dl\n", // no clue why the -1
        ))
    }
    
    fn gen_cell_write(&self) -> String {
        String::from(concat!(
            "\tmov rax, 1\n",
            "\tmov rdi, 1\n",
            "\tmov rsi, rbx\n",
            "\tmov rdx, 1\n",
            "\tsyscall\n",
        ))
    }
    
    fn gen_loop_start(&self) -> String {
        String::from("")    // not implemented
    }
    
    fn gen_loop_end(&self) -> String {
        String::from("")    // not implemented
    }
    
    pub fn gen_init(&self) -> String {
        format!(concat!(
            ".global _start\n",
            ".intel_syntax noprefix\n\n",
            
            ".section .data\n",
            // error messages go here
            
            ".section .text\n",
            "_start:\n",
            
            // get starting heap address
            "\tmov rax, 12\n",
            "\tmov rdi, 0\n",
            "\tsyscall\n\n",
            
            // save heap boundaries
            "\tmov r12, rax\n",
            "\tlea r13, [r12 + {}]\n\n",
            
            // allocate memory
            "\tmov rax, 12\n",
            "\tmov rdi, r13\n",
            "\tsyscall\n\n",
            
            // sets the memory pointer to the begining of the heap
            "\tmov rbx, r12\n\n",
        ), self.cell_count)
    }
    
    pub fn gen_exit(&self) -> String {
        String::from(concat!(
            "\n",
            "\tmov rax, 60\n",
            "\tmov rdi, 0\n",
            "\tsyscall\n",
        ))
    }
    
    pub fn gen_token(&self, tok : Token) -> String {
        match tok {
            Token::MemNext   => self.gen_mem_next(),
            Token::MemPrev   => self.gen_mem_prev(),
            Token::CellInc   => self.gen_cell_inc(),
            Token::CellDec   => self.gen_cell_dec(),
            Token::Read      => self.gen_cell_read(),
            Token::Write     => self.gen_cell_write(),
            Token::LoopStart => self.gen_loop_start(),
            Token::LoopEnd   => self.gen_loop_end(),
        }
    }
}