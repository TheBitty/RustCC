use crate::parser::ast::{Expression, Program, Statement};
use crate::transforms::Transform;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::collections::HashMap;

/// Variable Name Obfuscation
/// Replaces all variable names with random strings
pub struct VariableObfuscator;

impl Transform for VariableObfuscator {
    fn apply(&self, program: &mut Program) -> std::result::Result<(), String> {
        let mut rng = thread_rng();

        for function in &mut program.functions {
            let mut var_map: HashMap<String, String> = HashMap::new();

            // Gather all variable names from the function body
            for statement in &function.body {
                if let Statement::VariableDeclaration { name, .. } = statement {
                    if !var_map.contains_key(name) {
                        // Generate a random name with mixed case for increased confusion
                        let new_name: String = std::iter::repeat(())
                            .map(|()| {
                                let c = rng.sample(Alphanumeric) as char;
                                if rng.gen_bool(0.5) {
                                    c.to_uppercase().next().unwrap_or(c)
                                } else {
                                    c
                                }
                            })
                            .take(12) // Longer names increase confusion
                            .collect();

                        // Add prefixes that resemble internal compiler symbols
                        var_map.insert(
                            name.clone(),
                            format!("_${}_{}", rng.gen::<u32>() % 1000, new_name),
                        );
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
        "Variable Obfuscator"
    }
}

impl VariableObfuscator {
    fn obfuscate_statement(&self, statement: &mut Statement, var_map: &HashMap<String, String>) {
        match statement {
            Statement::VariableDeclaration {
                name,
                initializer,
                data_type: _,
                is_global: _,
                alignment: _,
            } => {
                if let Some(new_name) = var_map.get(name) {
                    *name = new_name.clone();
                }
                self.obfuscate_expression(initializer, var_map);
            }
            Statement::ArrayDeclaration {
                name,
                initializer,
                data_type: _,
                size: _,
                is_global: _,
                alignment: _,
            } => {
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
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
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
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
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
            // Add a catch-all for C11 features and other statements
            _ => {
                // No variable names to obfuscate in these statements
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
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
            Expression::TernaryIf {
                condition,
                then_expr,
                else_expr,
            } => {
                self.obfuscate_expression(condition, var_map);
                self.obfuscate_expression(then_expr, var_map);
                self.obfuscate_expression(else_expr, var_map);
            }
            Expression::Cast { expr, .. } => {
                self.obfuscate_expression(expr, var_map);
            }
            Expression::SizeOf(expr) => {
                self.obfuscate_expression(expr, var_map);
            }
            Expression::ArrayAccess { array, index } => {
                self.obfuscate_expression(array, var_map);
                self.obfuscate_expression(index, var_map);
            }
            Expression::ArrayLiteral(elements) => {
                for element in elements {
                    self.obfuscate_expression(element, var_map);
                }
            }
            Expression::StructFieldAccess { object, .. } => {
                self.obfuscate_expression(object, var_map);
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                self.obfuscate_expression(pointer, var_map);
            }
            // Add a catch-all for C11 features and other expressions
            _ => {
                // No variable names to obfuscate in these expressions
            }
        }
    }
}
