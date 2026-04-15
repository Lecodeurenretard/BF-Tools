pub enum Token {
    MemInc,
    MemDec,
    CellInc,
    CellDec,
    Read,
    Write,
    LoopStart,
    LoopEnd,
}

impl Token {
    fn to_corresponding_str(&self) -> String {
        match self {
            Token::MemInc       => String::from(">"),
            Token::MemDec       => String::from("<"),
            Token::CellInc      => String::from("+"),
            Token::CellDec      => String::from("-"),
            Token::Read         => String::from(","),
            Token::Write        => String::from("."),
            Token::LoopStart    => String::from("["),
            Token::LoopEnd      => String::from("]"),
        }
    }
    
    fn tokenize_base_instr(c : char) -> Option<Token> {
        match c {
            '>' => Some(Token::MemInc),
            '<' => Some(Token::MemDec),
            '+' => Some(Token::CellInc),
            '-' => Some(Token::CellDec),
            ',' => Some(Token::Read),
            '.' => Some(Token::Write),
            '[' => Some(Token::LoopStart),
            ']' => Some(Token::LoopEnd),
            _   => None
        }
    }
    
    pub fn tokenize(s : &String) -> Vec<Token> {
        let mut res : Vec<Token> = Vec::new();
        for c in s.chars() {
            match Token::tokenize_base_instr(c) {
                Some(token) => res.push(token),
                None        => ()            // Ignore
            }
        }
        res
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.to_corresponding_str())
    }
}