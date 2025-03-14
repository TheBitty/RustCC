use crate::parser::ast::{Program, Statement, Expression, BinaryOp, Type};
use crate::transforms::Transform;
use std::collections::HashMap;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

/// Variable Name Obfuscation
/// Replaces all variable names with random strings
pub struct VariableObfuscator;

impl Transform for VariableObfuscator {
    fn apply(&self, program: &mut Program) -> Result<(), String> {
        let mut rng = thread_rng();
        
        for function in &mut program.functions {
            let mut var_map: HashMap<String, String> = HashMap::new();
            
            // Gather all variable names from the function body
            for statement in &function.body {
                if let Statement::VariableDeclaration { name, .. } = statement {
                    if !var_map.contains_key(name) {
                        // Generate a random name
                        let new_name: String = std::iter::repeat(())
                            .map(|()| rng.sample(Alphanumeric))
                            .map(char::from)
                            .take(8)
                            .collect();
                        
                        var_map.insert(name.clone(), format!("_obf_{}", new_name));
                    }
                }
            }
            
            // Replace variable names in statements
            for statement in &mut function.body {
                self.obfuscate_statement(statement, &var_map);
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "Variable Name Obfuscator"
    }
}

impl VariableObfuscator {
    fn obfuscate_statement(&self, statement: &mut Statement, var_map: &HashMap<String, String>) {
        match statement {
            Statement::VariableDeclaration { name, initializer, data_type: _ } => {
                if let Some(new_name) = var_map.get(name) {
                    *name = new_name.clone();
                }
                self.obfuscate_expression(initializer, var_map);
            }
            Statement::Return(expr) => {
                self.obfuscate_expression(expr, var_map);
            }
            Statement::ExpressionStatement(expr) => {
                self.obfuscate_expression(expr, var_map);
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.obfuscate_statement(stmt, var_map);
                }
            }
            Statement::If { condition, then_block, else_block } => {
                self.obfuscate_expression(condition, var_map);
                self.obfuscate_statement(then_block, var_map);
                if let Some(else_stmt) = else_block {
                    self.obfuscate_statement(else_stmt, var_map);
                }
            }
            Statement::While { condition, body } => {
                self.obfuscate_expression(condition, var_map);
                self.obfuscate_statement(body, var_map);
            }
            Statement::For { initializer, condition, increment, body } => {
                if let Some(init) = initializer {
                    self.obfuscate_statement(init, var_map);
                }
                if let Some(cond) = condition {
                    self.obfuscate_expression(cond, var_map);
                }
                if let Some(inc) = increment {
                    self.obfuscate_expression(inc, var_map);
                }
                self.obfuscate_statement(body, var_map);
            }
            Statement::DoWhile { body, condition } => {
                self.obfuscate_statement(body, var_map);
                self.obfuscate_expression(condition, var_map);
            }
            Statement::Break | Statement::Continue => {}
            Statement::Switch { expression, cases } => {
                self.obfuscate_expression(expression, var_map);
                for case in cases {
                    if let Some(value) = &mut case.value {
                        self.obfuscate_expression(value, var_map);
                    }
                    for stmt in &mut case.statements {
                        self.obfuscate_statement(stmt, var_map);
                    }
                }
            }
        }
    }
    
    fn obfuscate_expression(&self, expr: &mut Expression, var_map: &HashMap<String, String>) {
        match expr {
            Expression::Variable(name) => {
                if let Some(new_name) = var_map.get(name) {
                    *name = new_name.clone();
                }
            }
            Expression::BinaryOperation { left, right, .. } => {
                self.obfuscate_expression(left, var_map);
                self.obfuscate_expression(right, var_map);
            }
            Expression::IntegerLiteral(_) => {}
            Expression::StringLiteral(_) => {}
            Expression::CharLiteral(_) => {}
            Expression::UnaryOperation { operand, .. } => {
                self.obfuscate_expression(operand, var_map);
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.obfuscate_expression(arg, var_map);
                }
            }
            Expression::Assignment { target, value } => {
                self.obfuscate_expression(target, var_map);
                self.obfuscate_expression(value, var_map);
            }
            Expression::TernaryIf { condition, then_expr, else_expr } => {
                self.obfuscate_expression(condition, var_map);
                self.obfuscate_expression(then_expr, var_map);
                self.obfuscate_expression(else_expr, var_map);
            }
            Expression::Cast { expr, .. } => {
                self.obfuscate_expression(expr, var_map);
            }
            Expression::SizeOf { .. } => {}
            Expression::ArrayAccess { array, index } => {
                self.obfuscate_expression(array, var_map);
                self.obfuscate_expression(index, var_map);
            }
            Expression::StructFieldAccess { object, .. } => {
                self.obfuscate_expression(object, var_map);
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                self.obfuscate_expression(pointer, var_map);
            }
        }
    }
}

