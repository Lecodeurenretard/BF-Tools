use std::cmp;
use std::num::{IntErrorKind, ParseIntError};
use crate::other::is_permutation;

#[derive(Clone, Debug)]
pub enum Token {
    MemNext,
    MemPrev,
    CellInc,
    CellDec,
    Read,
    Write,
    BracketOpen,
    BracketClose,
    ConfigFunc(String),
    ParenOpen,
    ParenClose,
    IntLit(usize),
    CharLit(char),
}

pub enum TokenWrappedValue<'a> {
    Str(&'a String),
    Int(usize),
    Char(char),
    NULL,
}

impl<'a> TokenWrappedValue<'a> {
    pub fn get_str(&self) -> Option<&String> {
        match self {
            TokenWrappedValue::Str(s) => Some(s),
            _ => None
        }
    }
    pub fn get_int(&self) -> Option<usize> {
        match self {
            TokenWrappedValue::Int(i) => Some(*i),
            _ => None
        }
    }
    pub fn get_char(&self) -> Option<char> {
        match self {
            TokenWrappedValue::Char(c) => Some(*c),
            _ => None
        }
    }
}

impl<'a> PartialEq for TokenWrappedValue<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TokenWrappedValue::Str(a), TokenWrappedValue::Str(b)) => a == b,
            (TokenWrappedValue::Int(a)  , TokenWrappedValue::Int(b))   => a == b,
            (TokenWrappedValue::Char(a)  , TokenWrappedValue::Char(b))   => a == b,
            (TokenWrappedValue::NULL, TokenWrappedValue::NULL) => true,
            _ => false
        }
    }
}

impl Token {
    fn to_corresponding_str(&self) -> String {
        match self {
            Token::MemNext                   => String::from(">"),
            Token::MemPrev                   => String::from("<"),
            Token::CellInc                   => String::from("+"),
            Token::CellDec                   => String::from("-"),
            Token::Read                      => String::from(","),
            Token::Write                     => String::from("."),
            Token::BracketOpen               => String::from("["),
            Token::BracketClose              => String::from("]"),
            Token::ConfigFunc(name) => name.clone(),
            Token::ParenOpen                 => String::from("("),
            Token::ParenClose                => String::from(")"),
            Token::IntLit(i)         => i.to_string(),
            Token::CharLit(c)         => c.to_string(),
        }
    }
    
    fn tokenize_single_char(c : char) -> Option<Token> {
        match c {
            '>' => Some(Token::MemNext),
            '<' => Some(Token::MemPrev),
            '+' => Some(Token::CellInc),
            '-' => Some(Token::CellDec),
            ',' => Some(Token::Read),
            '.' => Some(Token::Write),
            '[' => Some(Token::BracketOpen),
            ']' => Some(Token::BracketClose),
            '(' => Some(Token::ParenOpen),
            ')' => Some(Token::ParenClose),
            _   => None
        }
    }
    
    fn tokenize_config_function(s : &Vec<char>, start : usize) -> (Token, usize) {
        if s.len() <= start {
            unreachable!("Starting index too high.");
        }
        if s[start] != '#' {
            unreachable!("The first character the string passed to Token::tokenize_config_function() is not an '#'.");
        }
        
        let mut buffer = String::from("#");
        let mut i = start + 1;
        
        while i < s.len() && s[i] != '('  && s[i] != ')'  && s[i] != '{' && s[i] != '}'  {
            buffer.push(s[i]);
            i += 1;
        }
        
        (Token::ConfigFunc(buffer), i)
    }
    
    fn tokenize_literal(s : &Vec<char>, start : usize) -> (Token, usize) {
        if s.len() <= start {
            unreachable!("Starting index too high.");
        }
        
        if !s[start].is_numeric() {
            return (Token::CharLit(s[start]), start);
        }
        
        let mut i = start;
        let mut buf = String::new();
        while i < s.len() && s[i].is_numeric() {
            buf.push(s[i]);
            i += 1;
        }
        
        let literal;
        match buf.parse() {
            Ok(num) => {
                literal = num;
            },
            Err(err) => {
                let err : ParseIntError = err;  // Rust can't infer err's type
                if err.kind() == &IntErrorKind::PosOverflow {
                    panic!("Interger too big, numbers must be at most 255.");
                }
                
                // other errors can't happened with the way 
                unreachable!();
            }
        }
        (Token::IntLit(literal), i)
    }
    
    fn tokenize_arguments_and_parenthesis(s : &Vec<char>, start : usize) -> Option<(Vec<Token>, usize)> {
        if s.len() <= start {
            unreachable!("Starting index too high.");
        }
        if s[start] != '(' {
            return None;
        }
        
        let mut i = skip_whitespaces(s, start + 1);
        let mut literals = Vec::new();
        while i < s.len() {
            if s[i] == ')' {
                break;
            }
            
            let lit;
            (lit, i) = Token::tokenize_literal(s, i);
            literals.push(lit);
            
            i = skip_whitespaces(s, i);
            if s[i] != ',' {
                panic!("Expected a `,` or a `)` but found `{}`.", s[i]);
            }
            i = skip_whitespaces(s, i + 1);
        }
        
        i = skip_whitespaces(s, i);
        if i >= s.len() || s[i] != ')' {
            return None;
        }
        
        let mut tokens = vec![Token::ParenOpen];
        tokens.extend(literals);
        tokens.push(Token::ParenClose);
        
        Some((tokens, i + 1))
    }
    
