Brainfuck formal syntax in  [EBNF](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form)
```BNF
program           ::= {instruction | whitespace}
instruction       ::= basic_instruction | loop
loop =            ::= '[' {instruction | whitespace} ']'
basic_instruction ::= '<' | '>' | '+' | '-' | ',' | '.'
comment           ::= ? Any unused character (whitespaces are also comments) ?
whitespace        ::= ? Every character c returning true with c.is_whitespace() ? (* Described at https://www.unicode.org/reports/tr44/#White_Space *)
```

Extended brainfuck syntax in  [EBNF](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form)
```BNF
program                      ::= {configuration_function_call}, {instruction | comment | whitespace}

configuration_function_call ::= '#', function_name
function_name                ::= ? Any character except '(', ')', '{' and '}' ?
comment                      ::= ? Any unused character ? | '{' block_comment '}' | whitespace
block_comment        ::= '{' {? Any character except '}' ?} '}'


instruction                  ::= basic_instruction | loop
loop =                       ::= '[' {instruction | whitespace} ']'
basic_instruction            ::= '<' | '>' | '+' | '-' | ',' | '.'
whitespace        ::= ? Every character c returning true with c.is_whitespace() ?
```