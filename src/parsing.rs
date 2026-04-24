use crate::tokenization::Token;
use std::cmp::Ordering;
use crate::other::is_permutation;

#[derive(Clone, Copy)]
pub struct BasicInstruction {
    kind : Token,
    count : usize,
}

#[derive(Clone)]
pub struct Loop {
    inner_instructions : Vec<Instruction>,
    id : usize,
}

#[derive(Clone)]
pub enum Instruction {
    Basic(BasicInstruction),
    Loop(Loop),
}


impl BasicInstruction {
    pub fn get_kind(&self) -> Token {
        self.kind
    }
    
    pub fn get_count(&self) -> usize {
        self.count
    }
    
    pub fn are_opposites(instr1 : BasicInstruction, instr2 : BasicInstruction) -> bool {
        if instr1.kind != instr2.kind {
            return false;
        }
        
        let opposites = [
            (Token::MemNext, Token::MemPrev),
            (Token::CellInc, Token::CellDec),
        ];
        for opposite in opposites {
            if is_permutation((instr1.kind, instr2.kind), opposite) {
                return true;
            }
        }
        false
    }
}

impl TryFrom<Token> for BasicInstruction {
    type Error = String;
    
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::MemNext => Ok(BasicInstruction { kind: Token::MemNext, count: 1}),
            Token::MemPrev => Ok(BasicInstruction { kind: Token::MemPrev, count: 1}),
            Token::CellInc => Ok(BasicInstruction { kind: Token::CellInc, count: 1}),
            Token::CellDec => Ok(BasicInstruction { kind: Token::CellDec, count: 1}),
            Token::Read    => Ok(BasicInstruction { kind: Token::Read   , count: 1}),
            Token::Write   => Ok(BasicInstruction { kind: Token::Write  , count: 1}),
            tok     => Err(format!("The token '{tok:?}' isn't a basic instruction."))
        }
    }
}

impl Loop {
    pub fn get_innner_instr(&self) -> &Vec<Instruction> {
        &self.inner_instructions
    }
    
    pub fn get_id(&self) -> usize {
        self.id
    }
    
    /// Parse the vector of token as a loop, expect vec[starting_index] is Token::LoopStart.
    /// Return The resulting loop and the index of the corresponding Token::LoopEnd.
    pub fn parse_loop(vec : &Vec<Token>, starting_index : usize, starting_id : usize) -> (Loop, usize, usize) {
        let mut res_loop = Loop {
            inner_instructions: Vec::new(),
            id: starting_id,
        };
        let mut i = starting_index;
        let mut max_id = starting_id;
        
        let mut iter = vec[starting_index..].iter();
        while let Some(token) = iter.next() {
            i += 1;
            
            // insert basic instruction into vector
            if let Ok(basic_inst) = BasicInstruction::try_from(*token) {
                res_loop.inner_instructions.push(Instruction::Basic(basic_inst));
                continue;
            }
            
            // parse inner arrays
            if *token == Token::LoopStart {
                let parsed_inner_loop : Loop;
                (parsed_inner_loop, i, max_id) = Loop::parse_loop(&vec, i, max_id + 1);
                res_loop.inner_instructions.push(Instruction::Loop(parsed_inner_loop));
            }
            
            if *token == Token::LoopEnd {
                return (res_loop, i, max_id);
            }
        }
        
        panic!("Loop never closed.");
    }
}

impl Instruction {
    #[cfg(test)]
    fn parse_test(s : &str) -> Vec<Instruction> {
        Instruction::parse(Token::tokenize(s))
    }
    
    pub fn parse(vec: Vec<Token>) -> Vec<Instruction> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < vec.len() {
            i += 1;
            res.push(
                match vec[i] {
                    Token::MemNext   => Instruction::Basic(BasicInstruction::try_from(Token::MemNext).unwrap()),
                    Token::MemPrev   => Instruction::Basic(BasicInstruction::try_from(Token::MemPrev).unwrap()),
                    Token::CellInc   => Instruction::Basic(BasicInstruction::try_from(Token::CellInc).unwrap()),
                    Token::CellDec   => Instruction::Basic(BasicInstruction::try_from(Token::CellDec).unwrap()),
                    Token::Read      => Instruction::Basic(BasicInstruction::try_from(Token::Read).unwrap()),
                    Token::Write     => Instruction::Basic(BasicInstruction::try_from(Token::Write).unwrap()),
                    Token::LoopStart => {
                        let res = Loop::parse_loop(&vec, 0, 0);
                        i = res.1 + 1;  // skipping after Token::LoopEnd
                        Instruction::Loop(res.0)
                    },
                    Token::LoopEnd   => panic!("Loop never opened."),
                }
            )
        }
        res
    }
    
    pub fn is_basic_instruction(&self) -> bool {
        match self {
            Instruction::Basic(_) => true,
            _                     => false
        }
    }
    pub fn get_basic_instruction(&self) -> Option<&BasicInstruction> {
        match self {
            Instruction::Basic(b) => Some(b),
            _                                        => None
        }
    }
    pub fn get_basic_instruction_mut(&mut self) -> Option<&mut BasicInstruction> {
        match self {
            Instruction::Basic(b) => Some(b),
            _                                            => None
        }
    }
    
    pub fn is_loop(&self) -> bool {
        match self {
            Instruction::Loop(_) => true,
            _                    => false
        }
    }
    pub fn get_loop(&self) -> Option<&Loop> {
        match self {
            Instruction::Loop(l) => Some(l),
            _                           => None
        }
    }
    pub fn get_loop_mut(&mut self) -> Option<&mut Loop> {
        match self {
            Instruction::Loop(l) => Some(l),
            _                               => None
        }
    }
}



