mod variable;
mod control_flow;
mod dead_code;
mod string;

pub use variable::VariableObfuscator;
pub use control_flow::ControlFlowObfuscator;
pub use dead_code::DeadCodeInserter;
pub use string::StringEncryptor;

// Add any common utility functions or shared code here 