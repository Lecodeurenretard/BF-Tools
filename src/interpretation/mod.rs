use crate::tokenization::Token;
use crate::parsing::{BasicInstruction, ExtendedBasicInstruction, Instruction, Literal, Loop};

use std::io::{ErrorKind, Read, Write, stdin, stdout};

pub struct Interpreter {
    bound_check : bool,
    data_ptr : u32,
    data : Vec<u8>
}

impl Interpreter {
    pub fn new(enable_bound_check : bool) -> Interpreter {
        Interpreter { 
            bound_check: enable_bound_check,
            data_ptr: 0,
            data: vec![0; 128],
        }
    }
    
    fn cell_count(&self) -> u32 {
        self.data
            .len()
            .try_into()
            .unwrap()    // The cell size is stored as an u32 in the compiler code
    }
    
    fn get_data(&self) -> u8 {
        self.data[self.data_ptr as usize]
    }
    
    fn set_data(&mut self, new_value : u8) {
        self.data[self.data_ptr as usize] = new_value;
    }
    
    fn check_ptr(&self, offset : i32) -> bool {
        if !self.bound_check {
            return false;
        }
        
        let pos = (self.data_ptr as i64) + (offset as i64);
        0 <= pos && pos < self.cell_count().into()
    }
    
    fn exec_mem_next(&mut self, count : u32) -> Result<(), String> {
        if !self.check_ptr(count as i32) {
            return Err(
                format!("Runtime error: The memory pointer is overflowing.")
            );
        }
        
        self.data_ptr += count;
        Ok(())
    }
    
    fn exec_mem_prev(&mut self, count : u32) -> Result<(), String> {
        if !self.check_ptr(-(count as i32)) {
            return Err(
                format!("Runtime error: The memory pointer is underflowing.")
            );
        }
        
        self.data_ptr -= count;
        Ok(())
    }
    
    fn exec_cell_inc(&mut self, count : u32) {
        let to_add = count % (u8::MAX as u32);
        self.set_data(
            u8::wrapping_add(self.get_data(), to_add as u8)
        );
    }

    fn exec_cell_dec(&mut self, count : u32) {
        let to_add = count % (u8::MAX as u32);
        self.set_data(
            u8::wrapping_sub(self.get_data(), to_add as u8)
        );
    }
    
    fn exec_cell_read(&mut self, count : usize) -> Result<(), String> {
        let mut buf = [0; 1];
        for _ in 0..count {
            stdin().read_exact(&mut buf)
                .or_else(|err| {
                    match err.kind() {
                        ErrorKind::UnexpectedEof => Ok(()),
                        _ => Err(err)
                    }}
                ).map_err(|err| format!("Could not read stdin because of this error: {err}"))?;
        }
        
        self.set_data(buf[0]);
        Ok(())
    }
    
    fn exec_cell_write(&self, count : usize) -> Result<(), String> {
        for _ in 0..count {
            stdout().write(&[self.get_data()])
                .map_err(|err| format!("Couldn't finish write to stdout because of this error:\n{err}"))?;
        }
        Ok(())
    }
    
    fn exec_set_cell(&mut self, arg : Literal) {
        self.set_data(
            match arg.get_int() {
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
            }
        );
    }
    
    fn exec_goto(&mut self, arg : Literal) -> Result<(), String> {
        let Some(dest) = arg.get_int() else {
            unreachable!();
        };
        
        
        let offset = (self.data_ptr as i64) - (dest as i64);
        if self.check_ptr(offset.try_into().unwrap()) {
            return Err(
                format!("Pointer out of bound after `{}` instruction.", Token::SetCell.to_string())
            );
        }
        self.data_ptr = dest;
        Ok(())
    }
    
    fn exec_basic_instr(&mut self, instr : &BasicInstruction) -> Result<(), String> {
        match instr.get_kind() {
            Token::MemNext   => self.exec_mem_next(instr.get_count() as u32),
            Token::MemPrev   => self.exec_mem_prev(instr.get_count() as u32),
            Token::CellInc   => {self.exec_cell_inc(instr.get_count() as u32); Ok(())},
            Token::CellDec   => {self.exec_cell_dec(instr.get_count() as u32); Ok(())},
            Token::Read      => self.exec_cell_read(instr.get_count()),
            Token::Write     => self.exec_cell_write(instr.get_count()),
            _                => unreachable!()
        }
    }
    
    fn exec_ext_basic_instr(&mut self, instr : &ExtendedBasicInstruction) -> Result<(), String> {
        match instr.get_kind() {
            Token::SetCell => {self.exec_set_cell(instr.get_arg()); Ok(())},
            Token::Goto    => self.exec_goto(instr.get_arg()),
            _              => unreachable!()
        }
    }
    
    fn exec_loop(&mut self, instructions : &Loop) -> Result<(), String> {
        while self.get_data() != 0 {
            for instr in instructions.get_inner_instr() {
                self.exec_instr(instr)?;
            }
        }
        Ok(())
    }
    
    pub fn exec_config_functions(&mut self, instructions : &Vec<Instruction>) -> Result<(), String> {
        for instr in instructions {
            let Some(config_fun) = instr.get_configuration_function() else {
                break;
            };
            
            if config_fun.get_name() == "#|M|=" {
                if config_fun.get_args().len() != 1 {
                    return Err(format!("In function {}(): Wrong number of arguments, expected 1.", config_fun.get_name()));
                }
                
                let first_arg = config_fun.get_args()[0];
                self.data.resize(
                    first_arg.get_int()
                        .unwrap() as usize,
                    0
                );
                continue;
            }
            
            unreachable!()
        }
        
        Ok(())
    }
    
    pub fn exec_instr(&mut self, instr : &Instruction) -> Result<(), String> {
        if let Some(basic) = instr.get_basic_instruction() {
            return self.exec_basic_instr(basic);
        }
        
        if let Some(ext_basic) = instr.get_extended_basic_instruction() {
            return self.exec_ext_basic_instr(&ext_basic);
        }
        
        if let Some(l) = instr.get_loop() {
            return self.exec_loop(l);
        }
        
        if instr.is_configuration_function() {
            // They are handled in gen_config_functions() before the call to this function.
            return Ok(());
        }
        
        unreachable!("Instruction is not those categorises: loop, basic instruction or configuration function.");
    }
}