use super::call_graph::CallGraph;
use super::inliner::Inliner;
use super::recursion::RecursionDetector;
use crate::parser::ast::{Function, Program};
use crate::transforms::Transform;

/// Function Inliner transform
/// Performs function inlining for small, non-recursive functions
/// This improves performance and makes reverse engineering more difficult
pub struct FunctionInliner {
    pub max_instructions: usize,
    pub inline_all: bool, // Force inline all eligible functions
}

impl FunctionInliner {
    pub fn new(max_instructions: usize, inline_all: bool) -> Self {
        FunctionInliner {
            max_instructions,
            inline_all,
        }
    }

    fn should_inline(&self, function: &Function) -> bool {
        // Don't inline recursive functions
        if RecursionDetector::is_recursive(function) {
            return false;
        }

        // Force inline if specified
        if self.inline_all {
            return true;
        }

        // Only inline small functions (simple heuristic based on statement count)
        // In a full implementation, we would estimate the actual instruction count
        function.body.len() <= self.max_instructions
    }
}

impl Transform for FunctionInliner {
    fn apply(&self, program: &mut Program) -> std::result::Result<(), String> {
        // 1. Create a call graph
        let call_graph = CallGraph::build(program);

        // 2. Determine inlining order with topological sorting
        let inlining_order = CallGraph::topological_sort(&call_graph);

        // Make a copy of the functions to avoid mutable borrowing issues
        let functions = program.functions.clone();

        // Find all functions that should be inlined, in the correct order
        let mut inline_candidates = Vec::new();
        for function_name in inlining_order {
            if let Some(function) = functions.iter().find(|f| f.name == function_name) {
                if self.should_inline(function) {
                    inline_candidates.push(function);
                }
            }
        }

        if inline_candidates.is_empty() {
            return Ok(());
        }

        println!("Inlining {} functions", inline_candidates.len());

        // Modify each function to inline function calls
        for function in &mut program.functions {
            // Skip functions that are being inlined
            if inline_candidates.iter().any(|f| f.name == function.name) {
                continue;
            }

            // Process statements in the function to inline calls
            Inliner::inline_function_calls(&mut function.body, &inline_candidates);
        }

        // Remove inlined functions from the program if they're only called internally
        // but never remove the main function
        if self.inline_all {
            program.functions.retain(|f| {
                !inline_candidates
                    .iter()
                    .any(|candidate| candidate.name == f.name)
                    || f.name == "main"
            });
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Function Inliner"
    }
}
