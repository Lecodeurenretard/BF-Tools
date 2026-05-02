#!/bin/bash

echo "Testing C-String.bf"
cargo run --release -- double.bf -o test 1> /dev/null || exit 1

echo -n "Hello" > input
echo -n "Hello" > expected
./test < input > output
diff --brief output expected > /dev/null
if [ $? -ne 0 ]; then
	echo "Unexpected output from Reading string "Hello":" 1>&2
	cat output 1>&2
	exit 2
fi

echo -n "Brainfuck" > input
echo -n "Brainfuck" > expected
./test < input > output
diff --brief output expected > /dev/null
if [ $? -ne 0 ]; then
	echo "Unexpected output from Reading string "Hello":" 1>&2
	cat output 1>&2
	exit 2
fi

echo "Testing double.bf"
cargo run --release -- double.bf -o test 1> /dev/null || exit 1

# testing double.bf < "?" (expected "~")
echo -n "?" > input
echo -n "~" > expected
./test < input > output

diff --brief output expected > /dev/null
if [ $? -ne 0 ]; then
	echo "Unexpected output from doubling '?' (ASCII 63):" 1>&2
	cat output 1>&2
	exit 2
fi
echo "First double.bf test successful."


# testing double.bf < '"' (expected "D")
echo -n '"' > input
echo -n 'D' > expected
./test < input > output


diff --brief output expected > /dev/null
if [ $? -ne 0 ]; then
	echo "Unexpected output from doubling 'D' (ASCII 34):" 1>&2
	cat output 1>&2
	exit 2
fi
echo "Second double.bf test successful."
echo "Every tests for double.bf were successful!"
echo ""



echo "Testing hello world.bf"
cargo run --release -- "hello world.bf" -o test 1> /dev/null 2> /dev/null || exit 1 	 # Ignoring stout & stderr to not overload the terminal

echo "Hello World!" > expected
./test > output
diff --brief output expected > /dev/null
if [ $? -ne 0 ]; then
	echo "Hello World.bf does not prints "Hello World!\\n", it prints:" 1>&2
	cat output 1>&2
	exit 2
fi
echo "hello world.bf test successful."





echo "Testing hello world optimized.bf"
cargo run --release -- "hello world optimized.bf" -o test 1> /dev/null 2> /dev/null || exit 1 	 # Ignoring stout & stderr to not overload the terminal

echo -n "Hello World!" > expected
./test > output
diff --brief output expected > /dev/null
if [ $? -ne 0 ]; then
	echo "Hello World optimized.bf does not prints "Hello World!", it prints:" 1>&2
	cat output 1>&2
	exit 2
fi
echo "hello world optimized.bf test successful."
echo ""


# Useless on Github but better when running in local
rm test
rm input
rm output
rm expected
echo "All good!"
exit 0