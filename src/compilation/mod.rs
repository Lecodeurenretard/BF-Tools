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
    
    fn init_error_throw(&self, err_name : &str) -> String {
        format!(concat!(
            "\tmov rax, 1\n",
            "\tlea rsi, {name}\n",
            "\tmov rdx, {name}Len\n",
        ), name=err_name)
    }
    
    fn check_ptr(&self, offset : isize) -> String {
        format!(concat!(
            "\tlea r8, [rbx + {offset}]\n",
            "{init_err}",
            "\tcmp r12, r8\n",
            "\tjnbe runtime_error\n",   // jumps if r12 <= r8
            
            // r8, rax, rsi, rdx may change during wmp or jb calls
            "\tlea r8, [rbx + {offset}]\n",
            "{init_err}",
            "\tcmp r8, r13\n",
            "\tjnbe runtime_error\n",   // same
        ), offset=offset, init_err=self.init_error_throw("OoB"))
    }
    
    /// Create an error with its the message
    /// its output must be in the data segment.
    fn create_error(&self, name : &str, message : &str) -> String {
        format!(concat!(
            "\t{name}: .asciz \"{msg}\\n\"\n",
            "\t.set {name}Len, $-{name}\n"
        ), name=name, msg=message)
    }
    
    fn gen_mem_next(&self, count : usize) -> String {
        self.check_ptr(count as isize)
        + &format!(
            "\tadd rbx, {count}\n"
        )
    }
    
    fn gen_mem_prev(&self, count : usize) -> String {
        self.check_ptr(-(count as isize))
        + &format!(
            "\tsub rbx, {count}\n"
        )
    }
    
    fn gen_cell_inc(&self, count : usize) -> String {
        format!(
            "\tadd byte ptr [rbx], {count}\n"
        )
    }

    fn gen_cell_dec(&self, count : usize) -> String {
        format!(
            "\tsub byte ptr [rbx], {count}\n"
        )
    }
    
    fn gen_cell_read(&self, count : usize) -> String {
        // Calling sys_read on std_in
        format!(concat!(
            "\tmov rax, 0\n",
            "\tmov rdi, 0\n",
            "\tsub rsp, {count}\n",    // reserve count bytes (the size of a register) on the stack
            "\tmov rsi, rsp\n",
            "\tmov rdx, {count}\n",
            "\tsyscall\n",
            "\tmov dl, [rsp - 1 + {count}]\n",     // the last value entered
            "\tmov [rbx], dl\n",
        ), count=count)
        + &String::from("\tpop rdx\n").repeat(count)    // freeing the stack
    }
    
    fn gen_cell_write(&self, count : usize) -> String {
        String::from(concat!(
            "\tmov rax, 1\n",
            "\tmov rdi, 1\n",
            "\tmov rsi, rbx\n",
            "\tmov rdx, 1\n",
        ))
        // registers are preserved
        // couldn't find any source but works on my machine
        + &String::from("\tsyscall\n").repeat(count)
    }
    
    fn gen_loop_start(&self) -> String {
        String::from("")    // not implemented
    }
    
    fn gen_loop_end(&self) -> String {
        String::from("")    // not implemented
    }
    
    fn predefined_functions(&self) -> String {
        format!(concat!(
            "runtime_error:\n",
            "\tmov r8, rax\n",    // saves the exit code
            "\tmov rax, 1\n",
            "\tmov rdi, 2\n",
            //rsi and rdx set by the caller
            "\tsyscall\n",
            
            "\tmov rax, 60\n",
            "\tmov rdi, r8\n",
            "\tsyscall\n",
        ))
    }
    
    pub fn gen_init(&self) -> String {
        format!(concat!(
            ".global _start\n",
            ".intel_syntax noprefix\n\n",
            
            ".section .data\n",
            // error messages go here
            "{OoB}",
            "\n",
            
            ".section .text\n",
            "{functions}",
            
            "_start:\n",
            // get starting heap address
            "\tmov rax, 12\n",
            "\tmov rdi, 0\n",
            "\tsyscall\n\n",
            
            // save heap boundaries
            "\tmov r12, rax\n",
            "\tlea r13, [r12 + {nb_cells}]\n\n",
            
            // allocate memory
            "\tmov rax, 12\n",
            "\tmov rdi, r13\n",
            "\tsyscall\n\n",
            
            // sets the memory pointer to the begining of the heap
            "\tmov rbx, r12\n\n",
        ),
            functions=self.predefined_functions(),
            nb_cells=self.cell_count,
            OoB=self.create_error("OoB", "The memory pointer is out of bounds."),
        )
    }
    
    pub fn gen_exit(&self, exit_code : i8) -> String {
        format!(concat!(
            "\n",
            "\tmov rax, 60\n",
            "\tmov rdi, {}\n",
            "\tsyscall\n",
        ), exit_code)
    }
    
    pub fn gen_token(&self, tok : Token) -> String {
        match tok {
            Token::MemNext(count)   => self.gen_mem_next(count),
            Token::MemPrev(count)   => self.gen_mem_prev(count),
            Token::CellInc(count)   => self.gen_cell_inc(count),
            Token::CellDec(count)   => self.gen_cell_dec(count),
            Token::Read(count)      => self.gen_cell_read(count),
            Token::Write(count)     => self.gen_cell_write(count),
            Token::LoopStart               => self.gen_loop_start(),
            Token::LoopEnd                 => self.gen_loop_end(),
        }
    }
}