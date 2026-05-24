import subprocess
import sys
import os

# Easier to see than just a file=sys.stderr argument
def print_err(*values: object, sep: str = " ", end: str = "\n"): 
	print(*values, sep=sep, end=end, file=sys.stderr)

def compile_bf(source_file : str, is_ebf : bool):
	source_file += ".ebf" if is_ebf else ".bf"
	
	result = subprocess.run(
		["cargo", "run", "--release", "--", source_file, "compile"],
		stdout=subprocess.DEVNULL,
	)
	
	if result.returncode != 0:
		print_err(f"Failed to compile {source_file}")
		sys.exit(1)


def run_test(input_bytes : bytes, expected_bytes : bytes, desc : str, source_file : str, is_ebf : bool):
	source_file_with_ext = source_file
	source_file_with_ext += ".ebf" if is_ebf else ".bf"
	
	# test compiled
	with open("input", "wb") as f:
		f.write(input_bytes)
	with open("expected", "wb") as f:
		f.write(expected_bytes)
	
	res = None
	with open("input", "rb") as stdin, open("output", "wb") as stdout:
		res = subprocess.run([f"./{source_file}"], stdin=stdin, stdout=stdout)
	
	if res.returncode != 0:
		# stderr not captured
		sys.exit(2)
	
	with open("output", "rb") as f:
		output_compiled = f.read()
	
	if output_compiled != expected_bytes:
		print_err(f"Bad output from {desc} (expected {expected_bytes}):")
		print_err(output_compiled)
		sys.exit(2)
	
	
	# test interpreter
	res = None
	with open("input", "rb") as stdin, open("output", "wb") as stdout:
		res = subprocess.run(
			["cargo", "run", "--release", "--", source_file_with_ext, "interpret"],
			stdin=stdin,
			stdout=stdout
		)
	
	if res.returncode != 0:
		# stderr not captured
		sys.exit(2)
	
	output_interpreted : str
	with open("output", "rb") as f:
		output_interpreted = f.read()
	
	if output_compiled != output_interpreted:
		print_err(f"Compiled and interpreted versions of {source_file_with_ext} does not yield the same output.")
		print_err(f"compiled: `{output_compiled}`")
		print_err(f"interpreted: `{output_interpreted}`")


# --- C-String.bf ---
print("Testing C-String.bf")
compile_bf("C-String", False)

run_test(b"Hello\0", b"Hello\0", 'First "C-String" test', "C-String", False)
run_test(b"Brainfuck\0", b"Brainfuck\0", 'Second "C-String" test', "C-String", False)


# --- double.bf ---
print("Testing double.bf")
compile_bf("double", False)

run_test(b"?", b"~", "doubling '?' (ASCII 63)", "double", False)
print("First double.bf test successful.")

run_test(b'"', b"D", "doubling '\"' (ASCII 34)", "double", False)
print("Second double.bf test successful.")
print("Every tests for double.bf were successful!")
print()


# --- hello world.bf ---
print("Testing hello world.bf")
compile_bf("hello world", False)

run_test(b"", b"Hello World!\n", 'Hello World.bf output', "hello world", False)
print("hello world.bf test successful.")
print()

# --- hello world readable.bf ---
print("Testing hello world readable.ebf")
compile_bf("hello world readable", True)

run_test(b"", b"Hello World!\n", 'Hello World.ebf output', "hello world readable", True)
print("hello world readable.ebf test successful.")
print()

# --- hello world optimized.bf ---
print("Testing hello world optimized.bf")
compile_bf("hello world optimized", False)

run_test(b"", b"Hello World!\n", 'Hello World optimized.bf output', "hello world optimized", False)
print("hello world optimized.bf test successful.")
print()


# Cleanup, useless on Github but better for local testing
tests = (
	"C-String",
	"double",
	"hello world",
	"hello world optimized",
	"hello world readable",
)
for f in ("input", "output", "expected", *tests):
	try:
		os.remove(f)
	except FileNotFoundError:
		...

print("All good!")
sys.exit(0)