    pub fn tokenize(s : String) -> Vec<Token> {
        let s : Vec<char> = s.chars().collect();
        
        let mut res : Vec<Token> = Vec::new();
        let mut commented = false;
        
        let mut i : usize = 0;
        while i < s.len() {
            let c = s[i];
            if c == '}' {
                if !commented {
                    panic!("A comment was closed but never opened.")
                }
                commented = false;
            }
            if c == '{' { commented = true; }
            if commented {
                i += 1;
                continue;
            }
            
            if let Some(token) = Token::tokenize_single_char(c) {
                res.push(token);
                i += 1;
                continue;
            };
            
            if c == '#' {
                let tok;
                (tok, i) = Token::tokenize_config_function(&s, i);
                res.push(tok);
                
                if i >= s.len() {
                    panic!("Expecting parenthesis to call a configuration function.");
                }
                if let Some(pair) = Token::tokenize_arguments_and_parenthesis(&s, i) {
                    res.extend(pair.0);
                    i = pair.1;
                }
            }
            i += 1;
        }
        if commented {
            panic!("A comment was opened but never closed.")
        }
        res
    }
    
    #[cfg(test)]
    pub fn test_tokenize(s : &str) -> Vec<Token> {
        Token::tokenize(String::from(s))
    }
    
    #[cfg(test)]
    pub fn tokenize_and_reduce(s : &str) -> Vec<Token> {
        let mut res = Token::test_tokenize(s);
        Token::reorder_opposites(&mut res);
        res
    }
    
    pub fn reorder_opposites(mut vec_tokens : &mut Vec<Token>) {
        fn reorder_opposite(v : &mut Vec<Token>, opposites : (Token, Token)) {
            let compare_inc_dec = |tok1: &Token, tok2 : &Token| {
                let contrary = (&opposites.0, &opposites.1);
                if !is_permutation((tok1, tok2), contrary) {
                    return cmp::Ordering::Equal;
                }
                if (tok1, tok2) == contrary {
                    cmp::Ordering::Greater
                } else if (tok1, tok2) == (&contrary.1, &contrary.0) {
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
    
    pub fn get_wrapped_value<'a>(&'a self) -> TokenWrappedValue<'a> {
        match self {
            Token::ConfigFunc(s) => TokenWrappedValue::Str(s),
            Token::IntLit(i)      => TokenWrappedValue::Int(*i),
            Token::CharLit(c)      => TokenWrappedValue::Char(*c),
            _ => TokenWrappedValue::NULL
        }
    }
    
    pub fn compare_wrapped_values(&self, token : &Token) -> bool {
        if self != token {
            return false;
        }
        self.get_wrapped_value() == token.get_wrapped_value()
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


/// Skips all whitespaces and returns the index of the first non whitespace character.
/// If there is nothing but whitespaces after s[start] returns s.len().
/// If the starting index is too high, returns it.
fn skip_whitespaces(s : &Vec<char>, start : usize) -> usize {
    if start <= s.len() {
        return start;
    }
    
    for (i, c) in s[start..].iter().enumerate() {
        if !c.is_whitespace() {
            return i;
        }
    }
    s.len()
}




#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize_func() {

    }
    
    #[test]
    fn test_tokenize() {
        assert_eq!(
            Token::test_tokenize("><+-,.[]"),
            vec![Token::MemNext, Token::MemPrev, Token::CellInc, Token::CellDec, Token::Read, Token::Write, Token::BracketOpen, Token::BracketClose]
        );
    }
    
    #[test]
    fn test_tokenize_comments() {
        assert_eq!(
            Token::test_tokenize("I love sentences, they let me write tests with punctuation. Even tough there is no '+' nor '>' I'll take it."),
            vec![Token::Read, Token::Write, Token::CellInc, Token::MemNext, Token::Write]
        );
        assert_eq!(
            Token::test_tokenize("This time let's play with '{', '}' and '.'"),
            vec![Token::Write]
        );
        assert_eq!(
            Token::test_tokenize("Wrapping up a link in a comment. {https://github.com/Lecodeurenretard/BF-Tools/tree/master}"),
            vec![Token::Write]
        );
        
        assert_eq!(
            Token::test_tokenize(".+ {Comment >-<} -[]"),
            vec![Token::Write, Token::CellInc, Token::CellDec, Token::BracketOpen, Token::BracketClose]
        );
    }
    
    #[test]
    #[should_panic(expected = "never opened")]
    fn test_tokenize_comments_not_opened() {
        Token::test_tokenize("+,}");
    }
    
    #[test]
    #[should_panic(expected = "never closed")]
    fn test_tokenize_comments_not_closed() {
        Token::test_tokenize("{+,");
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