# Generated assembly
## Special registers
- `r12`: A pointer to the beginning of the allocated memory.
- `r13`: A pointer to the end of the allocated memory.
- `rbx`: A pointer to the current cell.

## Functions
> `runtime_error`

Prints an error and quits.

Arguments:
- `rax`: The exit code, by convention should be non zero.
- `rsi`: A pointer to the begining of the error message.
- `rdx`: The length of the message to print.