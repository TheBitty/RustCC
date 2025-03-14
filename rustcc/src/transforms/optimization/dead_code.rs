use crate::parser::ast::{Expression, Program, Statement};
use crate::transforms::Transform;
use std::collections::HashSet;

/// Dead Code Elimination
/// Removes variables that are never used
pub struct DeadCodeEliminator;

impl Transform for DeadCodeEliminator {
    fn apply(&self, program: &mut Program) -> std::result::Result<(), String> {
        for function in &mut program.functions {
            // Find all used variables in the function
            let used_vars = self.find_used_variables(function);

            // Remove declarations of unused variables
            function.body.retain(|stmt| match stmt {
                Statement::VariableDeclaration { name, .. } => used_vars.contains(name),
                _ => true,
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
            Statement::ArrayDeclaration { initializer, size, .. } => {
                self.find_used_vars_in_expr(initializer, used_vars);
                if let Some(size_expr) = size {
                    self.find_used_vars_in_expr(size_expr, used_vars);
                }
            }
            Statement::ExpressionStatement(expr) => {
                self.find_used_vars_in_expr(expr, used_vars);
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.find_used_vars_in_statement(stmt, used_vars);
                }
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
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
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
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

    #[allow(clippy::only_used_in_recursion)]
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
            Expression::TernaryIf {
                condition,
                then_expr,
                else_expr,
            } => {
                self.find_used_vars_in_expr(condition, used_vars);
                self.find_used_vars_in_expr(then_expr, used_vars);
                self.find_used_vars_in_expr(else_expr, used_vars);
            }
            Expression::Cast { expr, .. } => {
                self.find_used_vars_in_expr(expr, used_vars);
            }
            Expression::SizeOf(expr) => {
                self.find_used_vars_in_expr(expr, used_vars);
            }
            Expression::ArrayAccess { array, index } => {
                self.find_used_vars_in_expr(array, used_vars);
                self.find_used_vars_in_expr(index, used_vars);
            }
            Expression::ArrayLiteral(elements) => {
                for element in elements {
                    self.find_used_vars_in_expr(element, used_vars);
                }
            }
            Expression::StructFieldAccess { object, .. } => {
                self.find_used_vars_in_expr(object, used_vars);
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                self.find_used_vars_in_expr(pointer, used_vars);
            }
            _ => {
                // Integer, string and char literals don't reference variables
            }
        }
    }
}
