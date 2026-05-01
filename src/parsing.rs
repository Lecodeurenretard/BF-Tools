use crate::tokenization::Token;
use std::cmp::Ordering;
use crate::other::is_permutation;

#[derive(Clone)]
pub struct BasicInstruction {
    kind : Token,
    count : usize,
}

#[derive(Clone)]
pub struct Loop {
    inner_instructions : Vec<Instruction>,
    id : usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Literal {
    Int(u32),
    Char(char),
}

#[derive(Clone)]
pub struct ConfigFunction {
    name : String,
    args : Vec<Literal>,
}

#[derive(Clone)]
pub enum Instruction {
    Config(ConfigFunction),
    Basic(BasicInstruction),
    Loop(Loop),
}


impl BasicInstruction {
    pub fn get_kind(&self) -> &Token {
        &self.kind
    }
    
    pub fn get_count(&self) -> usize {
        self.count
    }
    
    pub fn are_opposites(instr1 : BasicInstruction, instr2 : BasicInstruction) -> bool {
        let opposites = [
            (&Token::MemNext, &Token::MemPrev),
            (&Token::CellInc, &Token::CellDec),
        ];
        for opposite in opposites {
            if is_permutation((&instr1.kind, &instr2.kind), opposite) {
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
            Token::MemNext => Ok(BasicInstruction { kind: Token::MemNext, count: 1 }),
            Token::MemPrev => Ok(BasicInstruction { kind: Token::MemPrev, count: 1 }),
            Token::CellInc => Ok(BasicInstruction { kind: Token::CellInc, count: 1 }),
            Token::CellDec => Ok(BasicInstruction { kind: Token::CellDec, count: 1 }),
            Token::Read    => Ok(BasicInstruction { kind: Token::Read   , count: 1 }),
            Token::Write   => Ok(BasicInstruction { kind: Token::Write  , count: 1 }),
            tok     => Err(format!("The token '{tok:?}' isn't a basic instruction."))
        }
    }
}

impl TryFrom<&Token> for BasicInstruction {
    type Error = String;
    
    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        BasicInstruction::try_from(value.clone())
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
        
        if vec[starting_index] != Token::BracketOpen {
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
            let token = &vec[i];
            
            // insert basic instruction into vector
            if let Ok(basic_inst) = BasicInstruction::try_from(token) {
                res_loop.inner_instructions.push(Instruction::Basic(basic_inst));
                continue;
            }
            
            // parse inner arrays
            if token == &Token::BracketOpen {
                let parsed_inner_loop : Loop;
                (parsed_inner_loop, i, max_id) = Loop::parse(&vec, i, max_id + 1);
                res_loop.inner_instructions.push(Instruction::Loop(parsed_inner_loop));
                continue;
            }
            
            // end of loop
            if token == &Token::BracketClose {
                return (res_loop, i, max_id);
            }
        }
        
        // vec is not empty
        if *vec.last().unwrap() == Token::BracketClose {
            return (res_loop, vec.len() - 1, max_id);
        }
        
        panic!("Loop never closed.");
    }
}

impl Literal {
    pub fn is_int(&self) -> bool {
        match self {
            Literal::Int(_) => true,
            _  => false
        }
    }
    
    pub fn is_char(&self) -> bool {
        match self {
            Literal::Char(_) => true,
            _  => false
        }
    }
    
    pub fn get_int(&self) -> Option<u32> {
        match self {
            Literal::Int(val) => Some(*val),
            _  => None
        }
    }
    
    pub fn get_char(&self) -> Option<char> {
        match self {
            Literal::Char(val) => Some(*val),
            _  => None
        }
    }
}

impl ConfigFunction {
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_args(&self) -> &Vec<Literal> {
        &self.args
    }

