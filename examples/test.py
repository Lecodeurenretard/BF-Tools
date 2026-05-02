import subprocess
import sys
import os

# Easier to see than just a file=sys.stderr argument
def print_err(*values: object, sep: str = " ", end: str = "\n"): 
	print(*values, sep=sep, end=end)

def compile_bf(source_file : str):
	result = subprocess.run(
		["cargo", "run", "--release", "--", source_file, "-o", "test"],
		stdout=subprocess.DEVNULL,
	)
	if result.returncode != 0:
		print_err(f"Failed to compile {source_file}")
		sys.exit(1)


def run_test(input_bytes : bytes, expected_bytes : bytes, desc : str):
	with open("input", "wb") as f:
		f.write(input_bytes)
	with open("expected", "wb") as f:
		f.write(expected_bytes)
	
	res = None
	with open("input", "rb") as stdin, open("output", "wb") as stdout:
		res = subprocess.run(["./test"], stdin=stdin, stdout=stdout)
	
	if res.returncode != 0:
		# stderr not captured
		sys.exit(2)
	
	with open("output", "rb") as f:
		output = f.read()
	
	if output != expected_bytes:
		print_err(f"Bad output from {desc} (expected {expected_bytes}):")
		print_err(output)
		sys.exit(2)


# --- C-String.bf ---
print("Testing C-String.bf")
compile_bf("C-String.bf")

run_test(b"Hello\0", b"Hello\0", 'First "C-String" test')
run_test(b"Brainfuck\0", b"Brainfuck\0", 'Second "C-String" test')


# --- double.bf ---
print("Testing double.bf")
compile_bf("double.bf")

run_test(b"?", b"~", "doubling '?' (ASCII 63)")
print("First double.bf test successful.")

run_test(b'"', b"D", "doubling '\"' (ASCII 34)")
print("Second double.bf test successful.")
print("Every tests for double.bf were successful!")
print()


# --- hello world.bf ---
print("Testing hello world.bf")
compile_bf("hello world.bf")

run_test(b"", b"Hello World!\n", 'Hello World.bf output')
print("hello world.bf test successful.")
print()


# --- hello world optimized.bf ---
print("Testing hello world optimized.bf")
compile_bf("hello world optimized.bf")

run_test(b"", b"Hello World!", 'Hello World optimized.bf output')
print("hello world optimized.bf test successful.")
print()


# Cleanup, useless on Github but better for local testing
for f in ("test", "input", "output", "expected"):
	try:
		os.remove(f)
	except FileNotFoundError:
		...

print("All good!")
sys.exit(0)