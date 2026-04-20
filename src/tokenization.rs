use std::cmp::Ordering;

#[derive(Clone, Copy)]
pub enum Token {
    MemNext(usize),
    MemPrev(usize),
    CellInc(usize),
    CellDec(usize),
    Read(usize),
    Write(usize),
    LoopStart,
    LoopEnd,
}

pub enum TokenPairType {    // brackets
    Loop,
}

impl Token {
    fn count(&self) -> usize {
        match self {
            Token::MemNext(count) => *count,
            Token::MemPrev(count) => *count,
            Token::CellInc(count) => *count,
            Token::CellDec(count) => *count,
            Token::Read(count)    => *count,
            Token::Write(count)   => *count,
            Token::LoopStart              => 1,
            Token::LoopEnd                => 1,
        }
    }
    
    fn add(self, n : usize) -> Self {
        match self {
            Token::MemNext(count) => Token::MemNext(count + n),
            Token::MemPrev(count) => Token::MemPrev(count + n),
            Token::CellInc(count) => Token::CellInc(count + n),
            Token::CellDec(count) => Token::CellDec(count + n),
            Token::Read(count)    => Token::Read(count + n),
            Token::Write(count)   => Token::Write(count + n),
            Token::LoopStart             => Token::LoopStart,
            Token::LoopEnd               => Token::LoopEnd,
        }
    }
    fn sub(self, n : usize) -> Self {
        match self {
            Token::MemNext(count) => Token::MemNext(count - n),
            Token::MemPrev(count) => Token::MemPrev(count - n),
            Token::CellInc(count) => Token::CellInc(count - n),
            Token::CellDec(count) => Token::CellDec(count - n),
            Token::Read(count)    => Token::Read(count - n),
            Token::Write(count)   => Token::Write(count - n),
            Token::LoopStart             => Token::LoopStart,
            Token::LoopEnd               => Token::LoopEnd,
        }
    }
    
    fn to_corresponding_str(&self) -> String {
        match self {
            Token::MemNext(_) => String::from(">"),
            Token::MemPrev(_) => String::from("<"),
            Token::CellInc(_) => String::from("+"),
            Token::CellDec(_) => String::from("-"),
            Token::Read(_)    => String::from(","),
            Token::Write(_)   => String::from("."),
            Token::LoopStart  => String::from("["),
            Token::LoopEnd    => String::from("]"),
        }
    }
    
    fn tokenize_basic_instruction_and_loop(c : char) -> Option<Token> {
        match c {
            '>' => Some(Token::MemNext(1)),
            '<' => Some(Token::MemPrev(1)),
            '+' => Some(Token::CellInc(1)),
            '-' => Some(Token::CellDec(1)),
            ',' => Some(Token::Read(1)),
            '.' => Some(Token::Write(1)),
            '[' => Some(Token::LoopStart),
            ']' => Some(Token::LoopEnd),
            _   => None
        }
    }
    
    pub fn is_basic_instruction(&self) -> bool {
        match self {
            Token::MemNext(_) => true,
            Token::MemPrev(_) => true,
            Token::CellInc(_) => true,
            Token::CellDec(_) => true,
            Token::Read(_)    => true,
            Token::Write(_)   => true,
            _                 => false
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
        let mut current_token = Token::CellInc(0);  // will be removed while simplifying
        for (i, c) in s.chars().enumerate() {
            let Some(token) = Token::tokenize_basic_instruction_and_loop(c) else {
                continue;   // The character is just a comment
            };
            
            if token == current_token {
                current_token = current_token.add(1);
            } else {
                res.push(current_token);
                current_token = token;
            }
            if i + 1 == s.chars().collect::<Vec<char>>().len() {
                res.push(current_token);
                break;
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


pub fn simplify_token_list(mut token_list : Vec<Token> ) -> Vec<Token> {
    fn is_permutation(t1 : (&Token, &Token), t2 : (&Token, &Token)) -> bool {
        t1 == t2 || t1 == (t2.1, t2.0)
    }
    
    /// Reduce opposites, given that tokens[index_first] and tokens[index_first + 1] are opposites.
    /// Returns the index from which the iteration should continue
    fn reduce_opposites(tokens : &mut Vec<Token>, index_first : usize) -> usize {
        let idex = index_first;     // too long
        let curr_tok_count = tokens[idex].count();
        let next_tok_count = tokens[idex + 1].count();
        
        match curr_tok_count.cmp(&next_tok_count) {
            Ordering::Less => {
                tokens[idex] = Token::CellInc(0);
                tokens[idex + 1] = tokens[idex + 1].sub(curr_tok_count);
            }
            Ordering::Equal => {
                tokens[idex] = Token::CellInc(0);
                tokens[idex + 1] = Token::CellInc(0);
            }
            Ordering::Greater => {
                tokens[idex] = tokens[idex].sub(next_tok_count);
                tokens[idex + 1] = Token::CellInc(0);
            }
        }
        
        if idex == 0 {
            return 0;  // checks again the previous one
        }              // bc for <<-+>> which is reduced to <<>> the program needs to step back to cancel those
        idex - 1
    }
    
    let mut i  = 0;
    while i < token_list.len() {
        if i == token_list.len() - 1 {
            break;
        }
        
        if token_list[i].count() == 0{
            token_list.remove(i);
            continue;
        }
        
        
        // does not compare the count
        if is_permutation((&token_list[i], &token_list[i+1]), (&Token::CellInc(0), &Token::CellDec(0))) {
            i = reduce_opposites(&mut token_list, i);
            continue;
        }
        if is_permutation((&token_list[i], &token_list[i+1]), (&Token::MemNext(0), &Token::MemPrev(0))) {
            i = reduce_opposites(&mut token_list, i);
            continue;
        }
        
        i += 1;
    }
    token_list
}