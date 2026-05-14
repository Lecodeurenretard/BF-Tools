use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Parameters {
    /// The brainfuck file to parse (with the .bf extension).
    #[arg(value_parser = file_is_brainfuck)]
    input_file : String,
    
    /// The name of the executable binary file produced, by default is the name of the input file without extention.
    #[arg(short, long = "output")]
    output_file : Option<String>,
    
    /// Compile only, do not assemble nor link. The generated assembly is not deleted.
    #[arg(short='S')]
    #[arg(group = "compilation_steps")]
    compile_only : bool,
    
    /// Compile and assemble only, do not link. The generated assembly and object file are not deleted.
    #[arg(short='c')]
    #[arg(group = "compilation_steps")]
    assemble_only : bool,
    
    /// Generate debug symbols for tools such as GDB.
    #[arg(short='g', long="gen-debug")]
    debug_symbols : bool,
    
    /// Prevent the compiler to reorder instructions.
    #[arg(long="no-token-reordering")]
    disable_reordering : bool,
    
    /// Prevent the compiler to simplify the program.
    #[arg(long="no-token-reduction")]
    disable_simplification : bool,
    
    /// Prevent the compiler to generate bound checking code.
    /// Exposes the program to undefined behaviors.
    #[arg(long="no-bound-checking")]
    disable_bound_checking : bool,
    
    /// Compile the source as a regular brainfuck program.
    #[arg(long="no-ebf")]
    disable_ebf : bool,
}

impl Parameters {
    #[cfg(test)]
    fn test_new(input : &str, output : Option<&str>) -> Parameters {
        Parameters {
            input_file: String::from(input),
            output_file: match output {
                Some(out) => Some(String::from(out)),
                None => None
            },
            compile_only: false,
            assemble_only: false,
            debug_symbols: false,
            disable_reordering: false,
            disable_simplification: false,
            disable_bound_checking: false,
            disable_ebf: false,
        }
    }
    
    pub fn get_input_file(&self) -> String {
        self.input_file.clone()
    }
    
    pub fn get_output_file(&self) -> String {
        self.output_file.clone().unwrap_or(
            self.input_file.rsplit_once(".")
                .expect("The file has no '.' in it, the argument 'input_file' is not checked good enough.")
                .0
                .to_string()
        )
    }
    
    pub fn get_dbg_enabled(&self) -> bool {
        self.debug_symbols
    }
    
    pub fn get_compile_only(&self) -> bool {
        self.compile_only
    }
    
    pub fn get_assemble_only(&self) -> bool {
        self.assemble_only
    }
    
    pub fn get_disable_reordering(&self) -> bool {
        self.disable_reordering
    }
    
    pub fn get_disable_simplification(&self) -> bool {
        // can't simplify without reordering
        self.disable_simplification && self.disable_reordering
    }
    
    pub fn get_disable_bound_checking(&self) -> bool {
        self.disable_bound_checking
    }
    
    pub fn get_disable_ebf(&self) -> bool {
        self.disable_ebf
    }
}


fn file_exists(s : &str) -> Result<String, String> {
    if let Ok(metadata) = std::fs::metadata(s){ // The cases where the function returns an error are handled later
        if !metadata.is_file() {
            return Err(String::from("This path does not lead to a file."));
        }
    }
    
    match std::fs::exists(s) {
        Ok(file_exists) => {
            if file_exists {
                Ok(String::from(s))
            } else {
                Err(format!("File not found: `{s}`."))
            }
        }
        Err(err) => Err(format!("Please verify the permissions of the file `{err}`."))
    }
}

fn file_is_brainfuck(s : &str) -> Result<String, String> {
    let filename = file_exists(s)?;
    if filename.ends_with(".bf") || filename.ends_with(".ebf") {
        return Ok(String::from(s));
    }
    Err(String::from("The file is not a brainfuck file (.bf) nor an extended brainfuck file (.ebf)."))
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_output() {
        assert_eq!(Parameters::test_new("input.bf"        , None).get_output_file()      , String::from("input"));
        assert_eq!(Parameters::test_new("input.bf.bf"     , None).get_output_file()      , String::from("input.bf"));
        assert_eq!(Parameters::test_new("input.out.zip.bf", None).get_output_file()      , String::from("input.out.zip"));
        assert_eq!(Parameters::test_new("input.bf", Some("out")).get_output_file()       , String::from("out"));
        assert_eq!(Parameters::test_new("input.bf", Some("out.zip.bf")).get_output_file(), String::from("out.zip.bf"));
    }
    
    #[test]
    fn test_file_exists() {
        assert_eq!(file_exists("Cargo.toml"), Ok(String::from("Cargo.toml")));
        assert!(file_exists("adasdifsds").is_err());
        assert!(file_exists("src").is_err());
    }
    
    #[test]
    fn test_file_is_brainfuck() {
        assert!(file_is_brainfuck("src").is_err());
        assert!(file_is_brainfuck("src/main.rs").is_err());
        assert!(file_is_brainfuck("DoesNotExists.bf").is_err());
        assert!(file_is_brainfuck("DoesNotExists.ebf").is_err());
        
        assert_eq!(file_is_brainfuck("examples/hello world.bf"), Ok(String::from("examples/hello world.bf")));
        assert_eq!(file_is_brainfuck("examples/hello world readable.ebf"), Ok(String::from("examples/hello world readable.ebf")));
    }
}