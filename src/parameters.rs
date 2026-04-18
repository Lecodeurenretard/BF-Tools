use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Parameters {
    // named versions are indicated by the _n suffix
    // positional versions are indicated by the _p suffix
    #[arg(short, long = "input")]
    #[arg(value_parser = file_is_brainfuck)]
    #[arg(group = "input_file")]
    input_file_n : Option<String>,
    
    #[arg(value_parser = file_is_brainfuck)]
    #[arg(group = "input_file")]
    input_file_p : Option<String>,
    
    #[arg(short, long = "output")]
    output_file : Option<String>,
    
    #[arg(short='g', long="gen-debug")]
    debug_symbols : bool,
    
    #[arg(long)]
    #[arg(default_value_t = 128)]
    cell_count : usize,
}

impl Parameters {
    fn get_input(&self) -> String {
        if self.input_file_p.is_none() {
            return  self.input_file_n.clone().expect("Neither of 'input_file' version has been detected.");
        }
        self.input_file_p.clone().expect("Neither of 'input_file' version has been detected.")
    }
    
    pub fn get_input_file(&self) -> String {
        self.get_input()
    }
    
    pub fn get_output_file(&self) -> String {
        self.output_file.clone().unwrap_or(
            self.get_input().rsplit_once(".")
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