{NOTE: If Extended Brainfuck is disabled, make sure to delete comments}

+>-[>>+>+[++>-<<]-<+<+]>---.<<<<++.<<----..+++.>------.<<+.>.+++.------.>>-.<+.

Shortest known hello world from this site: {http://inversed.ru/InvMem.htm#InvMem_7}

Print new line:
>[-<<->>]<<++.

{
When begining the new line memory looks like this:
|  c0  | c1 |  c2  |
____________________
|  108 | 33 | 100  |
With the memory pointer pointing to c1

Since \n is represented by 10, we just have to do
c0 -= c2
c0 += 2
}