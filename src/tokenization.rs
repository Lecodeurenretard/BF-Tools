use std::cmp;

use crate::other::is_permutation;

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
    
    #[cfg(test)]
    pub fn tokenize_and_reduce(s : &str) -> Vec<Token> {
        let mut res = Token::tokenize(s);
        Token::reorder_opposites(&mut res);
        res
    }
    
    pub fn reorder_opposites(mut vec_tokens : &mut Vec<Token>) {
        fn reorder_opposite(v : &mut Vec<Token>, opposites : (Token, Token)) {
            let compare_inc_dec = |tok1: &Token, tok2 : &Token| {
                if !is_permutation((*tok1, *tok2), opposites) {
                    return cmp::Ordering::Equal;
                }
                if (*tok1, *tok2) == opposites {
                    cmp::Ordering::Greater
                } else if (*tok1, *tok2) == (opposites.1, opposites.0) {
                    cmp::Ordering::Less
                } else if tok1 == tok2 {
                    cmp::Ordering::Equal
                } else {
                    unreachable!()
                }
            };
            let mut range_start = 0;
            let mut in_range = false;
            let mut ranges = Vec::new();
            // Find subarrays where tokens are all opposite instrucions.
            for (i, tok) in v.iter().enumerate() {
                if (*tok != opposites.0 && *tok != opposites.1) || i == v.len() - 1 {
                    if in_range {
                        if i - range_start > 1{ 
                            ranges.push(range_start..i);
                        }
                        in_range = false;
                    }
                    continue;
                }
                
                if !in_range {
                    range_start = i;
                    in_range = true;
                }
            }
            
            // reorders sub arrays
            for r in ranges {
                v[r].sort_by(compare_inc_dec);
            }
        }
        reorder_opposite(&mut vec_tokens, (Token::CellDec, Token::CellInc));
        reorder_opposite(&mut vec_tokens, (Token::MemPrev, Token::MemNext));
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
            Token::tokenize("Wrapping up a link in a comment. {https://github.com/Lecodeurenretard/BF-Tools/tree/master}"),
            vec![Token::Write]
        );
        
        assert_eq!(
            Token::tokenize(".+ {Comment >-<} -[]"),
            vec![Token::Write, Token::CellInc, Token::CellDec, Token::LoopStart, Token::LoopEnd]
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
    
    #[test]
    fn test_reorder_inc() {
        let mut tokens = vec![
            Token::CellDec,
            Token::CellInc,
            Token::CellInc,
            Token::CellDec,
        ];
        Token::reorder_opposites(&mut tokens);
        
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::CellInc);
        assert_eq!(tokens[1], Token::CellInc);
        assert_eq!(tokens[2], Token::CellDec);
        assert_eq!(tokens[3], Token::CellDec);
        
    }
    
    #[test]
    fn test_reorder_next() {
        let mut tokens = vec![
            Token::MemNext,
            Token::MemPrev,
            Token::MemNext,
            Token::MemPrev,
        ];
        Token::reorder_opposites(&mut tokens);
        
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::MemNext);
        assert_eq!(tokens[1], Token::MemNext);
        assert_eq!(tokens[2], Token::MemPrev);
        assert_eq!(tokens[3], Token::MemPrev);
    }
    
    #[test]
    fn test_reorder_mingled1() {
        let mut tokens = vec![
            Token::MemNext,
            Token::MemPrev,
            Token::CellDec,
            Token::CellInc,
        ];
        Token::reorder_opposites(&mut tokens);
        
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::MemNext);
        assert_eq!(tokens[1], Token::MemPrev);
        assert_eq!(tokens[2], Token::CellDec);
        assert_eq!(tokens[3], Token::CellInc);
    }
    
    #[test]
    fn test_reorder_mingled2() {
        let mut tokens = vec![
            Token::MemNext,
            Token::CellInc,
            Token::MemPrev,
            Token::CellDec,
        ];
        Token::reorder_opposites(&mut tokens);
        
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::MemNext);
        assert_eq!(tokens[1], Token::CellInc);
        assert_eq!(tokens[2], Token::MemPrev);
        assert_eq!(tokens[3], Token::CellDec);
    }
    
    #[test]
    fn test_reorder_wall1() {
        let mut tokens = vec![
            Token::MemNext,
            Token::CellInc,
            Token::Read,
            Token::MemPrev,
            Token::CellDec,
        ];
        Token::reorder_opposites(&mut tokens);
        
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::MemNext);
        assert_eq!(tokens[1], Token::CellInc);
        assert_eq!(tokens[2], Token::Read);
        assert_eq!(tokens[3], Token::MemPrev);
        assert_eq!(tokens[4], Token::CellDec);
    }
    
    #[test]
    fn test_reorder_wall2() {
        let mut tokens = vec![
            Token::CellInc,
            Token::CellDec,
            Token::CellInc,
            Token::Read,
            Token::MemPrev,
            Token::MemNext,
            Token::MemPrev,
        ];
        Token::reorder_opposites(&mut tokens);
        
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0], Token::CellInc);
        assert_eq!(tokens[1], Token::CellInc);
        assert_eq!(tokens[2], Token::CellDec);
        assert_eq!(tokens[3], Token::Read);
        assert_eq!(tokens[4], Token::MemNext);
        assert_eq!(tokens[5], Token::MemPrev);
        assert_eq!(tokens[6], Token::MemPrev);
    }
}