    pub fn parse_configuration_function(tokens : &Vec<Token>, start : usize) -> (ConfigFunction, usize) {
        if tokens[start] != Token::ConfigFunc(String::from("")) {
            unreachable!();
        }
        
        let name = tokens[start].get_wrapped_value()
            .get_str()
            .unwrap()
            .clone();       //bad lifetime
        
        let mut i = start + 1;
        if i >= tokens.len() {
            panic!("Expected a `(` after the name of a configuration function but arrived at end of file.");
        }
        if tokens[i] != Token::ParenOpen {
            panic!("Expected a `(` after the name of a configuration function but found `{}`.", tokens[i]);
        }
        i += 1;
        
        let mut args = Vec::new();
        while i < tokens.len() {
            args.push(
                match tokens[i] {
                    Token::IntLit(i) => Literal::Int(i),
                    Token::CharLit(c) => Literal::Char(c),
                    _ => break
                }
            );
            i += 1;
        }
        
        if i >= tokens.len() {
            panic!("Expected a `)` but arrived at end of file.");
        }
        if tokens[i] != Token::ParenClose {
            panic!("Expected a `)` but found token `{}` instead.", tokens[i]);
        }
        
        (ConfigFunction { name, args }, i + 1)
    }
}

impl Instruction {
    #[cfg(test)]
    fn parse_test(s : &str) -> Vec<Instruction> {
        Instruction::parse(Token::tokenize(String::from(s)))
    }
    
    fn parse_configuration_functions(tokens : &Vec<Token>) -> (Vec<Instruction>, usize) {
        let mut parsed_functions = std::collections::HashSet::new();
        let mut i = 0;
        let mut res = Vec::new();
        while i < tokens.len() {
            if tokens[i] != Token::ConfigFunc(String::from("")) {
                break;
            }
            
            if parsed_functions.contains(&&tokens[i]) {
                panic!("This function has already been called.")
            }
            parsed_functions.insert(&tokens[i]);
            
            let config;
            (config, i) = ConfigFunction::parse_configuration_function(tokens, i);
            
            res.push(Instruction::Config(config));
        }
        (res, i)
    }
    
