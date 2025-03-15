use crate::parser::ast::{Expression, Function, Statement};
use std::collections::{HashMap, HashSet};

/// Methods for detecting recursive function calls
pub struct RecursionDetector;

impl RecursionDetector {
    /// Check if a function is recursive (calls itself directly or indirectly)
    pub fn is_recursive(function: &Function) -> bool {
        // Check for direct recursion first (faster check)
        if Self::has_direct_recursion(function) {
            return true;
        }
        
        // For indirect recursion, we need to build a call graph
        let mut call_graph = HashMap::new();
        Self::build_call_graph_for_function(function, &mut call_graph);
        
        // Check for cycles in the call graph starting from this function
        let mut visited = HashSet::new();
        let mut path = HashSet::new();
        Self::has_cycle_in_call_graph(&function.name, &call_graph, &mut visited, &mut path)
    }
    
    /// Check if a function directly calls itself
    fn has_direct_recursion(function: &Function) -> bool {
        let function_name = &function.name;

        for statement in &function.body {
            if Self::has_recursive_call(statement, function_name) {
                return true;
            }
        }

        false
    }
    
    /// Build a call graph for a function and its callees
    fn build_call_graph_for_function(function: &Function, call_graph: &mut HashMap<String, HashSet<String>>) {
        let mut called_functions = HashSet::new();
        
        // Find all functions called by this function
        for statement in &function.body {
            Self::find_called_functions(statement, &mut called_functions);
        }
        
        // Add to call graph
        call_graph.insert(function.name.clone(), called_functions);
    }
    
    /// Check if there's a cycle in the call graph starting from a node
    fn has_cycle_in_call_graph(
        node: &str,
        call_graph: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        path: &mut HashSet<String>,
    ) -> bool {
        // If node is already in current path, we found a cycle
        if path.contains(node) {
            return true;
        }
        
        // If already visited and no cycle found, skip
        if visited.contains(node) {
            return false;
        }
        
        // Add to current path
        path.insert(node.to_string());
        
        // Check all called functions
        if let Some(callees) = call_graph.get(node) {
            for callee in callees {
                if Self::has_cycle_in_call_graph(callee, call_graph, visited, path) {
                    return true;
                }
            }
        }
        
        // Remove from current path and mark as visited
        path.remove(node);
        visited.insert(node.to_string());
        
        false
    }
    
