use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Parameters {
    /// The brainfuck file to parse (with the .bf extension).
    #[arg(value_parser = file_is_brainfuck)]
    input_file : String,
    
    #[clap(subcommand)]
    subcommand : Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compiles brainfuck to assembly then to an executable with as and ld.
    Compile {
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
        
        /// Prevent the compiler from reordering instructions while parsing.
        #[arg(long="no-token-reordering")]
        disable_reordering : bool,
        
        /// Prevent the compiler from simplifing the program.
        #[arg(long="no-token-reduction")]
        disable_simplification : bool,
        
        /// Prevent the compiler from generating bound checking code.
        /// Exposes the program to undefined behaviors.
        #[arg(long="no-bound-checking")]
        disable_bound_checking : bool,
        
        /// Compile the source without any added functionnalities other that brainfuck doesn't provide.
        #[arg(long="no-ebf")]
        disable_ebf : bool,
    },
    
    /// Execute in real-time the brainfuck program.
    Interpret {
        /// Prevent the interpreter from reordering instructions while parsing.
        #[arg(long="no-token-reordering")]
        disable_reordering : bool,
        
        /// Prevent the interpreter from not executing no-ops.
        #[arg(long="no-token-reduction")]
        disable_simplification : bool,
        
        /// Interpret the source without any added functionnalities other that brainfuck doesn't provide.
        #[arg(long="no-ebf")]
        disable_ebf : bool,
    }
}

impl Command {
    #[cfg(test)]
    pub fn test_new(subcmd : char, output : Option<&str>) -> Command {
        match subcmd {
            'c' => Command::Compile {
                output_file: output.map(|s| String::from(s)),
                compile_only: false,
                assemble_only: false,
                debug_symbols: false,
                disable_reordering: false,
                disable_simplification: false,
                disable_bound_checking: false,
                disable_ebf: false
            },
            
            'i' => Command::Interpret {
                disable_reordering: false,
                disable_simplification: false,
                disable_ebf: false,
            },
            
            _ => unreachable!()
        }
    }
    
    pub fn get_output_file(&self) -> Option<&Option<String>> {
        match self {
            Command::Compile { output_file, ..} => Some(output_file),
            _ => None
        }
    }
    
    pub fn get_compile_only(&self) -> Option<bool> {
        match self {
            Command::Compile { compile_only, ..} => Some(*compile_only),
            _ => None
        }
    }
    
    pub fn get_assemble_only(&self) -> Option<bool> {
        match self {
            Command::Compile { assemble_only, ..} => Some(*assemble_only),
            _ => None
        }
    }

    pub fn get_debug_symbols(&self) -> Option<bool> {
        match self {
            Command::Compile { debug_symbols, ..} => Some(*debug_symbols),
            _ => None
        }
    }
    
    pub fn get_disable_bound_checking(&self) -> Option<bool> {
        match self {
            Command::Compile { disable_bound_checking, ..} => Some(*disable_bound_checking),
            _ => None
        }
    }
    
    pub fn get_disable_reordering(&self) -> bool {
        match self {
            Command::Compile { disable_reordering, ..} => *disable_reordering,
            Command::Interpret { disable_reordering, ..} => *disable_reordering,
        }
    }
    
    pub fn get_disable_simplification(&self) -> bool {
        match self {
            Command::Compile { disable_simplification, ..} => *disable_simplification,
            Command::Interpret { disable_simplification, ..} => *disable_simplification,
        }
    }
    
    pub fn get_disable_ebf(&self) -> bool {
        match self {
            Command::Compile { disable_ebf, ..} => *disable_ebf,
            Command::Interpret { disable_ebf, ..} => *disable_ebf,
        }
    }
}

impl Parameters {
    #[cfg(test)]
    fn test_new(input : &str, command : char, output : Option<&str>) -> Parameters {
        Parameters {
            input_file: String::from(input),
            subcommand: Command::test_new(command, output),
        }
    }
    
    pub fn get_input_file(&self) -> String {
        self.input_file.clone()
    }
    
    pub fn get_output_file(&self) -> Option<String> {
        Some(self.subcommand
            .get_output_file()?
            .clone()
            .unwrap_or(
            self.input_file
                .rsplit_once(".")
                .expect("The file has no '.' in it, the argument 'input_file' is not checked good enough.")
                .0
                .to_string()
        ))
    }
    
    pub fn get_dbg_enabled(&self) -> bool {
        self.subcommand
            .get_debug_symbols()
            .unwrap_or(false)
    }
    
    pub fn get_compile_only(&self) -> bool {
        self.subcommand
            .get_compile_only()
            .unwrap_or(false)
    }
    
    pub fn get_assemble_only(&self) -> bool {
        self.subcommand
            .get_assemble_only()
            .unwrap_or(false)
    }
    
    pub fn get_disable_reordering(&self) -> bool {
        self.subcommand.get_disable_reordering()
    }
    
    pub fn get_disable_simplification(&self) -> bool {
        // can't simplify without reordering
        self.subcommand.get_disable_simplification()
        && self.subcommand.get_disable_reordering()
    }
    
    pub fn get_disable_bound_checking(&self) -> bool {
        self.subcommand
            .get_disable_bound_checking()
            .unwrap_or(false)
    }
    
    pub fn get_disable_ebf(&self) -> bool {
        self.subcommand.get_disable_ebf()
    }
    
    pub fn is_compilation(&self) -> bool {
        match self.subcommand {
            Command::Compile { .. } => true,
            _ => false
        }
    }
    
    pub fn is_interpretation(&self) -> bool {
        match self.subcommand {
            Command::Interpret { .. } => true,
            _ => false
        }
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
        assert_eq!(Parameters::test_new("input.bf"        , 'c', None).get_output_file()      , Some(String::from("input")));
        assert_eq!(Parameters::test_new("input.bf.bf"     , 'c', None).get_output_file()      , Some(String::from("input.bf")));
        assert_eq!(Parameters::test_new("input.out.zip.bf", 'c', None).get_output_file()      , Some(String::from("input.out.zip")));
        assert_eq!(Parameters::test_new("input.bf", 'c', Some("out")).get_output_file()       , Some(String::from("out")));
        assert_eq!(Parameters::test_new("input.bf", 'c', Some("out.zip.bf")).get_output_file(), Some(String::from("out.zip.bf")));
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