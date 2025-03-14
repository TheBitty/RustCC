use crate::parser::ast::{Expression, Program, Statement};
use std::collections::{HashMap, HashSet};

/// Handles building and analyzing the call graph
pub struct CallGraph;

impl CallGraph {
    /// Build a call graph for a program
    pub fn build(program: &Program) -> HashMap<String, HashSet<String>> {
        let mut call_graph: HashMap<String, HashSet<String>> = HashMap::new();

        // Initialize empty sets for all functions
        for function in &program.functions {
            call_graph.insert(function.name.clone(), HashSet::new());
        }

        // First collect all function names to avoid borrow conflicts
        let function_names: Vec<String> =
            program.functions.iter().map(|f| f.name.clone()).collect();

        // Fill in calls for each function
        for caller in &function_names {
            // Find the callees for each function
            let callees =
                if let Some(function) = program.functions.iter().find(|f| &f.name == caller) {
                    Self::find_called_functions(&function.body)
                } else {
                    HashSet::new()
                };

            // Add all valid callees to the call graph
            if let Some(callees_set) = call_graph.get_mut(caller) {
                for callee in &callees {
                    // Check if this callee exists in our program
                    if function_names.contains(callee) {
                        callees_set.insert(callee.clone());
                    }
                }
            }
        }

        call_graph
    }

    // Find all functions called within a statement block
    fn find_called_functions(statements: &[Statement]) -> HashSet<String> {
        let mut called_functions = HashSet::new();

        for statement in statements {
            Self::find_called_functions_in_statement(statement, &mut called_functions);
        }

        called_functions
    }

    fn find_called_functions_in_statement(
        statement: &Statement,
        called_functions: &mut HashSet<String>,
    ) {
        match statement {
            Statement::ExpressionStatement(expr) => {
                Self::find_called_functions_in_expr(expr, called_functions);
            }
            Statement::Return(expr) => {
                Self::find_called_functions_in_expr(expr, called_functions);
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    Self::find_called_functions_in_statement(stmt, called_functions);
                }
            }
            Statement::If { condition, then_block, else_block } => {
                Self::find_called_functions_in_expr(condition, called_functions);
                Self::find_called_functions_in_statement(then_block, called_functions);
                if let Some(else_stmt) = else_block {
                    Self::find_called_functions_in_statement(else_stmt, called_functions);
                }
            }
            Statement::While { condition, body } => {
                Self::find_called_functions_in_expr(condition, called_functions);
                Self::find_called_functions_in_statement(body, called_functions);
            }
            Statement::DoWhile { body, condition } => {
                Self::find_called_functions_in_statement(body, called_functions);
                Self::find_called_functions_in_expr(condition, called_functions);
            }
            Statement::For { initializer, condition, increment, body } => {
                if let Some(init) = initializer {
                    Self::find_called_functions_in_statement(init, called_functions);
                }
                if let Some(cond) = condition {
                    Self::find_called_functions_in_expr(cond, called_functions);
                }
                if let Some(inc) = increment {
                    Self::find_called_functions_in_expr(inc, called_functions);
                }
                Self::find_called_functions_in_statement(body, called_functions);
            }
            Statement::Switch { expression, cases } => {
                Self::find_called_functions_in_expr(expression, called_functions);
                for case in cases {
                    for stmt in &case.statements {
                        Self::find_called_functions_in_statement(stmt, called_functions);
                    }
                }
            }
            Statement::VariableDeclaration { initializer, .. } => {
                Self::find_called_functions_in_expr(initializer, called_functions);
            }
            Statement::ArrayDeclaration { initializer, size, .. } => {
                Self::find_called_functions_in_expr(initializer, called_functions);
                if let Some(size_expr) = size {
                    Self::find_called_functions_in_expr(size_expr, called_functions);
                }
            }
            // No function calls in break/continue
            Statement::Break | Statement::Continue => {}
        }
    }

    fn find_called_functions_in_expr(
        expr: &Expression,
        called_functions: &mut HashSet<String>,
    ) {
        match expr {
            Expression::FunctionCall { name, arguments } => {
                called_functions.insert(name.clone());
                for arg in arguments {
                    Self::find_called_functions_in_expr(arg, called_functions);
                }
            }
            Expression::BinaryOperation { left, right, .. } => {
                Self::find_called_functions_in_expr(left, called_functions);
                Self::find_called_functions_in_expr(right, called_functions);
            }
            Expression::UnaryOperation { operand, .. } => {
                Self::find_called_functions_in_expr(operand, called_functions);
            }
            Expression::Assignment { target, value } => {
                Self::find_called_functions_in_expr(target, called_functions);
                Self::find_called_functions_in_expr(value, called_functions);
            }
            Expression::TernaryIf { condition, then_expr, else_expr } => {
                Self::find_called_functions_in_expr(condition, called_functions);
                Self::find_called_functions_in_expr(then_expr, called_functions);
                Self::find_called_functions_in_expr(else_expr, called_functions);
            }
            Expression::Cast { expr: inner_expr, .. } => {
                Self::find_called_functions_in_expr(inner_expr, called_functions);
            }
            Expression::ArrayAccess { array, index } => {
                Self::find_called_functions_in_expr(array, called_functions);
                Self::find_called_functions_in_expr(index, called_functions);
            }
            Expression::StructFieldAccess { object, .. } => {
                Self::find_called_functions_in_expr(object, called_functions);
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                Self::find_called_functions_in_expr(pointer, called_functions);
            }
            Expression::SizeOf(expr_box) => {
                Self::find_called_functions_in_expr(expr_box, called_functions);
            }
            Expression::ArrayLiteral(elements) => {
                for element in elements {
                    Self::find_called_functions_in_expr(element, called_functions);
                }
            }
            // Other expression types don't contain function calls
            _ => {}
        }
    }

    /// Perform a topological sort of the call graph to determine inlining order
    pub fn topological_sort(call_graph: &HashMap<String, HashSet<String>>) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        // Get all function names first to avoid borrowing call_graph multiple times
        let function_names: Vec<String> = call_graph.keys().cloned().collect();

        // Visit each node in the call graph
        for function_name in function_names {
            if !visited.contains(&function_name) {
                Self::visit_node(
                    &function_name,
                    call_graph,
                    &mut visited,
                    &mut temp_visited,
                    &mut result,
                );
            }
        }

        // Reverse for correct inlining order (from leaves to roots)
        result.reverse();
        result
    }

    #[allow(clippy::only_used_in_recursion)]
    fn visit_node(
        node: &str,
        call_graph: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) {
        // Check for cycles (should not happen as we filter recursive functions)
        if temp_visited.contains(node) {
            return; // Cycle detected, skip
        }

        // Skip if already visited
        if visited.contains(node) {
            return;
        }

        // Mark as temporarily visited for cycle detection
        temp_visited.insert(node.to_string());

        // Visit all dependencies first
        if let Some(dependencies) = call_graph.get(node) {
            for dep in dependencies {
                Self::visit_node(dep, call_graph, visited, temp_visited, result);
            }
        }

        // Mark as visited and add to result
        temp_visited.remove(node);
        visited.insert(node.to_string());
        result.push(node.to_string());
    }
} 