pub struct Reducer {
    instructions : Vec<Instruction>,
    position : usize,
}

impl Reducer {
    #[cfg(test)]
    fn test_reduced(s : &str) -> Vec<Instruction> {
        let mut reducer = Reducer::new(Instruction::parse(Token::tokenize(s)));
        reducer.reduce();
        reducer.instructions.clone()
    }
    
    /// returns if the current element is the one before the last one
    fn is_at_before_last(&self) -> bool {
        self.position == self.instructions.len() - 1
    }
    
    fn current_instruction(&self) -> &Instruction {
        &self.instructions[self.position]
    }
    fn current_instruction_mut(&mut self) -> &mut Instruction {
        &mut self.instructions[self.position]
    }
    
    fn reduce_trivial(&mut self) -> bool {
        let mut pop_curr = false;
        if let Some(curr_instr) = self.current_instruction().get_basic_instruction() {
            if curr_instr.count == 0 {
                pop_curr = true;
            }
        }
        
        if pop_curr {
            self.instructions.remove(self.position);
        }
        pop_curr
    }
    
    fn reduce_consecutives(&mut self) -> bool {
        // loops can't be reduced
        if self.current_instruction().get_loop().is_some() {
            return false;
        }
        
        if !self.current_instruction().is_basic_instruction() {
            unimplemented!("There's an unexpected third type in Instruction.");
        }
        self.position += 1;
        
        let start_pos = self.position;
        let mut end_pos = start_pos;
        
        // 
        while end_pos < self.instructions.len() {
            let Some(basic_instr) = self.instructions[end_pos].get_basic_instruction() else {
                break;
            };
            if basic_instr.kind != self.instructions[start_pos].get_basic_instruction().unwrap().kind {
                break;
            }
            end_pos += 1;
        }
        
        // Instructions don't repeat
        if end_pos == start_pos {
            return false;
        }
        
        // end_pos is either out of bounds or "pointing" to a different instruction
        self.instructions.drain(start_pos..(end_pos-1));
        self.instructions[start_pos-1]
            .get_basic_instruction_mut()
            .unwrap()
            .count += end_pos - start_pos;
        
        true
    }
    
    /// Remove the current instruction and the next one if they are opposite.
    fn reduce_opposites(&mut self) -> bool {
        if self.position >= self.instructions.len() - 1 {
            return false;
        }
        
        let Some(curr_instr) = self.current_instruction().get_basic_instruction() else {
            return false;
        };
        let Some(next_instr) = self.instructions[self.position + 1].get_basic_instruction() else {
            return false;
        };
        
        if !BasicInstruction::are_opposites(curr_instr.clone(), next_instr.clone()) {
            return false;
        }
        
        
        let curr_instr_count = curr_instr.count;
        let next_instr_count = next_instr.count;
        
        // Separating match cases bc multiple mutable references
        let curr_instr = self.current_instruction_mut().get_basic_instruction_mut().unwrap();
        match curr_instr_count.cmp(&next_instr_count) {
            Ordering::Less => {
                curr_instr.count = 0;
            }
            Ordering::Equal => {
                curr_instr.count = 0;
            }
            Ordering::Greater => {
                curr_instr.count -= next_instr_count;
            }
        }
        
        let next_instr= self.instructions[self.position + 1].get_basic_instruction_mut().unwrap();
        match curr_instr_count.cmp(&next_instr_count) {
            Ordering::Less => {
                next_instr.count -= curr_instr_count;
            }
            Ordering::Equal => {
                next_instr.count = 0;
            }
            Ordering::Greater => {
                next_instr.count = 0;
            }
        }
        
        let mut modified;
        modified = self.reduce_trivial();
        self.position += 1;
        modified |= self.reduce_trivial();
        self.position -= 1;
        
        modified
    }
    
    pub fn new(instructions : Vec<Instruction>) -> Reducer {
        Reducer{
            instructions,
            position: 0,
        }
    }
    
    pub fn reduce(&mut self) {
        self.position = 0;
        while self.position < self.instructions.len() {
            
            self.reduce_trivial();
            if self.reduce_consecutives() || self.reduce_opposites() {
                self.reduce_trivial();
            }
            
            self.position += 1;
        }
    }
    
    pub fn clone_instructions(&self) -> Vec<Instruction> {
        self.instructions.clone()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse() {
        Instruction::parse_test("[]");
        Instruction::parse_test("[+-,.[]]");
    }
}