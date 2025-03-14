pub mod obfuscation;
pub mod optimization;

use crate::parser::ast::Program;

pub trait Transform {
    fn apply(&self, program: &mut Program) -> Result<(), String>;
    fn name(&self) -> &'static str;
} 