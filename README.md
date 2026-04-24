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

## Expected behaviors
- The memory is always initialzed to all zeros when starting program.
- Every cell in the memory is an 8 bit integer.
- Incrementing from a cell at 255 gives 0 and decrementing from 0 gives 255.

## Compiler usage
### Arguments
> **`--output`** (`-o`)

The path to the compiled executable's name, defaults to the input file's name with the `.bf` extention stripped. If "./output" is provided, "./output.asm", "./output.o" and "./output" will be overwritten, depending on other arguments "./output.asm" and "./output.o" might also be deleted.

> **`--cell-count`**

The number of element the list should have. Defaults to 128.

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
++++++++++                                                                                                        >

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


<!--Neat site: https://esolangs.org/wiki/Talk:Brainfuck-->