/// Control Flow Obfuscation
/// Adds extra conditional logic that always evaluates to true
/// but makes the control flow harder to understand
pub struct ControlFlowObfuscator;

impl Transform for ControlFlowObfuscator {
    fn apply(&self, program: &mut Program) -> Result<(), String> {
        for function in &mut program.functions {
            // Only apply to functions with more than one statement
            if function.body.len() > 1 {
                // Find return statements to obfuscate
                let mut return_indices = Vec::new();
                for (i, stmt) in function.body.iter().enumerate() {
                    if matches!(stmt, Statement::Return(_)) {
                        return_indices.push(i);
                    }
                }
                
                // Obfuscate each return statement
                for &idx in &return_indices {
                    if let Statement::Return(expr) = &function.body[idx] {
                        // Create a new obfuscated expression
                        // (x * 2 / 2) to preserve the value
                        let obfuscated_expr = Expression::BinaryOperation {
                            left: Box::new(
                                Expression::BinaryOperation {
                                    left: Box::new(expr.clone()),
                                    operator: BinaryOp::Multiply,
                                    right: Box::new(Expression::IntegerLiteral(2)),
                                }
                            ),
                            operator: BinaryOp::Divide,
                            right: Box::new(Expression::IntegerLiteral(2)),
                        };
                        
                        function.body[idx] = Statement::Return(obfuscated_expr);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "Control Flow Obfuscator"
    }
}

/// Dead Code Insertion
/// Adds meaningless but valid code to confuse reverse engineers
pub struct DeadCodeInserter;

impl Transform for DeadCodeInserter {
    fn apply(&self, program: &mut Program) -> Result<(), String> {
        let mut rng = thread_rng();
        
        for function in &mut program.functions {
            // Generate a list of dummy variable names
            let dummy_var_count = rng.gen_range(1..=3);
            let mut dummy_vars = Vec::new();
            
            for _ in 0..dummy_var_count {
                let var_name: String = std::iter::repeat(())
                    .map(|()| rng.sample(Alphanumeric))
                    .map(char::from)
                    .take(8)
                    .collect();
                
                dummy_vars.push(format!("_dummy_{}", var_name));
            }
            
            // Insert dummy declarations at random positions
            let original_len = function.body.len();
            let mut new_statements = Vec::new();
            
            for (i, stmt) in function.body.drain(..).enumerate() {
                // Decide whether to insert dead code before this statement
                if rng.gen_bool(0.3) && i < original_len - 1 { // Don't insert before the last statement
                    // Pick a random dummy variable
                    let var_idx = rng.gen_range(0..dummy_vars.len());
                    let var_name = dummy_vars[var_idx].clone();
                    
                    // Create a dummy declaration with a random value
                    let dummy_value = rng.gen_range(1..100);
                    let dummy_stmt = Statement::VariableDeclaration {
                        name: var_name.clone(),
                        data_type: Some(Type::Int),
                        initializer: Expression::IntegerLiteral(dummy_value),
                    };
                    
                    new_statements.push(dummy_stmt);
                }
                
                // Add the original statement
                new_statements.push(stmt);
            }
            
            function.body = new_statements;
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "Dead Code Inserter"
    }
} 