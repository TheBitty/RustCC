pub mod obfuscation;
pub mod api;
pub mod string;
pub mod control_flow;
pub mod optimization;

use crate::parser::ast::Program;

pub trait Transform {
    fn apply(&self, program: &mut Program) -> Result<(), String>;
    fn name(&self) -> &'static str;
} 