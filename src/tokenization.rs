#[derive(Clone, Copy, Debug)]
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

impl Token {
    fn to_corresponding_str(&self) -> String {
        match self {
            Token::MemNext   => String::from(">"),
            Token::MemPrev   => String::from("<"),
            Token::CellInc   => String::from("+"),
            Token::CellDec   => String::from("-"),
            Token::Read      => String::from(","),
            Token::Write     => String::from("."),
            Token::LoopStart => String::from("["),
            Token::LoopEnd   => String::from("]"),
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
    pub fn tokenize(s : &str) -> Vec<Token> {
        let mut res : Vec<Token> = Vec::new();
        for c in s.chars() {
            if let Some(token) = Token::tokenize_basic_instruction_and_loop(c) {
                res.push(token);
            };
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




#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize() {
        assert_eq!(
            Token::tokenize("><+-,.[]"),
            vec![Token::MemNext, Token::MemPrev, Token::CellInc, Token::CellDec, Token::Read, Token::Write, Token::LoopStart, Token::LoopEnd]
        );
    }
}