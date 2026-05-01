Brainfuck formal syntax in  [EBNF](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form)
```BNF
program           ::= {instruction | ex}
instruction       ::= basic_instruction | loop
loop =            ::= '[' {instruction} ']'
basic_instruction ::= '<' | '>' | '+' | '-' | ',' | '.'
comment           ::= ? Any unused character ?
```

Extended brainfuck syntax in  [EBNF](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form)
```BNF
program                      ::= {configuration_function_call}, {instruction | comment}

configuration_function_call ::= '#', function_name
function_name                ::= ? Any character except '(', ')', '{' and '}' ?
comment                      ::= ? Any unused character ? | '{' block_comment_content '}'
block_comment_content        ::= ? Any character except '}' ?


instruction                  ::= basic_instruction | loop
loop =                       ::= '[' {instruction} ']'
basic_instruction            ::= '<' | '>' | '+' | '-' | ',' | '.'
```