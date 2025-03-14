pub mod obfuscation;
pub mod optimization;

use crate::parser::ast::Program;

/// A transform that can be applied to a program AST
pub trait Transform {
    /// Apply the transform to the given program
    fn apply(&self, program: &mut Program) -> std::result::Result<(), String>;

    /// Get the name of the transform
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}
