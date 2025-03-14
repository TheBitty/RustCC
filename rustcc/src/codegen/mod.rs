#[cfg(feature = "llvm-backend")]
pub mod llvm;
pub mod x86_64;

use crate::parser::ast::Program;

pub struct CodeGenerator {
    backend: Backend,
}

#[allow(dead_code)]
pub enum Backend {
    X86_64,
    #[cfg(feature = "llvm-backend")]
    LLVM,
    #[cfg(not(feature = "llvm-backend"))]
    LLVMUnavailable,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            backend: Backend::X86_64, // Default to x86_64 for backward compatibility
        }
    }

    #[allow(dead_code)]
    pub fn with_backend(backend: Backend) -> Self {
        CodeGenerator { backend }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        match self.backend {
            Backend::X86_64 => {
                let mut generator = x86_64::X86_64Generator::new();
                generator.generate(program)
            }
            #[cfg(feature = "llvm-backend")]
            Backend::LLVM => {
                // LLVM requires context, which we can't easily create here
                // So we'll just return a message for now
                "LLVM IR generation is available through the LLVMCodeGenerator directly.".to_string()
            }
            #[cfg(not(feature = "llvm-backend"))]
            Backend::LLVMUnavailable => {
                "LLVM backend is not available. Compile with the 'llvm-backend' feature to enable it.".to_string()
            }
        }
    }
}