    pub fn parse(tokens: Vec<Token>) -> Vec<Instruction> {
        let (mut res, mut i) = Instruction::parse_configuration_functions(&tokens);
        
        let mut next_loop_id = 0;
        while i < tokens.len() {
            res.push(
                match tokens[i] {
                    Token::MemNext   => Instruction::Basic(BasicInstruction{ kind:  Token::MemNext, count: 1 }),
                    Token::MemPrev   => Instruction::Basic(BasicInstruction{ kind:  Token::MemPrev, count: 1 }),
                    Token::CellInc   => Instruction::Basic(BasicInstruction{ kind:  Token::CellInc, count: 1 }),
                    Token::CellDec   => Instruction::Basic(BasicInstruction{ kind:  Token::CellDec, count: 1 }),
                    Token::Read      => Instruction::Basic(BasicInstruction{ kind:  Token::Read,    count: 1 }),
                    Token::Write     => Instruction::Basic(BasicInstruction{ kind:  Token::Write,   count: 1 }),
                    Token::BracketOpen => {
                        let res = Loop::parse(&tokens, i, next_loop_id);
                        i = res.1;
                        next_loop_id = res.2 + 1;
                        Instruction::Loop(res.0)
                    },
                    Token::BracketClose   => panic!("Loop never opened."),
                    Token::ParenOpen      => unreachable!("Unexpected opening parenthesis."),   // Unrelevant parenthesis are discareded at tokenizations
                    Token::ParenClose     => unreachable!("Unexpected closing parenthesis."),
                    Token::ConfigFunc(_) => panic!("Configuration functions must be at the start of the program."),
                    Token::IntLit(l) => unreachable!("Unexpected literal `{l}`."),
                    Token::CharLit(l) => unreachable!("Unexpected literal `{l}`."),
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
    
    pub fn is_configuration_function(&self) -> bool {
        match self {
            Instruction::Config(_) => true,
            _                     => false
        }
    }
    pub fn get_configuration_function(&self) -> Option<&ConfigFunction> {
        match self {
            Instruction::Config(cf) => Some(cf),
            _                                        => None
        }
    }
    pub fn get_configuration_function_mut(&mut self) -> Option<&mut ConfigFunction> {
        match self {
            Instruction::Config(cf) => Some(cf),
            _                                            => None
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
        let mut reducer = Reducer::new(
            Instruction::parse(
                Token::tokenize_and_reduce(s)
            )
        );
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
            self.reduce_trivial();
        }
        pop_curr
    }
    
    fn reduce_consecutives(&mut self) -> bool {
        if self.position >= self.instructions.len() {
            return false;
        }
        
        // loops and configuration functions can't be reduced
        if self.current_instruction().is_loop() || self.current_instruction().is_configuration_function() {
            return false;
        }
        
        if !self.current_instruction().is_basic_instruction() {
            unreachable!("An instruction is neither basic, a loop nor a configuration function.");
        }
        
        let repeating_instruction = self.instructions[self.position]
            .get_basic_instruction()
            .unwrap();  // checked above
        
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
        // Slow but works
        let mut at_least_one_change = true;
        while at_least_one_change {
            at_least_one_change = false;
            
            self.position = 0;
            while self.position < self.instructions.len() {
                
                self.reduce_trivial();
                if self.reduce_consecutives() {
                    at_least_one_change = true;
                    self.reduce_trivial();
                }
                
                self.position += 1;
            }
            
            self.position = 0;
            while self.position < self.instructions.len() {
                
                self.reduce_trivial();
                if self.reduce_opposites() {
                    at_least_one_change = true;
                    self.reduce_trivial();
                }
                
                self.position += 1;
            }
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
    fn test_config_function_parse_good() {
        let (function, end) = ConfigFunction::parse_configuration_function(&Token::test_tokenize("#jojo()"), 0);
        assert_eq!(end, 3);
        assert_eq!(function.name, String::from("#jojo"));
        assert_eq!(function.args.len(), 0);
        
        let (function, end) = ConfigFunction::parse_configuration_function(&Token::test_tokenize("#>!(a,b,1)"), 0);
        assert_eq!(end, 6);
        assert_eq!(function.name, String::from("#>!"));
        assert_eq!(function.args.len(), 3);
        assert_eq!(function.args[0], Literal::Char('a'));
        assert_eq!(function.args[1], Literal::Char('b'));
        assert_eq!(function.args[2], Literal::Int(1));
    }
    
    #[test]
    #[should_panic(expected = "Expected a `(`")]
    fn test_config_function_parse_no_opening_paren1() {
        ConfigFunction::parse_configuration_function(
            &vec![Token::ConfigFunc(String::from("#fn"))],
            0
        );
    }
    
    #[test]
    #[should_panic(expected = "Expected a `(`")]
    fn test_config_function_parse_no_opening_paren2() {
        ConfigFunction::parse_configuration_function(
            &vec![Token::ConfigFunc(String::from("#fn")), Token::ParenClose],
            0
        );
    }
    
    #[test]
    #[should_panic(expected = "Expected a `)`")]
    fn test_config_function_parse_no_closing_paren() {
        ConfigFunction::parse_configuration_function(
            &vec![Token::ConfigFunc(String::from("#fn")), Token::ParenOpen],
            0
        );
    }
    
    #[test]
    #[should_panic(expected = "too high")]
    fn test_loop_parse_start_too_high() {
        Loop::parse(&vec![Token::BracketOpen, Token::BracketClose], 10, 0);
    }
    
    #[test]
    #[should_panic(expected = "never closed")]
    fn test_loop_parse_never_closed() {
        Loop::parse(&vec![Token::BracketOpen], 0, 0);
    }
    
    #[test]
    #[should_panic(expected = "not a loop")]
    fn test_loop_parse_never_opened1() {
        Loop::parse(&vec![Token::BracketClose], 0, 0);
    }
    
    #[test]
    #[should_panic(expected = "not a loop")]
    fn test_loop_parse_never_opened2() {
        Loop::parse(&vec![Token::Write], 0, 0);
    }
    
    #[test]
    fn test_loop_parse_empty() {
        let res = Loop::parse(&vec![Token::BracketOpen, Token::BracketClose], 0, 0);
        assert_eq!(res.0.inner_instructions.len(), 0);
        assert_eq!(res.1, 1);
        assert_eq!(res.2, 0);
    }
    
    #[test]
    fn test_loop_parse_basic() {
        let res = Loop::parse(&vec![Token::BracketOpen, Token::CellDec, Token::MemNext, Token::BracketClose], 0, 0);
        assert_eq!(res.0.inner_instructions[0].get_basic_instruction().unwrap().kind, Token::CellDec);
        assert_eq!(res.0.inner_instructions[1].get_basic_instruction().unwrap().kind, Token::MemNext);
        assert_eq!(res.1, 3);
        assert_eq!(res.2, 0);
    }
    
    #[test]
    fn test_loop_parse_inner_loop() {
        let res = Loop::parse(&vec![Token::BracketOpen, Token::CellDec, Token::BracketOpen, Token::MemNext, Token::BracketClose, Token::BracketClose], 0, 0);
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
    fn test_instruction_parse_loop_config() {
        Instruction::parse_test("#config()");
    }
    
    #[test]
    fn test_instruction_parse() {
        Instruction::parse_test("I love programming, brainfuck and punctuation. + something - someone");
        Instruction::parse_test("[]");
        Instruction::parse_test("[+-,.]");
        Instruction::parse_test("[+-,.[]]");
        Instruction::parse_test("#1()#2()[+-,.[]]");
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
        //test(Instruction::parse_test("[[]][[]]"));
        //test(Instruction::parse_test("[[[[[[]]]]]][][[]]"));
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
        let mut reducer = Reducer::new(instructions);
        assert_eq!(reducer.reduce_trivial(), true);
        assert_eq!(reducer.instructions.len(), 0);
        
        
        let instructions = vec![
            Instruction::Basic(BasicInstruction{kind: Token::MemNext, count: 0}),
            Instruction::Basic(BasicInstruction{kind: Token::CellInc, count: 0}),
            Instruction::Basic(BasicInstruction{kind: Token::Write, count: 0}),
        ];
        reducer = Reducer::new(instructions);
        assert_eq!(reducer.reduce_trivial(), true);
        assert_eq!(reducer.instructions.len(), 0);
        
        let instructions = vec![
            Instruction::Basic(BasicInstruction{kind: Token::MemNext, count: 0}),
            Instruction::Basic(BasicInstruction{kind: Token::CellInc, count: 1}),
            Instruction::Basic(BasicInstruction{kind: Token::Write, count: 0}),
        ];
        reducer = Reducer::new(instructions);
        assert_eq!(reducer.reduce_trivial(), true);
        assert_eq!(reducer.instructions.len(), 2);
        assert_eq!(reducer.instructions[0].get_basic_instruction().unwrap().get_kind(), &Token::CellInc);
        assert_eq!(reducer.instructions[0].get_basic_instruction().unwrap().get_count(), 1);
        assert_eq!(reducer.instructions[1].get_basic_instruction().unwrap().get_kind(), &Token::Write);
        assert_eq!(reducer.instructions[1].get_basic_instruction().unwrap().get_count(), 0);
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
        let instructions = Instruction::parse_test("----");
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
        let instructions = Instruction::parse_test("->,.");
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
        let instructions = Instruction::parse_test("<>");
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
    fn test_reducer_reduce1() {
        let reduced = Reducer::test_reduced("-----++");
        assert_eq!(reduced.len(), 1);
        assert_eq!(reduced[0].get_basic_instruction().unwrap().kind, Token::CellDec);
        assert_eq!(reduced[0].get_basic_instruction().unwrap().count, 3);
    }
    
    #[test]
    fn test_reducer_reduce2() {
        let reduced = Reducer::test_reduced(">>+++++[<]");
        assert_eq!(reduced.len(), 3);
        
        fn get_basic(instr : &Instruction) -> &BasicInstruction {
            instr.get_basic_instruction().unwrap()
        }
        fn get_loop_instr(instr : &Instruction) -> &Vec<Instruction> {
            &instr.get_loop().unwrap().inner_instructions
        }
        
        assert_eq!(get_basic(&reduced[0]).kind, Token::MemNext);
        assert_eq!(get_basic(&reduced[0]).count, 2);
        assert_eq!(get_basic(&reduced[1]).kind, Token::CellInc);
        assert_eq!(get_basic(&reduced[1]).count, 5);
        assert_eq!(get_basic(&get_loop_instr(&reduced[2])[0]).kind, Token::MemPrev);
        assert_eq!(get_basic(&get_loop_instr(&reduced[2])[0]).count, 1);
    }
}