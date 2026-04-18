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
    
    /// The number of cells the program have to allocate.
    #[arg(long)]
    #[arg(default_value_t = 128, value_parser=number_strictly_positive)]
    cell_count : usize,
}

impl Parameters {
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
    
    pub fn get_cell_count(&self) -> usize {
        self.cell_count
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
}


fn file_exists(s : &str) -> Result<String, String> {
    match std::fs::exists(s) {
        Ok(file_exists) => {
            if file_exists {
                Ok(String::from(s))
            } else {
                Err(format!("The following file is not found: `{s}`"))
            }
        }
        Err(err) => Err(format!("Please verify the permissions of the file `{err}`."))
    }
}

fn file_is_brainfuck(s : &str) -> Result<String, String> {
    let filename = file_exists(s)?;
    if filename.ends_with(".bf") {
        return Ok(String::from(s));
    } else {
        return Err(String::from("The file is not a brainfuck file (.bf)."));
    }
}

fn number_strictly_positive(s : &str) -> Result<usize, String> {
    let x : i32 = s.parse().expect("Argument is not a number");
    if x <= 0 {
        return Err(format!("{x} is negative or null."));
    }
    Ok(x as usize)
}