use std::cmp;
use std::num::{IntErrorKind, ParseIntError};
use crate::other::is_permutation;

#[derive(Clone, Hash, Debug)]
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
    IntLit(u32),
    CharLit(char),
}

#[derive(Debug)]
pub enum TokenWrappedValue<'a> {
    Str(&'a String),
    Int(u32),
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
    
    #[allow(unused)]
    pub fn get_int(&self) -> Option<u32> {
        match self {
            TokenWrappedValue::Int(i) => Some(*i),
            _ => None
        }
    }
    
    #[allow(unused)]
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
            _   => None
        }
    }
    
    fn tokenize_config_function_name(s : &Vec<char>, start : usize) -> (Token, usize) {
        if s.len() <= start {
            unreachable!("Starting index too high.");
        }
        if s[start] != '#' {
            unreachable!("The first character the string passed to Token::tokenize_config_function() is not an '#'.");
        }
        
        let mut buffer = String::from("#");
        let mut i = skip_whitespaces(s, start + 1);
        
        while i < s.len() && !"(){}".contains(s[i]) {
            buffer.push(s[i]);
            i = skip_whitespaces(s, i + 1);
        }
        
        (Token::ConfigFunc(buffer), i)
    }
    
    fn tokenize_literal_char(s : &Vec<char>, start : usize) -> (Token, usize) {
        if s.len() <= start {
            unreachable!("Starting index too high.");
        }
        return (Token::CharLit(s[start]), start + 1);
    }
    
    fn tokenize_literal_int(s : &Vec<char>, start : usize) -> (Token, usize) {
        if s.len() <= start {
            unreachable!("Starting index too high.");
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
                if err.kind() == &IntErrorKind::Empty {
                    unreachable!("The starting character is not a number.");
                }
                if err.kind() == &IntErrorKind::PosOverflow {
                    panic!("Number too big, numbers must be at most 2^64 (2^32 for 32-bits systems).");
                }
                
                // other errors are impossible by the way they are parsed.
                unreachable!();
            }
        }
        (Token::IntLit(literal), i)
    }
    
    fn tokenize_literal(s : &Vec<char>, start : usize) -> (Token, usize) {
        if s[start].is_numeric() {
            return Token::tokenize_literal_int(s, start);
        }
        Token::tokenize_literal_char(s, start)
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
            if i >= s.len() {
                panic!("Unclosed `(` in configuration function.");
            }
            if s[i] == ')' {
                continue;
            }
            if s[i] != ',' {
                panic!("Expected a `,` or a `)` but found a {} at position {i}.", s[i]);
            }
            i = skip_whitespaces(s, i + 1);
        }
        
        let mut tokens = vec![Token::ParenOpen];
        tokens.extend(literals);
        tokens.push(Token::ParenClose);
        
        Some((tokens, i + 1))
    }
    
    pub fn tokenize(s : String, allow_ebf : bool) -> Vec<Token> {
        let s : Vec<char> = s.chars().collect();
        
        let mut res : Vec<Token> = Vec::new();
        let mut commented = false;
        
        let mut i : usize = 0;
        while i < s.len() {
            let c = s[i];
            if allow_ebf{
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
            }
            
            if let Some(token) = Token::tokenize_single_char(c) {
                res.push(token);
                i += 1;
                continue;
            };
            
            if allow_ebf && c == '#' {
                let tok;
                (tok, i) = Token::tokenize_config_function_name(&s, i);
                res.push(tok);
                
                if i >= s.len() {
                    panic!("Expecting parenthesis to call a configuration function.");
                }
                if let Some(pair) = Token::tokenize_arguments_and_parenthesis(&s, i) {
                    res.extend(pair.0);
                    i = pair.1;
                    continue;
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
    pub fn test_tokenize(s : &str, with_ebf : bool) -> Vec<Token> {
        Token::tokenize(String::from(s), with_ebf)
    }
    
    #[cfg(test)]
    pub fn tokenize_and_reduce(s : &str, with_ebf : bool) -> Vec<Token> {
        let mut res = Token::test_tokenize(s, with_ebf);
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
            Token::IntLit(i)        => TokenWrappedValue::Int(*i),
            Token::CharLit(c)      => TokenWrappedValue::Char(*c),
            _ => TokenWrappedValue::NULL
        }
    }
    
    #[cfg(test)]
    fn test_unwrap_int(&self) -> u32 {
        self.get_wrapped_value().get_int().unwrap()
    }
    
    #[cfg(test)]
    fn test_unwrap_char(&self) -> char {
        self.get_wrapped_value().get_char().unwrap()
    }
    
    #[cfg(test)]
    fn test_unwrap_string(&self) -> String {
        self.get_wrapped_value().get_str().unwrap().clone()
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
    if start >= s.len() {
        return start;
    }
    
    for (i, c) in s[start..].iter().enumerate() {
        if !c.is_whitespace() && !c.is_control() {
            return i + start;
        }
    }
    s.len()
}




#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize_config_func_name_basic() {
        let string = "#myFunc".chars().collect();
        let (token, end) = Token::tokenize_config_function_name(&string, 0);
        
        assert_eq!(token, Token::ConfigFunc(String::from("")));
        assert_eq!(token.test_unwrap_string(), String::from("#myFunc"));
        assert_eq!(end, 7);
    }
    
    #[test]
    fn test_tokenize_config_func_name_empty() {
        let string = "#".chars().collect();
        let (token, end) = Token::tokenize_config_function_name(&string, 0);
        
        assert_eq!(token, Token::ConfigFunc(String::from("")));
        assert_eq!(token.test_unwrap_string(), String::from("#"));
        assert_eq!(end, 1);
    }
    
    #[test]
    fn test_tokenize_config_func_name_other_characters() {
        let string = "#&'~à$?".chars().collect();
        let (token, end) = Token::tokenize_config_function_name(&string, 0);
        
        assert_eq!(token, Token::ConfigFunc(String::from("")));
        assert_eq!(token.test_unwrap_string(), String::from("#&'~à$?"));
        assert_eq!(end, 7);
    }
    
    #[test]
    fn test_tokenize_config_func_name_instructions() {
        let string = "#,[->+<].".chars().collect();
        let (token, end) = Token::tokenize_config_function_name(&string, 0);
        
        assert_eq!(token, Token::ConfigFunc(String::from("")));
        assert_eq!(token.test_unwrap_string(), String::from("#,[->+<]."));
        assert_eq!(end, 9);
    }
    
    #[test]
    fn test_tokenize_config_func_name_whitespaces() {
        let string1 = "# \n\t\0\r".chars().collect();
        let (token1, end1) = Token::tokenize_config_function_name(&string1, 0);
        
        assert_eq!(token1, Token::ConfigFunc(String::from("")));
        assert_eq!(token1.test_unwrap_string(), String::from("#"));
        assert_eq!(end1, 6);
        
        let string2 = "# \n\t\0\r()".chars().collect();
        let (token2, end2) = Token::tokenize_config_function_name(&string2, 0);
        
        assert_eq!(token1, token2);
        assert_eq!(end1, end2);
    }
    
    #[test]
    fn test_tokenize_config_func_name_all() {
        let string = "#How The $*!& Does The World Die Twice?!()".chars().collect();
        let (token, end) = Token::tokenize_config_function_name(&string, 0);
        
        assert_eq!(token, Token::ConfigFunc(String::from("")));
        assert_eq!(token.test_unwrap_string(), String::from("#HowThe$*!&DoesTheWorldDieTwice?!"));
        assert_eq!(end, 40);
    }
    
    #[test]
    fn test_tokenize_int_literal_good() {
        let (token, end_index) = Token::tokenize_literal_int(&"0".chars().collect(), 0);
        assert_eq!(token, Token::IntLit(0));
        assert_eq!(token.test_unwrap_int(), 0);
        assert_eq!(end_index, 1);
        
        let (token, end_index) = Token::tokenize_literal_int(&"12345".chars().collect(), 0);
        assert_eq!(token, Token::IntLit(0));
        assert_eq!(token.test_unwrap_int(), 12345);
        assert_eq!(end_index, 5);
        
        let (token, end_index) = Token::tokenize_literal_int(&"abc12345".chars().collect(), 3);
        assert_eq!(token, Token::IntLit(0));
        assert_eq!(token.test_unwrap_int(), 12345);
        assert_eq!(end_index, 8);
        
        let (token, end_index) = Token::tokenize_literal_int(&"123abc456".chars().collect(), 0);
        assert_eq!(token, Token::IntLit(0));
        assert_eq!(token.test_unwrap_int(), 123);
        assert_eq!(end_index, 3);
    }
    
    #[test]
    #[should_panic(expected = "index too high")]
    fn test_tokenize_int_literal_bad_index() {
        Token::tokenize_literal_int(&"0".chars().collect(), 10);
    }
    
    #[test]
    #[should_panic(expected = "index too high")]
    fn test_tokenize_int_literal_empty() {
        Token::tokenize_literal_int(&"".chars().collect(), 0);
    }
    
    #[test]
    #[should_panic(expected = "not a number")]
    fn test_tokenize_int_literal_bad_character() {
        Token::tokenize_literal_int(&"123a456".chars().collect(), 3);
    }
    
    #[test]
    #[should_panic(expected = "too big")]
    fn test_tokenize_int_literal_overflow() {
        Token::tokenize_literal_int(&format!("{}", 2_u128.pow(65)).chars().collect(), 0);
    }
    
    #[test]
    fn test_tokenize_literal() {
        let s = "123abc1".chars().collect();
        
        assert_eq!(
            Token::tokenize_literal(&s, 0),
            Token::tokenize_literal_int(&s, 0),
        );
        assert_eq!(
            Token::tokenize_literal(&s, 2),
            Token::tokenize_literal_int(&s, 2),
        );
        assert_eq!(
            Token::tokenize_literal(&s, 3),
            Token::tokenize_literal_char(&s, 3),
        );
        assert_eq!(
            Token::tokenize_literal(&s, 5),
            Token::tokenize_literal_char(&s, 5),
        );
    }
    
    #[test]
    fn test_tokenize_args_good() {
        fn test(args : Vec<u8>) {
            let mut string = String::from("(");
            if args.is_empty() {
                unreachable!();
            }
            if args.len() == 1 {
                string = format!("({})", args[0]);
            } else {
                string += &args[0].to_string();
                for arg in &args[1..] {
                    string += &format!(", {arg}");
                }
                string += ")";
            }
            
            let Some((tokens, end_index)) = Token::tokenize_arguments_and_parenthesis(
                &string.chars().collect(),
                0
            ) else {
                unreachable!("Looks like the test is broken...")
            };
            
            let get_value = |tok : &Token| tok.test_unwrap_int();
            assert_eq!(end_index, string.len());
            assert_eq!(tokens.len(), args.len() + 2);
            assert_eq!(tokens.first().unwrap(), &Token::ParenOpen);
            assert_eq!(tokens.last().unwrap(), &Token::ParenClose);
            
            for i in 1..tokens.len()-1 {
                assert_eq!(tokens[i], Token::IntLit(0));
                assert_eq!(get_value(&tokens[i]), args[i-1].into());
            }
        }
        test(vec![0]);
        test(vec![0, 1, 2]);
        
        // testing ',)' behavior
        let Some(expected) = Token::tokenize_arguments_and_parenthesis(
            &vec!['(', '1', ',', '\n', '2', ')'],
            0
        ) else {
            unreachable!("Broken test...");
        };
        let Some(tested) = Token::tokenize_arguments_and_parenthesis(
            &vec!['(', '1', ',', '\n', '2', ',', ')'],
            0
        ) else {
            unreachable!("Broken test...");
        };
        
        assert_eq!(tested.0, expected.0);
        assert_eq!(tested.1, expected.1 + 1);   // the ',' was added
    }
    
    #[test]
    #[should_panic(expected = "too high")]
    fn test_tokenize_args_too_high() {
        Token::tokenize_arguments_and_parenthesis(
            &vec!['(', ')'],
            10
        );
    }
    
    #[test]
    #[should_panic(expected = "too high")]
    fn test_tokenize_args_unclosed() {
        Token::tokenize_arguments_and_parenthesis(
            &vec!['(', '1', ',', '\n', '2', ',', ' '],
            10
        );
    }
    
    #[test]
    fn test_tokenize_args_bad() {
        assert_eq!(
            Token::tokenize_arguments_and_parenthesis(
                &vec![')'],
                0
            ),
            None
        );
    }
    
    #[test]
    fn test_tokenize_args_empty() {
        let (tok, end) = Token::tokenize_arguments_and_parenthesis(
            &vec!['(', ')'],
            0
        ).unwrap();
        assert_eq!(tok.len(), 2);
        assert_eq!(end, 2);
        assert_eq!(tok[0], Token::ParenOpen);
        assert_eq!(tok[1], Token::ParenClose);
        return;
    }
    
    #[test]
    fn test_tokenize() {
        let tokens = Token::test_tokenize("><+-,.[]", false);
        assert_eq!(
            tokens,
            vec![Token::MemNext, Token::MemPrev, Token::CellInc, Token::CellDec, Token::Read, Token::Write, Token::BracketOpen, Token::BracketClose]
        );
        assert_eq!(tokens, Token::test_tokenize("><+-,.[]", true));
        
        assert_eq!(
            Token::test_tokenize("#Myfunc(1, a, 2, b)", false),
            vec![Token::Read, Token::Read, Token::Read]
        );
        
        let tokens = Token::test_tokenize("#Myfunc(1, a, 2, b)", true);
        assert_eq!(
            tokens,
            vec![Token::ConfigFunc(String::from("")), Token::ParenOpen, Token::IntLit(0), Token::CharLit('0'), Token::IntLit(0), Token::CharLit('0'), Token::ParenClose]
        );
        
        assert_eq!(
            tokens[0].test_unwrap_string(),
            "#Myfunc"
        );
        assert_eq!(
            tokens[2].test_unwrap_int(),
            1
        );
        assert_eq!(
            tokens[3].test_unwrap_char(),
            'a'
        );
        assert_eq!(
            tokens[4].test_unwrap_int(),
            2
        );
        assert_eq!(
            tokens[5].test_unwrap_char(),
            'b'
        );
    }
    
    #[test]
    fn test_tokenize_many_config_fn() {
        let tokens = Token::test_tokenize("#f1()#f2()>>", true);
        assert_eq!(tokens, Token::test_tokenize("#f1()#f2()>>", true));
        assert_eq!(tokens.len(), 8);
        
        assert_eq!(tokens[0], Token::ConfigFunc(String::from("")));
        assert_eq!(tokens[0].test_unwrap_string(), String::from("#f1"));
        assert_eq!(tokens[1], Token::ParenOpen);
        assert_eq!(tokens[2], Token::ParenClose);
        
        assert_eq!(tokens[3], Token::ConfigFunc(String::from("")));
        assert_eq!(tokens[3].test_unwrap_string(), String::from("#f2"));
        assert_eq!(tokens[4], Token::ParenOpen);
        assert_eq!(tokens[5], Token::ParenClose);
        
        assert_eq!(tokens[6], Token::MemNext);
        assert_eq!(tokens[7], Token::MemNext);
    }
    
    #[test]
    fn test_tokenize_comments() {
        assert_eq!(
            Token::test_tokenize("I love sentences, they let me write tests with punctuation. Even tough there is no '+' nor '>'.", false),
            vec![Token::Read, Token::Write, Token::CellInc, Token::MemNext, Token::Write]
        );
        assert_eq!(
            Token::test_tokenize("This time let's play with '{', '}' and '.'", true),
            vec![Token::Write]
        );
        assert_eq!(
            Token::test_tokenize("Wrapping up a link in a comment. {https://github.com/Lecodeurenretard/BF-Tools/tree/master}", true),
            vec![Token::Write]
        );
        
        assert_eq!(
            Token::test_tokenize(".+ {Comment >-<} -[]", true),
            vec![Token::Write, Token::CellInc, Token::CellDec, Token::BracketOpen, Token::BracketClose]
        );
        assert_eq!(
            Token::test_tokenize("{#func(a, b, c)>-<}", true),
            vec![]
        );
    }
    
    #[test]
    #[should_panic(expected = "never opened")]
    fn test_tokenize_comments_not_opened() {
        Token::test_tokenize("+,}", true);
    }
    
    #[test]
    #[should_panic(expected = "never closed")]
    fn test_tokenize_comments_not_closed() {
        Token::test_tokenize("{+,", true);
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
    
    #[test]
    fn test_skip_whitespaces() {
        fn to_vec(s : &str) -> Vec<char> {
            return  s.chars().collect();
        }
        
        assert_eq!(
            skip_whitespaces(&to_vec(""), 0),
            0
        );
        assert_eq!(
            skip_whitespaces(&to_vec("N"), 0),
            0
        );
        assert_eq!(
            skip_whitespaces(&to_vec(" N"), 0),
            1
        );
        assert_eq!(
            skip_whitespaces(&to_vec("N "), 0),
            0
        );
        assert_eq!(
            skip_whitespaces(&to_vec("\t\n\r noWhiteSpace"), 0),
            4
        );
        assert_eq!(
            skip_whitespaces(&to_vec("\t\n\r yes whitespace"), 0),
            4
        );
        
        assert_eq!(
            skip_whitespaces(&to_vec("Oi"), 10),
            10
        );
        assert_eq!(
            skip_whitespaces(&to_vec("\t\n\r noWhiteSpace"), 2),
            4
        );
        assert_eq!(
            skip_whitespaces(&to_vec("0, 1"), 1),
            1
        );
    }
}