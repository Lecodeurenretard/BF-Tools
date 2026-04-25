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
        let mut commented = false;
        for c in s.chars() {
            if c == '}' {
                if !commented {
                    panic!("A comment was closed but never opened.")
                }
                commented = false;
            }
            if c == '{' { commented = true; }
            if commented { continue; }
             
            
            if let Some(token) = Token::tokenize_basic_instruction_and_loop(c) {
                res.push(token);
            };
        }
        if commented {
            panic!("A comment was opened but never closed.")
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
    
    #[test]
    fn test_tokenize_comments() {
        assert_eq!(
            Token::tokenize("I love sentences, they let me write tests with punctuation. Even tough there is no '+' nor '>' I'll take it."),
            vec![Token::Read, Token::Write, Token::CellInc, Token::MemNext, Token::Write]
        );
        assert_eq!(
            Token::tokenize("This time let's play with '{', '}' and '.'"),
            vec![Token::Write]
        );
        assert_eq!(
            Token::tokenize("Wrapping up a link in a comment: {https://github.com/Lecodeurenretard/BF-Tools/tree/master}"),
            vec![]
        );
    }
    
    #[test]
    #[should_panic(expected = "never opened")]
    fn test_tokenize_comments_not_opened() {
        Token::tokenize("+,}");
    }
    
    #[test]
    #[should_panic(expected = "never closed")]
    fn test_tokenize_comments_not_closed() {
        Token::tokenize("{+,");
    }
}