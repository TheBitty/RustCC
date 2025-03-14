use crate::parser::ast::{Program, Statement, Expression};
use crate::transforms::Transform;
use std::collections::{HashSet, HashMap};

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
    
    fn should_inline(&self, function: &crate::parser::ast::Function) -> bool {
        // Don't inline recursive functions
        if Self::is_recursive(function) {
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
    
    fn is_recursive(function: &crate::parser::ast::Function) -> bool {
        // Simple implementation that looks for function calls with the same name
        // A more accurate implementation would do full call graph analysis
        let function_name = &function.name;
        
        for statement in &function.body {
            if Self::has_recursive_call(statement, function_name) {
                return true;
            }
        }
        
        false
    }
    
    fn has_recursive_call(statement: &crate::parser::ast::Statement, function_name: &str) -> bool {
        use crate::parser::ast::Statement;
        
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
            Statement::If { condition, then_block, else_block } => {
                Self::has_recursive_call_in_expr(condition, function_name) ||
                Self::has_recursive_call(then_block, function_name) ||
                if let Some(else_stmt) = else_block {
                    Self::has_recursive_call(else_stmt, function_name)
                } else {
                    false
                }
            }
            Statement::While { condition, body } => {
                Self::has_recursive_call_in_expr(condition, function_name) ||
                Self::has_recursive_call(body, function_name)
            }
            Statement::For { initializer, condition, increment, body } => {
                (if let Some(init) = initializer {
                    Self::has_recursive_call(init, function_name)
                } else {
                    false
                }) ||
                (if let Some(cond) = condition {
                    Self::has_recursive_call_in_expr(cond, function_name)
                } else {
                    false
                }) ||
                (if let Some(inc) = increment {
                    Self::has_recursive_call_in_expr(inc, function_name)
                } else {
                    false
                }) ||
                Self::has_recursive_call(body, function_name)
            }
            Statement::DoWhile { body, condition } => {
                Self::has_recursive_call(body, function_name) ||
                Self::has_recursive_call_in_expr(condition, function_name)
            }
            Statement::Return(expr) => {
                Self::has_recursive_call_in_expr(expr, function_name)
            }
            _ => false
        }
    }
    
    fn has_recursive_call_in_expr(expr: &crate::parser::ast::Expression, function_name: &str) -> bool {
        use crate::parser::ast::Expression;
        
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
                Self::has_recursive_call_in_expr(left, function_name) ||
                Self::has_recursive_call_in_expr(right, function_name)
            }
            Expression::UnaryOperation { operand, .. } => {
                Self::has_recursive_call_in_expr(operand, function_name)
            }
            Expression::Assignment { target, value } => {
                Self::has_recursive_call_in_expr(target, function_name) ||
                Self::has_recursive_call_in_expr(value, function_name)
            }
            _ => false
        }
    }
    
    // Create a call graph to understand function dependencies
    fn build_call_graph(&self, program: &Program) -> HashMap<String, HashSet<String>> {
        let mut call_graph: HashMap<String, HashSet<String>> = HashMap::new();
        
        // Initialize empty sets for all functions
        for function in &program.functions {
            call_graph.insert(function.name.clone(), HashSet::new());
        }
        
        // First collect all function names to avoid borrow conflicts
        let function_names: Vec<String> = program.functions.iter()
            .map(|f| f.name.clone())
            .collect();
        
        // Fill in calls for each function
        for caller in &function_names {
            // Find the callees for each function
            let callees = if let Some(function) = program.functions.iter().find(|f| &f.name == caller) {
                self.find_called_functions(&function.body)
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
    fn find_called_functions(&self, statements: &[Statement]) -> HashSet<String> {
        let mut called_functions = HashSet::new();
        
        for statement in statements {
            self.find_called_functions_in_statement(statement, &mut called_functions);
        }
        
        called_functions
    }
    
    fn find_called_functions_in_statement(&self, statement: &Statement, called_functions: &mut HashSet<String>) {
        match statement {
            Statement::ExpressionStatement(expr) => {
                self.find_called_functions_in_expr(expr, called_functions);
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.find_called_functions_in_statement(stmt, called_functions);
                }
            }
            Statement::If { condition, then_block, else_block } => {
                self.find_called_functions_in_expr(condition, called_functions);
                self.find_called_functions_in_statement(then_block, called_functions);
                if let Some(else_stmt) = else_block {
                    self.find_called_functions_in_statement(else_stmt, called_functions);
                }
            }
            Statement::While { condition, body } => {
                self.find_called_functions_in_expr(condition, called_functions);
                self.find_called_functions_in_statement(body, called_functions);
            }
            Statement::For { initializer, condition, increment, body } => {
                if let Some(init) = initializer {
                    self.find_called_functions_in_statement(init, called_functions);
                }
                if let Some(cond) = condition {
                    self.find_called_functions_in_expr(cond, called_functions);
                }
                if let Some(inc) = increment {
                    self.find_called_functions_in_expr(inc, called_functions);
                }
                self.find_called_functions_in_statement(body, called_functions);
            }
            Statement::DoWhile { body, condition } => {
                self.find_called_functions_in_statement(body, called_functions);
                self.find_called_functions_in_expr(condition, called_functions);
            }
            Statement::Return(expr) => {
                self.find_called_functions_in_expr(expr, called_functions);
            }
            Statement::VariableDeclaration { initializer, .. } => {
                self.find_called_functions_in_expr(initializer, called_functions);
            }
            Statement::Switch { expression, cases } => {
                self.find_called_functions_in_expr(expression, called_functions);
                for case in cases {
                    for stmt in &case.statements {
                        self.find_called_functions_in_statement(stmt, called_functions);
                    }
                }
            }
            Statement::Break | Statement::Continue => {}
        }
    }
    
    fn find_called_functions_in_expr(&self, expr: &Expression, called_functions: &mut HashSet<String>) {
        match expr {
            Expression::FunctionCall { name, arguments } => {
                called_functions.insert(name.clone());
                for arg in arguments {
                    self.find_called_functions_in_expr(arg, called_functions);
                }
            }
            Expression::BinaryOperation { left, right, .. } => {
                self.find_called_functions_in_expr(left, called_functions);
                self.find_called_functions_in_expr(right, called_functions);
            }
            Expression::UnaryOperation { operand, .. } => {
                self.find_called_functions_in_expr(operand, called_functions);
            }
            Expression::Assignment { target, value } => {
                self.find_called_functions_in_expr(target, called_functions);
                self.find_called_functions_in_expr(value, called_functions);
            }
            Expression::TernaryIf { condition, then_expr, else_expr } => {
                self.find_called_functions_in_expr(condition, called_functions);
                self.find_called_functions_in_expr(then_expr, called_functions);
                self.find_called_functions_in_expr(else_expr, called_functions);
            }
            Expression::Cast { expr, .. } => {
                self.find_called_functions_in_expr(expr, called_functions);
            }
            Expression::ArrayAccess { array, index } => {
                self.find_called_functions_in_expr(array, called_functions);
                self.find_called_functions_in_expr(index, called_functions);
            }
            Expression::StructFieldAccess { object, .. } => {
                self.find_called_functions_in_expr(object, called_functions);
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                self.find_called_functions_in_expr(pointer, called_functions);
            }
            Expression::IntegerLiteral(_) | Expression::StringLiteral(_) |
            Expression::CharLiteral(_) | Expression::SizeOf { .. } |
            Expression::Variable(_) => {
                // These don't call functions
            }
        }
    }
    
    // Perform a topological sort of the call graph to determine inlining order
    fn topological_sort(&self, call_graph: &HashMap<String, HashSet<String>>) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        
        // Get all function names first to avoid borrowing call_graph multiple times
        let function_names: Vec<String> = call_graph.keys().cloned().collect();
        
        // Visit each node in the call graph
        for function_name in function_names {
            if !visited.contains(&function_name) {
                self.visit_node(&function_name, call_graph, &mut visited, &mut temp_visited, &mut result);
            }
        }
        
        // Reverse for correct inlining order (from leaves to roots)
        result.reverse();
        result
    }
    
    fn visit_node(
        &self,
        node: &str,
        call_graph: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        result: &mut Vec<String>
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
                self.visit_node(dep, call_graph, visited, temp_visited, result);
            }
        }
        
        // Mark as visited and add to result
        temp_visited.remove(node);
        visited.insert(node.to_string());
        result.push(node.to_string());
    }
    
    fn inline_function_calls(
        &self,
        statements: &mut Vec<Statement>,
        inline_candidates: &[&crate::parser::ast::Function],
    ) {
        use crate::parser::ast::{Statement, Expression};
        
        // Process each statement in the block
        for i in 0..statements.len() {
            match &mut statements[i] {
                Statement::ExpressionStatement(expr) => {
                    self.inline_function_calls_in_expr(expr, inline_candidates);
                }
                Statement::Block(block_statements) => {
                    self.inline_function_calls(block_statements, inline_candidates);
                }
                Statement::If { condition, then_block, else_block } => {
                    self.inline_function_calls_in_expr(condition, inline_candidates);
                    
                    match then_block.as_mut() {
                        Statement::Block(block_statements) => {
                            self.inline_function_calls(block_statements, inline_candidates);
                        }
                        _ => {
                            // Convert to block if it's not already
                            let stmt = std::mem::replace(then_block, Box::new(Statement::Block(vec![])));
                            if let Statement::Block(block_statements) = then_block.as_mut() {
                                block_statements.push(*stmt);  // Dereference Box<Statement> to Statement
                                self.inline_function_calls(block_statements, inline_candidates);
                            }
                        }
                    }
                    
                    if let Some(else_stmt) = else_block {
                        match else_stmt.as_mut() {
                            Statement::Block(block_statements) => {
                                self.inline_function_calls(block_statements, inline_candidates);
                            }
                            _ => {
                                // Convert to block if it's not already
                                let stmt = std::mem::replace(else_stmt, Box::new(Statement::Block(vec![])));
                                if let Statement::Block(block_statements) = else_stmt.as_mut() {
                                    block_statements.push(*stmt);  // Dereference Box<Statement> to Statement
                                    self.inline_function_calls(block_statements, inline_candidates);
                                }
                            }
                        }
                    }
                }
                Statement::While { condition, body } => {
                    self.inline_function_calls_in_expr(condition, inline_candidates);
                    match body.as_mut() {
                        Statement::Block(block_statements) => {
                            self.inline_function_calls(block_statements, inline_candidates);
                        }
                        _ => {
                            // Convert to block if it's not already
                            let stmt = std::mem::replace(body, Box::new(Statement::Block(vec![])));
                            if let Statement::Block(block_statements) = body.as_mut() {
                                block_statements.push(*stmt);
                                self.inline_function_calls(block_statements, inline_candidates);
                            }
                        }
                    }
                }
                Statement::For { initializer, condition, increment, body } => {
                    if let Some(init) = initializer {
                        match init.as_mut() {
                            Statement::Block(block_statements) => {
                                self.inline_function_calls(block_statements, inline_candidates);
                            }
                            _ => {}
                        }
                    }
                    
                    if let Some(cond) = condition {
                        self.inline_function_calls_in_expr(cond, inline_candidates);
                    }
                    
                    if let Some(inc) = increment {
                        self.inline_function_calls_in_expr(inc, inline_candidates);
                    }
                    
                    match body.as_mut() {
                        Statement::Block(block_statements) => {
                            self.inline_function_calls(block_statements, inline_candidates);
                        }
                        _ => {
                            // Convert to block if it's not already
                            let stmt = std::mem::replace(body, Box::new(Statement::Block(vec![])));
                            if let Statement::Block(block_statements) = body.as_mut() {
                                block_statements.push(*stmt);
                                self.inline_function_calls(block_statements, inline_candidates);
                            }
                        }
                    }
                }
                Statement::DoWhile { body, condition } => {
                    self.inline_function_calls_in_expr(condition, inline_candidates);
                    match body.as_mut() {
                        Statement::Block(block_statements) => {
                            self.inline_function_calls(block_statements, inline_candidates);
                        }
                        _ => {
                            // Convert to block if it's not already
                            let stmt = std::mem::replace(body, Box::new(Statement::Block(vec![])));
                            if let Statement::Block(block_statements) = body.as_mut() {
                                block_statements.push(*stmt);
                                self.inline_function_calls(block_statements, inline_candidates);
                            }
                        }
                    }
                }
                Statement::VariableDeclaration { initializer, .. } => {
                    self.inline_function_calls_in_expr(initializer, inline_candidates);
                }
                Statement::Return(expr) => {
                    self.inline_function_calls_in_expr(expr, inline_candidates);
                }
                Statement::Switch { expression, cases } => {
                    self.inline_function_calls_in_expr(expression, inline_candidates);
                    for case in cases {
                        self.inline_function_calls(&mut case.statements, inline_candidates);
                    }
                }
                Statement::Break | Statement::Continue => {}
            }
        }
        
        // Second pass to actually perform inlining
        let mut i = 0;
        while i < statements.len() {
            let should_inline = match &statements[i] {
                Statement::ExpressionStatement(Expression::FunctionCall { name, .. }) => {
                    inline_candidates.iter().any(|f| &f.name == name)
                },
                _ => false
            };
            
            if should_inline {
                if let Statement::ExpressionStatement(Expression::FunctionCall { name, arguments: args }) = &statements[i].clone() {
                    // Find the function to inline
                    if let Some(function_to_inline) = inline_candidates.iter().find(|f| &f.name == name) {
                        // Create a variable name prefix to avoid name collisions
                        let prefix = format!("__inline_{}_", name);
                        
                        // Remove the original function call
                        statements.remove(i);
                        
                        // Create a block for the inlined function with variable declarations for parameters
                        let mut inlined_statements = Vec::new();
                        
                        // 3. Create a new scope for each inlined function
                        // Declare parameters and assign arguments
                        for (param_idx, param) in function_to_inline.parameters.iter().enumerate() {
                            let arg_expr = if param_idx < args.len() {
                                args[param_idx].clone()
                            } else {
                                // Default value if not enough arguments (should not happen in valid code)
                                Expression::IntegerLiteral(0)
                            };
                            
                            // Create a new variable for the parameter
                            let param_var = Statement::VariableDeclaration {
                                name: format!("{}{}", prefix, param.name),
                                data_type: Some(param.data_type.clone()),
                                initializer: arg_expr,
                            };
                            
                            inlined_statements.push(param_var);
                        }
                        
                        // Create a variable for the return value if needed
                        let has_return = function_to_inline.body.iter().any(|stmt| {
                            matches!(stmt, Statement::Return(_))
                        });
                        
                        let return_var_name = format!("{}return_val", prefix);
                        if has_return {
                            // Declare the return variable
                            inlined_statements.push(Statement::VariableDeclaration {
                                name: return_var_name.clone(),
                                data_type: Some(function_to_inline.return_type.clone()),
                                initializer: Expression::IntegerLiteral(0), // Default initialization
                            });
                        }
                        
                        // Process each statement from the function body
                        let mut processed_body = Vec::new();
                        let mut _had_early_return = false;
                        
                        // 4. Handle return statements by converting them to assignments
                        for statement in &function_to_inline.body {
                            if let Statement::Return(expr) = statement {
                                // Convert return to an assignment to the return variable
                                if has_return {
                                    let mut expr_clone = expr.clone();
                                    self.rename_variables_in_expr(&mut expr_clone, &prefix, &function_to_inline.parameters);
                                    processed_body.push(Statement::ExpressionStatement(
                                        Expression::Assignment {
                                            target: Box::new(Expression::Variable(return_var_name.clone())),
                                            value: Box::new(expr_clone),
                                        }
                                    ));
                                }
                                
                                // Early return - add a flag and break out of the loop
                                _had_early_return = true;
                                break;
                            } else {
                                // Clone the statement and rename variables if needed
                                let mut cloned_stmt = statement.clone();
                                self.rename_variables_in_statement(&mut cloned_stmt, &prefix, &function_to_inline.parameters);
                                processed_body.push(cloned_stmt);
                            }
                        }
                        
                        // Add all the processed statements
                        inlined_statements.extend(processed_body);
                        
                        // If this was a call expression, replace it with the return variable
                        if has_return {
                            // Use the return variable as the result of the expression
                            // If we're in a statement context, we need to assign to something
                            // The caller function would check the context and handle accordingly
                        }
                        
                        // Insert inlined statements
                        for (insert_idx, stmt) in inlined_statements.into_iter().enumerate() {
                            statements.insert(i + insert_idx, stmt);
                        }
                        
                        // Don't increment i since we need to process the newly inserted statements
                        continue;
                    }
                }
            }
            
            i += 1;
        }
    }
    
    // Helper function to rename variables in statements to avoid conflicts
    fn rename_variables_in_statement(
        &self, 
        statement: &mut Statement, 
        prefix: &str, 
        parameters: &[crate::parser::ast::FunctionParameter]
    ) {
        match statement {
            Statement::VariableDeclaration { name, initializer, .. } => {
                // Rename the variable declaration
                *name = format!("{}{}", prefix, name);
                
                // Process the initializer
                self.rename_variables_in_expr(initializer, prefix, parameters);
            }
            Statement::ExpressionStatement(expr) => {
                self.rename_variables_in_expr(expr, prefix, parameters);
            }
            Statement::Return(expr) => {
                self.rename_variables_in_expr(expr, prefix, parameters);
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.rename_variables_in_statement(stmt, prefix, parameters);
                }
            }
            Statement::If { condition, then_block, else_block } => {
                self.rename_variables_in_expr(condition, prefix, parameters);
                self.rename_variables_in_statement(then_block, prefix, parameters);
                if let Some(else_stmt) = else_block {
                    self.rename_variables_in_statement(else_stmt, prefix, parameters);
                }
            }
            Statement::While { condition, body } => {
                self.rename_variables_in_expr(condition, prefix, parameters);
                self.rename_variables_in_statement(body, prefix, parameters);
            }
            Statement::For { initializer, condition, increment, body } => {
                if let Some(init) = initializer {
                    self.rename_variables_in_statement(init, prefix, parameters);
                }
                if let Some(cond) = condition {
                    self.rename_variables_in_expr(cond, prefix, parameters);
                }
                if let Some(inc) = increment {
                    self.rename_variables_in_expr(inc, prefix, parameters);
                }
                self.rename_variables_in_statement(body, prefix, parameters);
            }
            Statement::DoWhile { body, condition } => {
                self.rename_variables_in_statement(body, prefix, parameters);
                self.rename_variables_in_expr(condition, prefix, parameters);
            }
            Statement::Switch { expression, cases } => {
                self.rename_variables_in_expr(expression, prefix, parameters);
                for case in cases {
                    for stmt in &mut case.statements {
                        self.rename_variables_in_statement(stmt, prefix, parameters);
                    }
                }
            }
            Statement::Break | Statement::Continue => {}
        }
    }
    
    fn rename_variables_in_expr(
        &self, 
        expr: &mut Expression, 
        prefix: &str, 
        parameters: &[crate::parser::ast::FunctionParameter]
    ) {
        match expr {
            Expression::Variable(name) => {
                // Check if this is a parameter name that needs to be renamed
                if parameters.iter().any(|p| p.name == *name) {
                    *name = format!("{}{}", prefix, name);
                }
            }
            Expression::BinaryOperation { left, right, .. } => {
                self.rename_variables_in_expr(left, prefix, parameters);
                self.rename_variables_in_expr(right, prefix, parameters);
            }
            Expression::UnaryOperation { operand, .. } => {
                self.rename_variables_in_expr(operand, prefix, parameters);
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.rename_variables_in_expr(arg, prefix, parameters);
                }
            }
            Expression::Assignment { target, value } => {
                self.rename_variables_in_expr(target, prefix, parameters);
                self.rename_variables_in_expr(value, prefix, parameters);
            }
            Expression::TernaryIf { condition, then_expr, else_expr } => {
                self.rename_variables_in_expr(condition, prefix, parameters);
                self.rename_variables_in_expr(then_expr, prefix, parameters);
                self.rename_variables_in_expr(else_expr, prefix, parameters);
            }
            Expression::Cast { expr: inner_expr, .. } => {
                self.rename_variables_in_expr(inner_expr, prefix, parameters);
            }
            Expression::ArrayAccess { array, index } => {
                self.rename_variables_in_expr(array, prefix, parameters);
                self.rename_variables_in_expr(index, prefix, parameters);
            }
            Expression::StructFieldAccess { object, .. } => {
                self.rename_variables_in_expr(object, prefix, parameters);
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                self.rename_variables_in_expr(pointer, prefix, parameters);
            }
            Expression::IntegerLiteral(_) | Expression::StringLiteral(_) |
            Expression::CharLiteral(_) | Expression::SizeOf { .. } => {
                // These don't have variables to rename
            }
        }
    }
    
    fn inline_function_calls_in_expr(
        &self,
        expr: &mut Expression,
        inline_candidates: &[&crate::parser::ast::Function],
    ) {
        // Fix the unused import warning
        use crate::parser::ast::Expression;
        
        match expr {
            Expression::FunctionCall { name: _, arguments } => {
                // Inline arguments first (depth-first)
                for arg in arguments {
                    self.inline_function_calls_in_expr(arg, inline_candidates);
                }
                
                // We don't actually inline the function here, that's done in the statement pass
                // This is just to process nested calls in the arguments
            }
            Expression::BinaryOperation { left, right, .. } => {
                self.inline_function_calls_in_expr(left, inline_candidates);
                self.inline_function_calls_in_expr(right, inline_candidates);
            }
            Expression::UnaryOperation { operand, .. } => {
                self.inline_function_calls_in_expr(operand, inline_candidates);
            }
            Expression::Assignment { target, value } => {
                self.inline_function_calls_in_expr(target, inline_candidates);
                self.inline_function_calls_in_expr(value, inline_candidates);
            }
            Expression::TernaryIf { condition, then_expr, else_expr } => {
                self.inline_function_calls_in_expr(condition, inline_candidates);
                self.inline_function_calls_in_expr(then_expr, inline_candidates);
                self.inline_function_calls_in_expr(else_expr, inline_candidates);
            }
            Expression::Cast { expr: inner_expr, .. } => {
                self.inline_function_calls_in_expr(inner_expr, inline_candidates);
            }
            Expression::ArrayAccess { array, index } => {
                self.inline_function_calls_in_expr(array, inline_candidates);
                self.inline_function_calls_in_expr(index, inline_candidates);
            }
            Expression::StructFieldAccess { object, .. } => {
                self.inline_function_calls_in_expr(object, inline_candidates);
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                self.inline_function_calls_in_expr(pointer, inline_candidates);
            }
            _ => {}
        }
    }
}

impl Transform for FunctionInliner {
    fn apply(&self, program: &mut Program) -> Result<(), String> {
        // 1. Create a call graph
        let call_graph = self.build_call_graph(program);
        
        // 2. Determine inlining order with topological sorting
        let inlining_order = self.topological_sort(&call_graph);
        
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
            self.inline_function_calls(&mut function.body, &inline_candidates);
        }
        
        // Remove inlined functions from the program if they're only called internally
        // but never remove the main function
        if self.inline_all {
            program.functions.retain(|f| 
                !inline_candidates.iter().any(|candidate| candidate.name == f.name) || 
                f.name == "main"
            );
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "Function Inliner"
    }
} 