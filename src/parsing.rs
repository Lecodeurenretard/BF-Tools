use crate::tokenization::Token;
use std::cmp::Ordering;
use crate::other::is_permutation;

#[derive(Clone, Debug)]
pub struct BasicInstruction {
    kind : Token,
    count : usize,
}


#[derive(Clone, Debug)]
pub struct ExtendedBasicInstruction {
    instr : BasicInstruction,
    arg : Literal
}

#[derive(Clone, Debug)]
pub struct Loop {
    inner_instructions : Vec<Instruction>,
    id : usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Literal {
    Int(u32),
    Char(char),
}

#[derive(Clone, Debug)]
pub struct ConfigFunction {
    name : String,
    args : Vec<Literal>,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    Config(ConfigFunction),
    Basic(BasicInstruction),
    ExtBasic(ExtendedBasicInstruction),
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

impl ExtendedBasicInstruction {
    pub fn new(kind : Token, count : usize, arg : Literal) -> ExtendedBasicInstruction {
        ExtendedBasicInstruction {
            instr: BasicInstruction {
                kind,
                count,
            },
            arg,
        }
    }
    
    pub fn get_kind(&self) -> &Token {
        self.instr.get_kind()
    }
    
    pub fn get_count(&self) -> usize {
        self.instr.get_count()
    }
    
    pub fn get_arg(&self) -> Literal {
        self.arg
    }
    
    pub fn override_itself(&self) -> bool {
        [
            Token::SetCell,
            Token::Goto,
        ].contains(&self.instr.kind)
    }
    
