Brainfuck formal syntax in  [EBNF](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form)
```BNF
program           ::= {instruction | whitespace};
instruction       ::= basic_instruction | loop;
loop =            ::= '[' {instruction | whitespace} ']';
basic_instruction ::= '<' | '>' | '+' | '-' | ',' | '.';
comment           ::= ? Any unused character (whitespaces are also comments) ?;
whitespace        ::= ? Every character c returning true with c.is_whitespace() ?;  (* Described at https://www.unicode.org/reports/tr44/#White_Space *)
```

Extended brainfuck syntax in  [EBNF](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form)
```BNF
program                      ::= {configuration_function_call}, {instruction | comment | whitespace};

configuration_function_call  ::= '#', function_name, '(', configuration_function_args, ')';
configuration_function_args  ::= {{whitespace}, literal, {whitespace}, ',', {whitespace}};
function_name                ::= ? Any character except '(', ')', '{' and '}' ?;

literal                      ::= int_literal | char_litral;
int_literal                  ::= ? Any number in [0, 2^32] without prefix (like '+') nor digit separator ?;
char_literal                 ::= ? Any non whitespace character ?;
byte                         ::= ? int_literal but in range [0, 255] ?
whitespace                   ::= ? Every character c returning true with c.is_whitespace() ?;

comment                      ::= ? Any unused character ? | '{' block_comment '}' | whitespace;
block_comment                ::= '{' {comment_content} '}';
comment_char                 ::= ? Any character except '}' ?

instruction                  ::= basic_instruction | added_instruction | loop;
loop                         ::= '[' {instruction | whitespace} ']';
basic_instruction            ::= '<' | '>' | '+' | '-' | ',' | '.';
added_instruction            ::= ('=', (char_literal | byte)) | ('@', int_literal);
```