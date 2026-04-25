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
    pub fn get_inner_instr(&self) -> &Vec<Instruction> {
        &self.inner_instructions
    }
    
    pub fn get_id(&self) -> usize {
        self.id
    }
    
    /// Parse the vector of token as a loop, expect vec[starting_index] is Token::LoopStart.
    /// Return The resulting loop and the index of the corresponding Token::LoopEnd.
    pub fn parse(vec : &Vec<Token>, starting_index : usize, starting_id : usize) -> (Loop, usize, usize) {
        if vec.is_empty() {
            unreachable!("The vector passed to Loop::parse() is empty.")
        }
        
        if starting_index >= vec.len() {
            unreachable!("Staring index {starting_index} is too high, it must be at most {} (for the given vec).", vec.len())
        }
        
        if vec[starting_index] != Token::LoopStart {
            unreachable!("Loop::parse() called on something that is not a loop.")
        }
        
        let mut res_loop = Loop {
            inner_instructions: Vec::new(),
            id: starting_id,
        };
        let mut i = starting_index;
        let mut max_id = starting_id;
        
        while i < vec.len() - 1 {
            i += 1;
            let token = vec[i];
            
            // insert basic instruction into vector
            if let Ok(basic_inst) = BasicInstruction::try_from(token) {
                res_loop.inner_instructions.push(Instruction::Basic(basic_inst));
                continue;
            }
            
            // parse inner arrays
            if token == Token::LoopStart {
                let parsed_inner_loop : Loop;
                (parsed_inner_loop, i, max_id) = Loop::parse(&vec, i, max_id + 1);
                res_loop.inner_instructions.push(Instruction::Loop(parsed_inner_loop));
                continue;
            }
            
            // end of loop
            if token == Token::LoopEnd {
                return (res_loop, i, max_id);
            }
        }
        
        // vec is not empty
        if *vec.last().unwrap() == Token::LoopEnd {
            return (res_loop, vec.len() - 1, max_id);
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
        let mut next_loop_id = 0;
        let mut i = 0;
        while i < vec.len() {
            res.push(
                match vec[i] {
                    Token::MemNext   => Instruction::Basic(BasicInstruction{ kind:  Token::MemNext, count: 1}),
                    Token::MemPrev   => Instruction::Basic(BasicInstruction{ kind:  Token::MemPrev, count: 1}),
                    Token::CellInc   => Instruction::Basic(BasicInstruction{ kind:  Token::CellInc, count: 1}),
                    Token::CellDec   => Instruction::Basic(BasicInstruction{ kind:  Token::CellDec, count: 1}),
                    Token::Read      => Instruction::Basic(BasicInstruction{ kind:  Token::Read,    count: 1}),
                    Token::Write     => Instruction::Basic(BasicInstruction{ kind:  Token::Write,   count: 1}),
                    Token::LoopStart => {
                        let res = Loop::parse(&vec, i, next_loop_id);
                        i = res.1;
                        next_loop_id = res.2 + 1;
                        Instruction::Loop(res.0)
                    },
                    Token::LoopEnd   => panic!("Loop never opened."),
                }
            );
            i += 1;
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
    
    fn current_instruction(&self) -> &Instruction {
        &self.instructions[self.position]
    }
    fn current_instruction_mut(&mut self) -> &mut Instruction {
        &mut self.instructions[self.position]
    }
    
    fn reduce_trivial(&mut self) -> bool {
        if self.position >= self.instructions.len() {
            return false;
        }
        
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
        if self.position >= self.instructions.len() {
            return false;
        }
        
        // loops can't be reduced
        if self.current_instruction().get_loop().is_some() {
            return false;
        }
        
        if !self.current_instruction().is_basic_instruction() {
            unimplemented!("There's an unexpected third type in Instruction.");
        }
        
        let Some(repeating_instruction) = self.instructions[self.position].get_basic_instruction() else {
            return false;
        };
        
        let start_pos = self.position + 1;
        let mut end_pos = start_pos;
        
        while end_pos < self.instructions.len() {
            let Some(end_instruction) = self.instructions[end_pos].get_basic_instruction() else {
                break;
            };
            if end_instruction.kind != repeating_instruction.kind {
                break;
            }
            end_pos += 1;
        }
        
        // Instructions don't repeat
        if end_pos == start_pos {
            return false;
        }
        
        for instr in &mut self.instructions[start_pos..end_pos] {
            instr.get_basic_instruction_mut()
            .unwrap()
            .count = 0;
        }
        self.instructions[start_pos-1]
            .get_basic_instruction_mut()
            .unwrap()
            .count += end_pos - start_pos;
        
        true
    }
    
    /// Remove the current instruction and the next one if they are opposite.
    fn reduce_opposites(&mut self) -> bool {
        if self.instructions.is_empty() || self.position >= self.instructions.len() - 1 {
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
        
        true
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
            if self.reduce_consecutives() {
                self.reduce_trivial();
            }
            
            self.position += 1;
        }
        
        self.position = 0;
        while self.position < self.instructions.len() {
            
            self.reduce_trivial();
            if self.reduce_opposites() {
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
    #[should_panic(expected = "too high")]
    fn test_loop_parse_start_too_high() {
        Loop::parse(&vec![Token::LoopStart, Token::LoopEnd], 10, 0);
    }
    
    #[test]
    #[should_panic(expected = "never closed")]
    fn test_loop_parse_never_closed() {
        Loop::parse(&vec![Token::LoopStart], 0, 0);
    }
    
    #[test]
    #[should_panic(expected = "not a loop")]
    fn test_loop_parse_never_opened1() {
        Loop::parse(&vec![Token::LoopEnd], 0, 0);
    }
    
    #[test]
    #[should_panic(expected = "not a loop")]
    fn test_loop_parse_never_opened2() {
        Loop::parse(&vec![Token::Write], 0, 0);
    }
    
    #[test]
    fn test_loop_parse_empty() {
        let res = Loop::parse(&vec![Token::LoopStart, Token::LoopEnd], 0, 0);
        assert_eq!(res.0.inner_instructions.len(), 0);
        assert_eq!(res.1, 1);
        assert_eq!(res.2, 0);
    }
    
    #[test]
    fn test_loop_parse_basic() {
        let res = Loop::parse(&vec![Token::LoopStart, Token::CellDec, Token::MemNext, Token::LoopEnd], 0, 0);
        assert_eq!(res.0.inner_instructions[0].get_basic_instruction().unwrap().kind, Token::CellDec);
        assert_eq!(res.0.inner_instructions[1].get_basic_instruction().unwrap().kind, Token::MemNext);
        assert_eq!(res.1, 3);
        assert_eq!(res.2, 0);
    }
    
    #[test]
    fn test_loop_parse_inner_loop() {
        let res = Loop::parse(&vec![Token::LoopStart, Token::CellDec, Token::LoopStart, Token::MemNext, Token::LoopEnd, Token::LoopEnd], 0, 0);
        assert_eq!(res.0.inner_instructions[0].get_basic_instruction().unwrap().kind, Token::CellDec);
        assert_eq!(res.0.inner_instructions[1].get_loop().unwrap().id, 1);
        assert_eq!(res.0.inner_instructions[1].get_loop().unwrap().inner_instructions[0].get_basic_instruction().unwrap().kind, Token::MemNext);
        assert_eq!(res.1, 5);
        assert_eq!(res.2, 1);
    }
    
    #[test]
    #[should_panic(expected = "Loop never opened.")]
    fn test_instruction_parse_loop_never_opened() {
        Instruction::parse_test("+,]");
    }
    
    #[test]
    fn test_instruction_parse() {
        Instruction::parse_test("I love programming, brainfuck and punctuation. + something - someone");
        Instruction::parse_test("[]");
        Instruction::parse_test("[+-,.]");
        Instruction::parse_test("[+-,.[]]");
    }
    
    #[test]
    fn test_instruction_parse_many_loops() {
        fn test(v : Vec<Instruction>) {
            assert!(!v.is_empty());
            if v.len() == 1 {
                return;
            }
            let mut greatest_loop_id = v[0].get_loop().unwrap().get_id();
            for l in &v[1..] {
                let id = l.get_loop().unwrap().get_id();
                if !(greatest_loop_id < id) {
                    panic!("Wrong id order.")
                }
                greatest_loop_id = id;
            }
        }
        
        test(Instruction::parse_test("[][]"));
        test(Instruction::parse_test("[[]][[]]"));
        test(Instruction::parse_test("[[[[[[]]]]]][][[]]"));
    }
    #[test]
    fn test_reducer_reduce_trivial_empty() {
        assert_eq!(Reducer::new(vec![]).reduce_trivial(), false);
    }
    
    #[test]
    fn test_reducer_reduce_trivial_reduction() {
        let instructions = vec![
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 0})
        ];
        assert_eq!(Reducer::new(instructions).reduce_trivial(), true);
    }
    
    #[test]
    fn test_reducer_reduce_trivial_no_reduction() {
        let instructions = vec![
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 3})
        ];
        assert_eq!(Reducer::new(instructions).reduce_trivial(), false);
    }
    
    #[test]
    fn test_reducer_reduce_consecutives_empty() {
        assert_eq!(Reducer::new(vec![]).reduce_consecutives(), false);
    }
    
    #[test]
    fn test_reducer_reduce_consecutives_reduce() {
        let instructions = vec![
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 1}),
        ];
        let mut reducer = Reducer::new(instructions);
        assert_eq!(reducer.reduce_consecutives(), true);
        assert_eq!(reducer.instructions.len(), 4);
        assert_eq!(reducer.instructions[0].get_basic_instruction().unwrap().count, 4);
        assert_eq!(reducer.instructions[1].get_basic_instruction().unwrap().count, 0);
        assert_eq!(reducer.instructions[2].get_basic_instruction().unwrap().count, 0);
        assert_eq!(reducer.instructions[3].get_basic_instruction().unwrap().count, 0);
    }
    
    #[test]
    fn test_reducer_reduce_consecutives_no_reduce() {
        let instructions = vec![
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::MemNext, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::Read, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::Write, count: 1}),
        ];
        let mut reducer = Reducer::new(instructions);
        assert_eq!(reducer.reduce_consecutives(), false);
        assert_eq!(reducer.instructions.len(), 4);
    }
    
    #[test]
    fn test_reducer_reduce_opposites_empty() {
        assert_eq!(Reducer::new(vec![]).reduce_opposites(), false);
    }
    
    #[test]
    fn test_reducer_reduce_opposites_reduce1() {
        let instructions = vec![
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::CellInc, count: 1}),
        ];
        let mut reducer = Reducer::new(instructions);
        assert_eq!(reducer.reduce_opposites(), true);
        assert_eq!(reducer.instructions.len(), 2);
        assert_eq!(reducer.instructions[0].get_basic_instruction().unwrap().count, 0);
    }
    
    #[test]
    fn test_reducer_reduce_opposites_reduce2() {
        let instructions = vec![
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 10}),
            Instruction::Basic(BasicInstruction{kind: Token::CellInc, count: 5}),
        ];
        let mut reducer = Reducer::new(instructions);
        assert_eq!(reducer.reduce_opposites(), true);
        assert_eq!(reducer.instructions.len(), 2);
        
        assert_eq!(reducer.instructions[0].get_basic_instruction().unwrap().count, 5);
        assert_eq!(reducer.instructions[1].get_basic_instruction().unwrap().count, 0);

    }
    
    #[test]
    fn test_reducer_reduce_opposites_reduce3() {
        let instructions = vec![
            Instruction::Basic(BasicInstruction{kind: Token::CellInc, count: 10}),
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 5}),
        ];
        let mut reducer = Reducer::new(instructions);
        assert_eq!(reducer.reduce_opposites(), true);
        assert_eq!(reducer.instructions.len(), 2);
        
        assert_eq!(reducer.instructions[0].get_basic_instruction().unwrap().count, 5);
        assert_eq!(reducer.instructions[1].get_basic_instruction().unwrap().count, 0);
    }
    
    #[test]
    fn test_reducer_reduce_opposites_no_reduce() {
        let instructions = vec![
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::MemNext, count: 1}),
        ];
        let mut reducer = Reducer::new(instructions);
        assert_eq!(reducer.reduce_opposites(), false);
        assert_eq!(reducer.instructions.len(), 2);
    }
    
    #[test]
    fn test_reducer_reduce() {
        let mut reducer = Reducer::new(vec![
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::CellInc, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::CellDec, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::CellInc, count: 1}),
        ]);
        reducer.reduce();
        assert_eq!(reducer.instructions.len(), 1);
        assert_eq!(reducer.instructions[0].get_basic_instruction().unwrap().kind, Token::CellDec);
        assert_eq!(reducer.instructions[0].get_basic_instruction().unwrap().count, 3);
        //TOFIX: the parser must check again areas where it reduced
        // also mybe instruction reordering?
    }
    
}