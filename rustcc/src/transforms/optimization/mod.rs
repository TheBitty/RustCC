mod constant_folder;
mod dead_code;
mod function_inliner;

pub use constant_folder::ConstantFolder;
pub use dead_code::DeadCodeEliminator;
pub use function_inliner::FunctionInliner;

#[cfg(test)]
mod tests;
