#[derive(Clone)]
pub enum Token {
    MemNext,
    MemPrev,
    CellInc,
    CellDec,
    Read,
    Write,
    LoopStart,
    LoopEnd,
}

pub enum TokenPairType {    // brackets
    Loop,
}

impl Token {
    fn to_corresponding_str(&self) -> String {
        match self {
            Token::MemNext       => String::from(">"),
            Token::MemPrev       => String::from("<"),
            Token::CellInc      => String::from("+"),
            Token::CellDec      => String::from("-"),
            Token::Read         => String::from(","),
            Token::Write        => String::from("."),
            Token::LoopStart    => String::from("["),
            Token::LoopEnd      => String::from("]"),
        }
    }
    
    fn tokenize_basic_instruction_and_loop(c : char) -> Option<Token> {
        match c {
            '>' => Some(Token::MemNext),
            '<' => Some(Token::MemPrev),
            '+' => Some(Token::CellInc),
            '-' => Some(Token::CellDec),
            ',' => Some(Token::Read),
            '.' => Some(Token::Write),
            '[' => Some(Token::LoopStart),
            ']' => Some(Token::LoopEnd),
            _   => None
        }
    }
    
    pub fn is_basic_instruction(&self) -> bool {
        match self {
            Token::MemNext  => true,
            Token::MemPrev  => true,
            Token::CellInc => true,
            Token::CellDec => true,
            Token::Read    => true,
            Token::Write   => true,
            _              => false
        }
    }
    
    pub fn is_loop(&self) -> bool {
        match self {
            Token::LoopStart => true,
            Token::LoopEnd   => true,
            _                => false
        }
    }
    
    pub fn tokenize(s : &str) -> Vec<Token> {
        let mut res : Vec<Token> = Vec::new();
        for c in s.chars() {
            match Token::tokenize_basic_instruction_and_loop(c) {
                Some(token) => res.push(token),
                None               => ()            // this char is a comment
            }
        }
        res
    }
}

impl std::cmp::PartialEq for Token {
    fn eq(&self, other : &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
impl std::cmp::Eq for Token {}


impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.to_corresponding_str())
    }
}


pub fn simplify_token_list(mut token_list : Vec<Token>) -> Vec<Token> {
    fn is_permutation(t1 : (&Token, &Token), t2 : (&Token, &Token)) -> bool {
        t1 == t2 || t1 == (t2.1, t2.0)
    }
    
    let mut i  = 0;
    while i < token_list.len() {
        if i == token_list.len() - 1 {
            break;
        }
        
        if is_permutation((&token_list[i], &token_list[i+1]), (&Token::CellInc, &Token::CellDec)) {
            token_list.remove(i);
            token_list.remove(i);   // removes i+1
            if i > 0 {
                i -= 1;  // checks again the previous one
            }
            continue;
        }
        if is_permutation((&token_list[i], &token_list[i+1]), (&Token::MemNext, &Token::MemPrev)) {
            token_list.remove(i);
            token_list.remove(i);
            if i > 0 {
                i -= 1;  // checks again the previous one
            }
            continue;
        }
        
        i += 1;
    }
    token_list
}