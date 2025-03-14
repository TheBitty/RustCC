mod control_flow;
mod dead_code;
mod string;
mod variable;

pub use control_flow::ControlFlowObfuscator;
pub use dead_code::DeadCodeInserter;
pub use string::StringEncryptor;
pub use variable::VariableObfuscator;

// Add any common utility functions or shared code here
