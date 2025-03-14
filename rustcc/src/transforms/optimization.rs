use crate::parser::ast::{Program, Statement, Expression, BinaryOp, UnaryOp};
use crate::transforms::Transform;
use std::collections::HashSet;

/// Constant Folding Optimization
/// Evaluates constant expressions at compile time
pub struct ConstantFolder;

impl Transform for ConstantFolder {
    fn apply(&self, program: &mut Program) -> Result<(), String> {
        for function in &mut program.functions {
            for statement in &mut function.body {
                self.fold_statement(statement);
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "Constant Folder"
    }
}

impl ConstantFolder {
    fn fold_statement(&self, statement: &mut Statement) {
        match statement {
            Statement::VariableDeclaration { initializer, .. } => {
                *initializer = self.fold_expression(initializer);
            }
            Statement::Return(expr) => {
                *expr = self.fold_expression(expr);
            }
            Statement::ExpressionStatement(expr) => {
                *expr = self.fold_expression(expr);
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.fold_statement(stmt);
                }
            }
            Statement::If { condition, then_block, else_block } => {
                *condition = self.fold_expression(condition);
                self.fold_statement(then_block);
                if let Some(else_stmt) = else_block {
                    self.fold_statement(else_stmt);
                }
            }
            Statement::While { condition, body } => {
                *condition = self.fold_expression(condition);
                self.fold_statement(body);
            }
            Statement::For { initializer, condition, increment, body } => {
                if let Some(init) = initializer {
                    self.fold_statement(init);
                }
                if let Some(cond) = condition {
                    *cond = self.fold_expression(cond);
                }
                if let Some(inc) = increment {
                    *inc = self.fold_expression(inc);
                }
                self.fold_statement(body);
            }
            Statement::DoWhile { body, condition } => {
                self.fold_statement(body);
                *condition = self.fold_expression(condition);
            }
            Statement::Break | Statement::Continue => {}
            Statement::Switch { expression, cases } => {
                *expression = self.fold_expression(expression);
                for case in cases {
                    if let Some(value) = &mut case.value {
                        *value = self.fold_expression(value);
                    }
                    for stmt in &mut case.statements {
                        self.fold_statement(stmt);
                    }
                }
            }
        }
    }

    fn fold_expression(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::BinaryOperation { left, operator, right } => {
                // Recursively fold the operands
                let folded_left = self.fold_expression(left);
                let folded_right = self.fold_expression(right);
                
                // Check if both operands are now constants
                if let (Expression::IntegerLiteral(left_val), Expression::IntegerLiteral(right_val)) = 
                    (&folded_left, &folded_right) {
                    // Perform the operation at compile time
                    let result = match operator {
                        BinaryOp::Add => left_val + right_val,
                        BinaryOp::Subtract => left_val - right_val,
                        BinaryOp::Multiply => left_val * right_val,
                        BinaryOp::Divide => {
                            if *right_val != 0 {
                                left_val / right_val
                            } else {
                                // Can't divide by zero, return the expression as is
                                return Expression::BinaryOperation {
                                    left: Box::new(folded_left),
                                    operator: operator.clone(),
                                    right: Box::new(folded_right),
                                };
                            }
                        }
                        _ => {
                            // For other operators, just return with folded operands
                            return Expression::BinaryOperation {
                                left: Box::new(folded_left),
                                operator: operator.clone(),
                                right: Box::new(folded_right),
                            };
                        }
                    };
                    
                    Expression::IntegerLiteral(result)
                } else {
                    // If we can't fold, return the expression with folded operands
                    Expression::BinaryOperation {
                        left: Box::new(folded_left),
                        operator: operator.clone(),
                        right: Box::new(folded_right),
                    }
                }
            }
            Expression::UnaryOperation { operator, operand } => {
                let folded_operand = self.fold_expression(operand);
                
                // Only for integer literals do we calculate the result now
                if let Expression::IntegerLiteral(val) = folded_operand {
                    match operator {
                        UnaryOp::Negate => Expression::IntegerLiteral(-val),
                        _ => Expression::UnaryOperation {
                            operator: operator.clone(),
                            operand: Box::new(folded_operand),
                        },
                    }
                } else {
                    Expression::UnaryOperation {
                        operator: operator.clone(),
                        operand: Box::new(folded_operand),
                    }
                }
            }
            Expression::FunctionCall { name, arguments } => {
                let folded_args = arguments.iter()
                    .map(|arg| self.fold_expression(arg))
                    .collect();
                
                Expression::FunctionCall {
                    name: name.clone(),
                    arguments: folded_args,
                }
            }
            // For other expressions, return as is
            _ => expr.clone(),
        }
    }
}