    /// Find all functions called by a statement
    fn find_called_functions(statement: &Statement, called_functions: &mut HashSet<String>) {
        match statement {
            Statement::ExpressionStatement(expr) => {
                Self::find_called_functions_in_expr(expr, called_functions);
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    Self::find_called_functions(stmt, called_functions);
                }
            }
            Statement::Return(expr) => {
                Self::find_called_functions_in_expr(expr, called_functions);
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                Self::find_called_functions_in_expr(condition, called_functions);
                Self::find_called_functions(then_block, called_functions);
                if let Some(else_stmt) = else_block {
                    Self::find_called_functions(else_stmt, called_functions);
                }
            }
            Statement::While { condition, body } => {
                Self::find_called_functions_in_expr(condition, called_functions);
                Self::find_called_functions(body, called_functions);
            }
            Statement::DoWhile { body, condition } => {
                Self::find_called_functions(body, called_functions);
                Self::find_called_functions_in_expr(condition, called_functions);
            }
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(init) = initializer {
                    Self::find_called_functions(init, called_functions);
                }
                if let Some(cond) = condition {
                    Self::find_called_functions_in_expr(cond, called_functions);
                }
                if let Some(inc) = increment {
                    Self::find_called_functions_in_expr(inc, called_functions);
                }
                Self::find_called_functions(body, called_functions);
            }
            Statement::Switch { expression, cases } => {
                Self::find_called_functions_in_expr(expression, called_functions);
                for case in cases {
                    for stmt in &case.statements {
                        Self::find_called_functions(stmt, called_functions);
                    }
                }
            }
            Statement::VariableDeclaration { initializer, .. } => {
                Self::find_called_functions_in_expr(initializer, called_functions);
            }
            Statement::ArrayDeclaration {
                initializer, size, ..
            } => {
                Self::find_called_functions_in_expr(initializer, called_functions);
                if let Some(size_expr) = size {
                    Self::find_called_functions_in_expr(size_expr, called_functions);
                }
            }
            // No function calls in break/continue
            Statement::Break | Statement::Continue => {}
        }
    }
    
    /// Find all functions called by an expression
    fn find_called_functions_in_expr(expr: &Expression, called_functions: &mut HashSet<String>) {
        match expr {
            Expression::FunctionCall { name, arguments } => {
                // Add the called function
                called_functions.insert(name.clone());
                
                // Check arguments for nested calls
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
            Expression::TernaryIf {
                condition,
                then_expr,
                else_expr,
            } => {
                Self::find_called_functions_in_expr(condition, called_functions);
                Self::find_called_functions_in_expr(then_expr, called_functions);
                Self::find_called_functions_in_expr(else_expr, called_functions);
            }
            Expression::Cast {
                expr: inner_expr, ..
            } => {
                Self::find_called_functions_in_expr(inner_expr, called_functions);
            }
            Expression::SizeOf(inner_expr) => {
                Self::find_called_functions_in_expr(inner_expr, called_functions);
            }
            Expression::ArrayAccess { array, index } => {
                Self::find_called_functions_in_expr(array, called_functions);
                Self::find_called_functions_in_expr(index, called_functions);
            }
            Expression::ArrayLiteral(elements) => {
                for elem in elements {
                    Self::find_called_functions_in_expr(elem, called_functions);
                }
            }
            Expression::StructFieldAccess { object, .. } => {
                Self::find_called_functions_in_expr(object, called_functions);
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                Self::find_called_functions_in_expr(pointer, called_functions);
            }
            // Other expressions don't contain function calls
            _ => {}
        }
    }

    /// Check if a statement contains a recursive call
    fn has_recursive_call(statement: &Statement, function_name: &str) -> bool {
        match statement {
            Statement::ExpressionStatement(expr) => {
                Self::has_recursive_call_in_expr(expr, function_name)
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    if Self::has_recursive_call(stmt, function_name) {
                        return true;
                    }
                }
                false
            }
            Statement::Return(expr) => Self::has_recursive_call_in_expr(expr, function_name),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                Self::has_recursive_call_in_expr(condition, function_name)
                    || Self::has_recursive_call(then_block, function_name)
                    || if let Some(else_stmt) = else_block {
                        Self::has_recursive_call(else_stmt, function_name)
                    } else {
                        false
                    }
            }
            Statement::While { condition, body } => {
                Self::has_recursive_call_in_expr(condition, function_name)
                    || Self::has_recursive_call(body, function_name)
            }
            Statement::DoWhile { body, condition } => {
                Self::has_recursive_call(body, function_name)
                    || Self::has_recursive_call_in_expr(condition, function_name)
            }
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                (if let Some(init) = initializer {
                    Self::has_recursive_call(init, function_name)
                } else {
                    false
                }) || (if let Some(cond) = condition {
                    Self::has_recursive_call_in_expr(cond, function_name)
                } else {
                    false
                }) || (if let Some(inc) = increment {
                    Self::has_recursive_call_in_expr(inc, function_name)
                } else {
                    false
                }) || Self::has_recursive_call(body, function_name)
            }
            Statement::Switch { expression, cases } => {
                Self::has_recursive_call_in_expr(expression, function_name)
                    || cases.iter().any(|case| {
                        case.statements
                            .iter()
                            .any(|stmt| Self::has_recursive_call(stmt, function_name))
                    })
            }
            Statement::VariableDeclaration { initializer, .. } => {
                Self::has_recursive_call_in_expr(initializer, function_name)
            }
            Statement::ArrayDeclaration {
                initializer, size, ..
            } => {
                Self::has_recursive_call_in_expr(initializer, function_name)
                    || if let Some(size_expr) = size {
                        Self::has_recursive_call_in_expr(size_expr, function_name)
                    } else {
                        false
                    }
            }
            // No function calls in break/continue
            Statement::Break | Statement::Continue => false,
        }
    }

    /// Check if an expression contains a recursive call
    fn has_recursive_call_in_expr(expr: &Expression, function_name: &str) -> bool {
        match expr {
            Expression::FunctionCall { name, arguments } => {
                if name == function_name {
                    return true;
                }
                for arg in arguments {
                    if Self::has_recursive_call_in_expr(arg, function_name) {
                        return true;
                    }
                }
                false
            }
            Expression::BinaryOperation { left, right, .. } => {
                Self::has_recursive_call_in_expr(left, function_name)
                    || Self::has_recursive_call_in_expr(right, function_name)
            }
            Expression::UnaryOperation { operand, .. } => {
                Self::has_recursive_call_in_expr(operand, function_name)
            }
            Expression::Assignment { target, value } => {
                Self::has_recursive_call_in_expr(target, function_name)
                    || Self::has_recursive_call_in_expr(value, function_name)
            }
            Expression::TernaryIf {
                condition,
                then_expr,
                else_expr,
            } => {
                Self::has_recursive_call_in_expr(condition, function_name)
                    || Self::has_recursive_call_in_expr(then_expr, function_name)
                    || Self::has_recursive_call_in_expr(else_expr, function_name)
            }
            Expression::Cast {
                expr: inner_expr, ..
            } => Self::has_recursive_call_in_expr(inner_expr, function_name),
            Expression::SizeOf(inner_expr) => {
                Self::has_recursive_call_in_expr(inner_expr, function_name)
            }
            Expression::ArrayAccess { array, index } => {
                Self::has_recursive_call_in_expr(array, function_name)
                    || Self::has_recursive_call_in_expr(index, function_name)
            }
            Expression::ArrayLiteral(elements) => elements
                .iter()
                .any(|elem| Self::has_recursive_call_in_expr(elem, function_name)),
            Expression::StructFieldAccess { object, .. } => {
                Self::has_recursive_call_in_expr(object, function_name)
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                Self::has_recursive_call_in_expr(pointer, function_name)
            }
            // Other expressions don't contain function calls
            _ => false,
        }
    }
}