    pub fn parse(vec : &Vec<Token>, start : usize) -> Option<(ExtendedBasicInstruction, usize)> {
        if start >= vec.len() {
            unreachable!("Index too high");
        }
        if ![Token::SetCell, Token::Goto].contains(&vec[start]) {
            return None;
        }
        if start + 1 >= vec.len() {
            panic!("Expected an argument after the `{}` instruction.", vec[start])
        }
        
        if vec[start] == Token::SetCell {
            if !vec[start + 1].is_lit() {
                panic!("Expected ASCII code or character after `=` instruction.");
            }
            
            let lit = Literal::try_from(vec[start + 1].clone())
                .unwrap();
            
            if lit.into_int() > u8::MAX.into() {
                panic!("The number has to fit in a byte (max 255).");
            }
            
            return Some((
                ExtendedBasicInstruction::new(Token::SetCell, 1, lit),
                start + 2
            ));
        }
        
        if vec[start] == Token::Goto {
            if vec[start + 1] != Token::IntLit(0) {
                panic!("Expected cell number after `@` instruction.");
            }
            
            let lit = Literal::try_from(vec[start + 1].clone())
                .unwrap();
            
            return Some((
                ExtendedBasicInstruction::new(Token::Goto, 1, lit),
                start + 2
            ));
        }
        unreachable!()
    }
}

impl From<BasicInstruction> for ExtendedBasicInstruction {
    fn from(value: BasicInstruction) -> Self {
        ExtendedBasicInstruction {
            instr: value,
            arg: Literal::Int(0),
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
    #[allow(unused)]
    pub fn is_int(&self) -> bool {
        match self {
            Literal::Int(_) => true,
            _  => false
        }
    }
    
    #[allow(unused)]
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
    
    pub fn into_int(&self) -> u32 {
        match self {
            Literal::Int(v)   => *v,
            Literal::Char(v) => *v as u32,
        }
    }
    
    #[allow(unused)]
    pub fn into_char(&self) -> Option<char> {
        match self {
            Literal::Int(v)   => {
                if *v > u8::MAX.into() {
                    return None;
                }
                Some(v.to_le_bytes()[0] as char)
            },
            Literal::Char(v) => Some(*v),
        }
    }
}

impl TryFrom<Token> for Literal {
    type Error = &'static str;
    
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::IntLit(v)   => Ok(Literal::Int(v)),
            Token::CharLit(v) => Ok(Literal::Char(v)),
            _ => Err("Not a literal.")
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
    fn parse_test(s : &str, with_ebf : bool) -> Vec<Instruction> {
        Instruction::parse(Token::tokenize(String::from(s), with_ebf))
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
                    Token::SetCell   => {
                        let Some((instr, end)) = ExtendedBasicInstruction::parse(&tokens, i) else {
                            unreachable!();
                        };
                        
                        i = end - 1;    // end points to the token after the instruction
                        Instruction::ExtBasic(instr)
                    },
                    Token::Goto      => {
                        let Some((instr, end)) = ExtendedBasicInstruction::parse(&tokens, i) else {
                            unreachable!();
                        };
                        
                        i = end - 1;    // end points to the token after the instruction
                        Instruction::ExtBasic(instr)
                    },
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
    
    pub fn is_extended_basic_instruction(&self) -> bool {
        match self {
            Instruction::ExtBasic(_) => true,
            _                     => false
        }
    }
    pub fn get_extended_basic_instruction(&self) -> Option<ExtendedBasicInstruction> {
        match self {
            Instruction::ExtBasic(b) => Some(b.clone()),
            Instruction::Basic(b)            => Some(b.clone().into()),
            _                                        => None
        }
    }
    
    #[allow(unused)]
    pub fn get_extended_basic_instruction_mut(&mut self) -> Option<&mut ExtendedBasicInstruction> {
        match self {
            Instruction::ExtBasic(b) => Some(b),
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
}



pub struct Reducer {
    instructions : Vec<Instruction>,
    position : usize,
}

impl Reducer {
    #[cfg(test)]
    fn test_reduced(s : &str, with_ebf : bool) -> Vec<Instruction> {
        let mut reducer = Reducer::new(
            Instruction::parse(
                Token::tokenize_and_reduce(s, with_ebf)
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
        if let Some(curr_instr) = self.current_instruction().get_extended_basic_instruction() {
            if curr_instr.get_count() == 0 {
                pop_curr = true;
            }
        }
        
        if pop_curr {
            self.instructions.remove(self.position);
            self.reduce_trivial();
        }
        pop_curr
    }
    
    fn reduce_overriding(&mut self) -> bool {
        if self.position >= self.instructions.len() {
            return false;
        }
        
        let Some(first_instr) = self.current_instruction().get_extended_basic_instruction() else {
            return false;
        };
        if !first_instr.override_itself() { 
            return false;
        }
        
        let begin = self.position;
        self.position += 1;
        while self.position < self.instructions.len() {
            let Some(curr_instr) = self.current_instruction().get_extended_basic_instruction() else {
                break;
            };
            
            if first_instr.get_kind() != curr_instr.get_kind() {
                break;
            }
            self.position += 1;
        }
        if self.position - begin <= 1 {
            return false;
        }
        
        self.instructions.drain(begin..self.position-1);
        true
    }
    
    fn reduce_consecutives(&mut self) -> bool {
        if self.position >= self.instructions.len() {
            return false;
        }
        
        // loops and configuration functions can't be reduced
        if self.current_instruction().is_loop() || self.current_instruction().is_configuration_function() {
            return false;
        }
        if self.current_instruction().is_extended_basic_instruction() {
            return self.reduce_overriding()
        }
        if !self.current_instruction().is_basic_instruction() {
            unreachable!("An instruction is neither (extended) basic, a loop nor a configuration function.");
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
            if end_instruction.get_kind() != repeating_instruction.get_kind() {
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
    
    fn assert_eq_ext_instr(instruction1 : ExtendedBasicInstruction, instruction2 : ExtendedBasicInstruction) {
        assert_eq!(instruction1.arg, instruction2.arg);
        assert_eq!(instruction1.instr.kind, instruction2.instr.kind);
        
        assert_eq!(instruction1.instr.count, instruction2.instr.count);
        assert_eq!(instruction1.instr.count, 1);  // For now every extended instructions overrride itself
    }
    
    #[test]
    #[should_panic(expected = "Index too high")]
    fn test_ext_basic_parse_index_too_high() {
        ExtendedBasicInstruction::parse(&Token::test_tokenize("=1 @20", true), 10);
    }
    
    #[test]
    #[should_panic(expected = "Expected an argument")]
    fn test_ext_basic_parse_missing_argument() {
        ExtendedBasicInstruction::parse(&vec![Token::Goto], 0);
    }
    
    #[test]
    fn test_ext_basic_parse() {
        let tokens = vec![Token::Read, Token::SetCell, Token::IntLit(1)];
        assert!(ExtendedBasicInstruction::parse(&tokens, 0).is_none());
    }
    
    #[test]
    #[should_panic(expected = "The number has to fit in a byte")]
    fn test_ext_basic_parse_set_cell_arg_overflow() {
        let tokens: Vec<Token> = vec![Token::SetCell, Token::IntLit(2048)];
        ExtendedBasicInstruction::parse(&tokens, 0);
    }
    
    #[test]
    fn test_ext_basic_parse_set_cell() {
        let tokens: Vec<Token> = vec![Token::SetCell, Token::IntLit(42)];
        let Some((res, end)) = ExtendedBasicInstruction::parse(&tokens, 0) else {
            panic!("ExtendedBasicInstruction::parse() returned None.");
        };
        assert_eq_ext_instr(
            res,
            ExtendedBasicInstruction::new(Token::SetCell, 1, Literal::Int(42))
        );
        assert_eq!(end, 2);
        
        
        let tokens: Vec<Token> = vec![Token::SetCell, Token::CharLit('N')];
        let Some((res, end)) = ExtendedBasicInstruction::parse(&tokens, 0) else {
            panic!("ExtendedBasicInstruction::parse() returned None.");
        };
        assert_eq_ext_instr(
            res,
            ExtendedBasicInstruction::new(Token::SetCell, 1, Literal::Char('N'))
        );
        assert_eq!(end, 2);
    }
    
    #[test]
    #[should_panic(expected = "Expected cell number")]
    fn test_ext_basic_parse_goto_bad_type() {
        let tokens: Vec<Token> = vec![Token::Goto, Token::CharLit('2')];
        ExtendedBasicInstruction::parse(&tokens, 0);
    }
    
    #[test]
    fn test_ext_basic_parse_goto() {
        let tokens: Vec<Token> = vec![Token::Goto, Token::IntLit(69)];
        let Some((res, end)) = ExtendedBasicInstruction::parse(&tokens, 0) else {
            panic!("ExtendedBasicInstruction::parse() returned None.");
        };
        assert_eq_ext_instr(
            res,
            ExtendedBasicInstruction::new(Token::Goto, 1, Literal::Int(69))
        );
        assert_eq!(end, 2);
    }
    
    #[test]
    fn test_config_function_parse_good() {
        let (function, end) = ConfigFunction::parse_configuration_function(
            &Token::test_tokenize("#jojo()", true),
            0
        );
        assert_eq!(end, 3);
        assert_eq!(function.name, String::from("#jojo"));
        assert_eq!(function.args.len(), 0);
        
        let (function, end) = ConfigFunction::parse_configuration_function(
            &Token::test_tokenize("#>!(a,b,1)",true),
            0
        );
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
        Instruction::parse_test("+,]", false);
    }
    
    #[test]
    fn test_instruction_parse_loop_config() {
        Instruction::parse_test("#config()", true);
    }
    
    #[test]
    fn test_instruction_parse() {
        Instruction::parse_test("I love programming, brainfuck and punctuation. + something - someone", false);
        Instruction::parse_test("@12=34 @1=2", true);
        Instruction::parse_test("[]", false);
        Instruction::parse_test("[+-,.]", false);
        Instruction::parse_test("[+-,.[]]", false);
        Instruction::parse_test("[+-,.[=N>]@2]", true);
        Instruction::parse_test("#1()#2()[+-,.[]]", true);
        Instruction::parse_test("#1()#23()[+-,.[=N>]@2]", true);
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
        
        test(Instruction::parse_test("[][]", false));
        test(Instruction::parse_test("[[]][[]]", false));
        test(Instruction::parse_test("[[[[[[]]]]]][][[]]", false));
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
    fn test_reducer_reduce_overriding_empty() {
        assert_eq!(Reducer::new(vec![]).reduce_overriding(), false);
    }
    
    #[test]
    fn test_reducer_reduce_overriding_no_reduce() {
        let instructions = Instruction::parse_test("+-", true);
        let mut reducer = Reducer::new(instructions);
        
        assert_eq!(reducer.reduce_overriding(), false);
        assert_eq!(reducer.instructions.len(), 2);
        // not verifying any further
        
        let instructions = Instruction::parse_test(".=1=2", true);
        let mut reducer = Reducer::new(instructions);
        
        assert_eq!(reducer.reduce_overriding(), false);
        assert_eq!(reducer.instructions.len(), 3);
    }
    
    #[test]
    fn test_reducer_reduce_overriding_reduce_set_cell() {
        let instructions = Instruction::parse_test("=1=2=3", true);
        let mut reducer = Reducer::new(instructions);
        
        assert_eq!(reducer.reduce_overriding(), true);
        assert_eq!(reducer.instructions.len(), 1);
        assert_eq_ext_instr(
            reducer.instructions[0].get_extended_basic_instruction().unwrap(),
            ExtendedBasicInstruction::new(Token::SetCell, 1, Literal::Int(3))
        );
    }
    
    #[test]
    fn test_reducer_reduce_overriding_reduce_goto() {
        let instructions = Instruction::parse_test("@1@2@3", true);
        let mut reducer = Reducer::new(instructions);
        
        assert_eq!(reducer.reduce_overriding(), true);
        assert_eq!(reducer.instructions.len(), 1);
        assert_eq_ext_instr(
            reducer.instructions[0].get_extended_basic_instruction().unwrap(),
            ExtendedBasicInstruction::new(Token::Goto, 1, Literal::Int(3))
        );
    }
    
    #[test]
    fn test_reducer_reduce_overriding_reduce_mingled1() {
        let instructions = Instruction::parse_test("@1=2@3", true);
        let mut reducer = Reducer::new(instructions);
        
        assert_eq!(reducer.reduce_overriding(), false);
        assert_eq!(reducer.instructions.len(), 3);
        assert_eq_ext_instr(
            reducer.instructions[0].get_extended_basic_instruction().unwrap(),
            ExtendedBasicInstruction::new(Token::Goto, 1, Literal::Int(1))
        );
        assert_eq_ext_instr(
            reducer.instructions[1].get_extended_basic_instruction().unwrap(),
            ExtendedBasicInstruction::new(Token::SetCell, 1, Literal::Int(2))
        );
        assert_eq_ext_instr(
            reducer.instructions[2].get_extended_basic_instruction().unwrap(),
            ExtendedBasicInstruction::new(Token::Goto, 1, Literal::Int(3))
        );
    }
    
    #[test]
    fn test_reducer_reduce_overriding_reduce_mingled2() {
        let instructions = Instruction::parse_test("@1@4=2=7", true);
        let mut reducer = Reducer::new(instructions);
        
        assert_eq!(reducer.reduce_overriding(), true);
        assert_eq!(dbg!(&reducer.instructions).len(), 3);
        assert_eq_ext_instr(
            reducer.instructions[0].get_extended_basic_instruction().unwrap(),
            ExtendedBasicInstruction::new(Token::Goto, 1, Literal::Int(4))
        );
        assert_eq_ext_instr(
            reducer.instructions[1].get_extended_basic_instruction().unwrap(),
            ExtendedBasicInstruction::new(Token::SetCell, 1, Literal::Int(2))
        );
        assert_eq_ext_instr(
            reducer.instructions[2].get_extended_basic_instruction().unwrap(),
            ExtendedBasicInstruction::new(Token::SetCell, 1, Literal::Int(7))
        );
    }
    
    #[test]
    fn test_reducer_reduce_consecutives_empty() {
        assert_eq!(Reducer::new(vec![]).reduce_consecutives(), false);
    }
    
    #[test]
    fn test_reducer_reduce_consecutives_reduce1() {
        let instructions = Instruction::parse_test("----", false);
        let mut reducer = Reducer::new(instructions);
        assert_eq!(reducer.reduce_consecutives(), true);
        assert_eq!(reducer.instructions.len(), 4);
        assert_eq!(reducer.instructions[0].get_basic_instruction().unwrap().count, 4);
        assert_eq!(reducer.instructions[1].get_basic_instruction().unwrap().count, 0);
        assert_eq!(reducer.instructions[2].get_basic_instruction().unwrap().count, 0);
        assert_eq!(reducer.instructions[3].get_basic_instruction().unwrap().count, 0);
    }
    
    #[test]
    fn test_reducer_reduce_consecutives_reduce2() {
        let instructions = Instruction::parse_test("++--++", false);
        let mut reducer = Reducer::new(instructions);
        assert_eq!(reducer.reduce_consecutives(), true);
        assert_eq!(reducer.instructions.len(), 6);
        assert_eq!(reducer.instructions[0].get_basic_instruction().unwrap().count, 2);
        assert_eq!(reducer.instructions[1].get_basic_instruction().unwrap().count, 0);
        assert_eq!(reducer.instructions[2].get_basic_instruction().unwrap().count, 1);
        assert_eq!(reducer.instructions[3].get_basic_instruction().unwrap().count, 1);
        assert_eq!(reducer.instructions[4].get_basic_instruction().unwrap().count, 1);
        assert_eq!(reducer.instructions[5].get_basic_instruction().unwrap().count, 1);
    }
    
    #[test]
    fn test_reducer_reduce_consecutives_reduce3() {
        let instructions = Instruction::parse_test("@1@2", true);
        let mut reducer = Reducer::new(instructions);
        assert_eq!(reducer.reduce_consecutives(), true);
        assert_eq!(reducer.instructions.len(), 1);
    }
    
    #[test]
    fn test_reducer_reduce_consecutives_no_reduce() {
        let instructions = Instruction::parse_test("->,.", false);
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
        let instructions = Instruction::parse_test("<>", false);
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
        let reduced = Reducer::test_reduced("-----++", false);
        assert_eq!(reduced.len(), 1);
        assert_eq!(reduced[0].get_basic_instruction().unwrap().kind, Token::CellDec);
        assert_eq!(reduced[0].get_basic_instruction().unwrap().count, 3);
    }
    
    #[test]
    fn test_reducer_reduce2() {
        let reduced = Reducer::test_reduced(">>+++++[<]", false);
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