/// Dead Code Elimination
/// Removes variables that are never used
pub struct DeadCodeEliminator;

impl Transform for DeadCodeEliminator {
    fn apply(&self, program: &mut Program) -> Result<(), String> {
        for function in &mut program.functions {
            // Find all used variables in the function
            let used_vars = self.find_used_variables(function);
            
            // Remove declarations of unused variables
            function.body.retain(|stmt| {
                match stmt {
                    Statement::VariableDeclaration { name, .. } => {
                        used_vars.contains(name)
                    }
                    _ => true,
                }
            });
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "Dead Code Eliminator"
    }
}

impl DeadCodeEliminator {
    fn find_used_variables(&self, function: &mut crate::parser::ast::Function) -> HashSet<String> {
        let mut used_vars = HashSet::new();
        
        // Then find variables used in expressions by traversing all statements
        for statement in &function.body {
            self.find_used_vars_in_statement(statement, &mut used_vars);
        }
        
        used_vars
    }
    
    fn find_used_vars_in_statement(&self, statement: &Statement, used_vars: &mut HashSet<String>) {
        match statement {
            Statement::Return(expr) => {
                self.find_used_vars_in_expr(expr, used_vars);
            }
            Statement::VariableDeclaration { initializer, .. } => {
                self.find_used_vars_in_expr(initializer, used_vars);
            }
            Statement::ExpressionStatement(expr) => {
                self.find_used_vars_in_expr(expr, used_vars);
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.find_used_vars_in_statement(stmt, used_vars);
                }
            }
            Statement::If { condition, then_block, else_block } => {
                self.find_used_vars_in_expr(condition, used_vars);
                self.find_used_vars_in_statement(then_block, used_vars);
                if let Some(else_stmt) = else_block {
                    self.find_used_vars_in_statement(else_stmt, used_vars);
                }
            }
            Statement::While { condition, body } => {
                self.find_used_vars_in_expr(condition, used_vars);
                self.find_used_vars_in_statement(body, used_vars);
            }
            Statement::For { initializer, condition, increment, body } => {
                if let Some(init) = initializer {
                    self.find_used_vars_in_statement(init, used_vars);
                }
                if let Some(cond) = condition {
                    self.find_used_vars_in_expr(cond, used_vars);
                }
                if let Some(inc) = increment {
                    self.find_used_vars_in_expr(inc, used_vars);
                }
                self.find_used_vars_in_statement(body, used_vars);
            }
            Statement::DoWhile { body, condition } => {
                self.find_used_vars_in_statement(body, used_vars);
                self.find_used_vars_in_expr(condition, used_vars);
            }
            Statement::Break | Statement::Continue => {}
            Statement::Switch { expression, cases } => {
                self.find_used_vars_in_expr(expression, used_vars);
                for case in cases {
                    if let Some(value) = &case.value {
                        self.find_used_vars_in_expr(value, used_vars);
                    }
                    for stmt in &case.statements {
                        self.find_used_vars_in_statement(stmt, used_vars);
                    }
                }
            }
        }
    }
    
    fn find_used_vars_in_expr(&self, expr: &Expression, used_vars: &mut HashSet<String>) {
        match expr {
            Expression::Variable(name) => {
                used_vars.insert(name.clone());
            }
            Expression::BinaryOperation { left, right, .. } => {
                self.find_used_vars_in_expr(left, used_vars);
                self.find_used_vars_in_expr(right, used_vars);
            }
            Expression::UnaryOperation { operand, .. } => {
                self.find_used_vars_in_expr(operand, used_vars);
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.find_used_vars_in_expr(arg, used_vars);
                }
            }
            Expression::Assignment { target, value } => {
                self.find_used_vars_in_expr(target, used_vars);
                self.find_used_vars_in_expr(value, used_vars);
            }
            Expression::TernaryIf { condition, then_expr, else_expr } => {
                self.find_used_vars_in_expr(condition, used_vars);
                self.find_used_vars_in_expr(then_expr, used_vars);
                self.find_used_vars_in_expr(else_expr, used_vars);
            }
            Expression::Cast { expr, .. } => {
                self.find_used_vars_in_expr(expr, used_vars);
            }
            Expression::ArrayAccess { array, index } => {
                self.find_used_vars_in_expr(array, used_vars);
                self.find_used_vars_in_expr(index, used_vars);
            }
            Expression::StructFieldAccess { object, .. } => {
                self.find_used_vars_in_expr(object, used_vars);
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                self.find_used_vars_in_expr(pointer, used_vars);
            }
            Expression::IntegerLiteral(_) | Expression::StringLiteral(_) | 
            Expression::CharLiteral(_) | Expression::SizeOf { .. } => {
                // These don't use variables
            }
        }
    }
} 