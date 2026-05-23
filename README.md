# Brainfuck Tools

This repository contains multiple tools to code in Brainfuck (BF) all written in Ruse. At its final state it will have:
- A compiler
  - Compile to Unix x64 assembly
  - Can possibly also compile to Wasm
- An interpreter
- A VSCode extension

## The language
This language was created by Urban Müller in 1993 as a joke on Turing-complete languages. Its minimalistic design makes it simultaneously easy to implement and difficult to write with.

### BF
In BF, the only way to store and access data is via a pointer which points on a list of 8 bits integers called the memory pointer. When printed to the user, the values of the list are interpreted as ASCII codes.
The language consists in 8 instructions each written as a single character, every other character will considered as comment:
- `<`: Decrements the memory pointer.
- `>`: Increments the memory pointer.
- `+`: Increments the value of the current cell (the element the memory pointer points to).
- `-`: Decrements the value of the current cell.
- `,`: Lets the user input a character into the current cell.
- `.`: Prints the character of the current cell.
- `[`: If the current cell contains zero, skips every instruction until the matching `]` (which is also skipped).
- `]`: If the current cell is non zero, executes again the code of the loop.

For example, this program asks for a value, doubles it and prints it:
```
,
[->++<]
>.
```

### Extended BF (EBF)
The crates in this repository work with a modified version of BF: Extended Brainfuck. To .
#### Comments
Every character that is not any of the above is ignore as it is considered as comment.

All character in curly braces `{}` is ignored regardless if it is an instruction or not.

#### New instructions
Those instructions require an argument passed without parentheses which is either an integer (max is 2<sup>32</sup>-1) or a character, in the latter case any non whitespace character can be passed **even the ones representing instructions**.


> <b><code>=<i>int|char</i></code></b>  (also called "set cell")

Set the current cell's value to this value. If a character is passed, the cell's value will be its ASCII code. If an interger is passed, it must be at most 255.

> <b><code>@<i>int</i></code></b>  (also called "go to cell")

Set points to the cell with the given number. Overflows are handled the same as in the `>` instruction.


#### Configuration functions
They are prebuilt, compilation-evalued functions. Their purpose is to setup the program.
All of them can only be called once and before any other instruction, the order in which they are provided doesn't change their execution.

##### List
The _int_ type indicates a positive integer which is at most 2<sup>32</sup> - 1.  
The _char_ type indicates that you can just put a character.

Here is the order types are checked if multiple types can be inferred to an argument:
1. _int_
2. _char_

_if you want to input a digit as a char, you can provide instead the ASCII code of the corresponding digit (0 is 48, 1 is 49, etc...)._

> <b><code>#|M|=(<i>int</i>)</code></b>

The number of cell the program can work with. Defaults to 128.

## Expected behaviors
- The memory is always initialzed to zero when starting program.
- Every memory cell is stored as a byte.
- Incrementing from a cell at 255 gives 0 and decrementing from 0 gives 255.
- The read instruction (`,`) treats the EoF character as 0 (reading from a file is just this loop `[>,]`).

## Compiler usage
### Command-line arguments
> **`--output`** (`-o`)

The path to the compiled executable's name, defaults to the input file's name with the `.bf` extention stripped. If "./output" is provided, "./output.asm", "./output.o" and "./output" will be overwritten, depending on other arguments "./output.asm" and "./output.o" might also be deleted.

> **`-S`**  [flag]

Compile only, do not assemble nor link. The generated assembly is not deleted.

> **`-c`**  [flag]

Compile and assemble only, do not link.The generated assembly and object file are not deleted.

> **`--gen-debug`** (`-g`)  [flag]

Compile with debug symbols. Debug symbols allow tools like GDB to run the executable with debigging tools (breakpoints, source assembly, ...).

> **`--no-token-reduction`**  [flag]

Prevents the compiler to reduce tokens, see [the token reduction section](#token-reduction).

> **`--no-bound-checking`**  [flag]

Prevents the compiler to generate bounds checking in the program. If not provided, at each `>` and `<` the programs checks if the memory pointer points out of bounds. 

> **`--no-ebf`**  [flag]

Compile the file as regular brainfuck (no comments nor added instructions).

<!--- None found yet
### Undefined behaviors
-->

### Token reduction
In order to lower file sizes and improve performance, the compiler can change the program during compilation.

Token reduction consists in two ways:
1. Run-length encode tokens.
2. Cancel out instructructions that counter act eachother.
	- For example, `+` and `-` are cancelled.

The former is effective since BF programs tend to have a lot of duplicates.

Before reduction:
```
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++                                           >
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++              >
+++++++++++++++++++++-+++++++++++++++++++++-+++++++++++++++++++++++++++++++++++++-++++++++++++++++++++++++++++++++ >
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++       >
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++    >
++++++++++++++++++++++++++++++++--------------------------------++++++++++++++++++++++++++++++++                   >
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++                            >
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++    >
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ >
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++       >
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++               >
+++++++++++++++++++++++++++++++++                                                                                  >
++++++++++                                                                                                         >

<<<<<<<<<<<<<
.>.>.>.>.>.>.>.>.>.>.>.>.
```

After reduction:
```
72+  >
101+ >
108+ >
108+ >
111+ >
32+  >
87+  >
111+ >
114+ >
108+ >
100+ >
33+  >
10+

12<
.>.>.>.>.>.>.>.>.>.>.>.>.
```


## Building from source
Just execute `cargo build`.

To run tests you can execute either `cargo test` for unit tests or `python3 examples/test.py` to test the programs in [examples](examples).


<!--Neat site: https://esolangs.org/wiki/Talk:Brainfuck -->
