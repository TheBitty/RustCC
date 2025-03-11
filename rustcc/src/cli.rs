use crate::compiler::Compiler;
use std::env;

pub struct CLI {
    args: Vec<String>,
}

impl CLI {
    pub fn new() -> Self {
        let args: Vec<String> = env::args().collect();
        CLI { args }
    }
    
    pub fn parse_args(&self) -> Result<(String, String), String> {
        if self.args.len() < 3 {
            return Err("Usage: rustcc <source_file> <output_file>".to_string());
        }
        
        let source_file = self.args[1].clone();
        let output_file = self.args[2].clone();
        
        Ok((source_file, output_file))
    }
    
    pub fn run(&self) -> Result<(), String> {
        let (source_file, output_file) = self.parse_args()?;
        
        println!("Compiling {} to {}", source_file, output_file);
        
        let compiler = Compiler::new(source_file, output_file);
        compiler.compile()?;
        
        println!("Compilation successful!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_args_valid() {
        let cli = CLI {
            args: vec![
                "rustcc".to_string(),
                "input.c".to_string(),
                "output".to_string()
            ],
        };
        
        let result = cli.parse_args();
        assert!(result.is_ok());
        
        let (source, output) = result.unwrap();
        assert_eq!(source, "input.c");
        assert_eq!(output, "output");
    }
    
    #[test]
    fn test_parse_args_invalid() {
        let cli = CLI {
            args: vec!["rustcc".to_string()],
        };
        
        let result = cli.parse_args();
        assert!(result.is_err());
    }
}
