use crate::parsing::{BasicInstruction, ExtendedBasicInstruction, Instruction, Literal, Loop};
use crate::tokenization::Token;

pub struct Generator {
    cell_count : u32,
    bound_check : bool,
}

impl Generator {
    pub fn new(enable_ebound_check : bool) -> Generator {
        Generator { 
            cell_count: 128,
            bound_check: enable_ebound_check,
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
        if !self.bound_check {
            return String::new();
        }
        
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
            "\t{name}: .asciz \"\\n{msg}\\n\"\n",
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
    
    fn gen_loop_start(&self, id : usize) -> String {
        format!(concat!(
            "\tcmp byte ptr [rbx], 0\n",
            "\tjz loop{id}End\n",
            "loop{id}Start:\n"
        ), id=id)
    }
    
    fn gen_loop_end(&self, id : usize) -> String {
        format!(concat!(
            "\tcmp byte ptr [rbx], 0\n",
            "\tjnz loop{id}Start\n",
            "loop{id}End:\n"
        ), id=id)
    }
    
    fn gen_set_cell(&self, arg : Literal) -> String {
        let val : u8 = match arg.get_int() {
            Some(v) => {
                // checked for overflow while parsing
                v as u8
            },
            None => {
                if arg.get_char().is_none() {
                    unreachable!()  // unimplemented type
                }
                arg.get_char().unwrap() as u8
            }
        };
        
        format!(
            "\tmov byte ptr [rbx], {val}\n"
        )
    }
    
    fn gen_goto(&self, arg : Literal) -> String {
        let Some(dest) = arg.get_int() else {
            unreachable!();
        };
        
        format!(concat!(
            "\tmov rbx, r12\n",
            "\tadd rbx, {}\n",
        ), dest)
        + &self.check_ptr(0)
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
    
    fn gen_basic_instr(&self, instr : &BasicInstruction) -> String {
        match instr.get_kind() {
            Token::MemNext   => self.gen_mem_next(instr.get_count()),
            Token::MemPrev   => self.gen_mem_prev(instr.get_count()),
            Token::CellInc   => self.gen_cell_inc(instr.get_count()),
            Token::CellDec   => self.gen_cell_dec(instr.get_count()),
            Token::Read      => self.gen_cell_read(instr.get_count()),
            Token::Write     => self.gen_cell_write(instr.get_count()),
            _                => unreachable!()
        }
    }
    
    fn gen_ext_basic_instr(&self, instr : &ExtendedBasicInstruction) -> String {
        match instr.get_kind() {
            Token::SetCell => self.gen_set_cell(instr.get_arg()),
            Token::Goto    => self.gen_goto(instr.get_arg()),
            _              => unreachable!()
        }
    }
    
    fn gen_loop(&self, instr : &Loop) -> String {
        let mut res : String;
        
        res = self.gen_loop_start(instr.get_id());
        for inner in instr.get_inner_instr() {
            res += &self.gen_instr(inner);
        }
        res += &self.gen_loop_end(instr.get_id());
        res
    }
    
    #[allow(unused_mut)]    // may add more functions later
    pub fn gen_config_functions(&mut self, instructions : &Vec<Instruction>) -> String {
        let mut res = String::new();
        for instr in instructions {
            let Some(config_fun) = instr.get_configuration_function() else {
                break;
            };
            
            if config_fun.get_name() == "#|M|=" {
                if config_fun.get_args().len() != 1 {
                    panic!("In function {}(): Wrong number of arguments, expected 1.", config_fun.get_name());
                }
                
                let first_arg = config_fun.get_args()[0];
                self.cell_count = first_arg.get_int()
                    .expect(&format!(
                        "In function {}(): Wrong parameter type for argument 1, expected an integer.", config_fun.get_name()
                    ));
                continue;
            }
            
            panic!("Unknown configuration function: `{}()`.", config_fun.get_name());
        }
        
        res
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
    
    pub fn gen_instr(&self, instr : &Instruction) -> String {
        if let Some(basic) = instr.get_basic_instruction() {
            return self.gen_basic_instr(basic);
        }
        
        if let Some(ext_basic) = instr.get_extended_basic_instruction() {
            return self.gen_ext_basic_instr(&ext_basic);
        }
        
        if let Some(l) = instr.get_loop() {
            return self.gen_loop(l);
        }
        
        if instr.is_configuration_function() {
            // They are handled in gen_config_functions() before the call to this function.
            return String::new();
        }
        
        unreachable!("Instruction is not those categorises: loop, basic instruction or configuration function.");
    }
}


// Generated assembly cannot be verified other than compiling and checking the result