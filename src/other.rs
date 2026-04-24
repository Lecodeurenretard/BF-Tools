use std::io::Write;
use std::process::Output;

use crate::compilation::Generator;
use crate::parsing::Instruction;

pub fn write_assembly(generator : Generator, instructions : Vec<Instruction>, output_file : &str) -> Result<(), std::io::Error> {
    let mut output = std::fs::File::create(output_file)?;
    
    output.write(
        generator.gen_init().as_bytes()
    )?;
    
    for instr in instructions {
        output.write(
            generator.gen_instr(&instr).as_bytes()
        )?;
    }
    
    output.write(
        generator.gen_exit(0).as_bytes()
    )?;
    
    Ok(())
}

pub fn check_cmd_output(cmd_output : Output, cmd_name : &str) {
    if !cmd_output.status.success() {
        let exit_status = cmd_output.status.code()
            .expect(&format!("{cmd_name} was terminated by a signal."));
        panic!(
            "{cmd_name} ended with status {exit_status}, it printed the following errors:\n{}",
            String::from_utf8(cmd_output.stderr)
                .expect(&format!("{cmd_name} failed and while decoding its output in stderr, an UTF8 error was raised.")),
        );
    }
}

pub fn is_permutation<T : std::cmp::Eq>(t1 : (T, T), t2 : (T, T)) -> bool {
    t1 == t2 || t1 == (t2.1, t2.0)
}


#[cfg(test)]
mod tests {
    use std::process::Command;
    use super::*;

    #[test]
    fn test_cmd_good() {
        let out = Command::new("echo")
            .arg("Hello")
            .output()
            .unwrap();
        check_cmd_output(out, "echo");
    }
    
    #[test]
    #[should_panic(expected = "curl ended with status")]
    fn test_cmd_bad_arg() {
        let out = Command::new("curl")
            .arg("NonExistent")
            .output()
            .unwrap();
        check_cmd_output(out, "curl");
    }
}