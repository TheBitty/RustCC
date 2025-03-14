use crate::parser::ast::{Expression, Program, Statement};
use crate::transforms::Transform;
use rand::{thread_rng, Rng};

/// String Encryption Obfuscation
/// Encrypts string literals to make them harder to identify
pub struct StringEncryptor;

impl Transform for StringEncryptor {
    fn apply(&self, program: &mut Program) -> std::result::Result<(), String> {
        let mut rng = thread_rng();

        for function in &mut program.functions {
            self.encrypt_strings_in_statements(&mut function.body, &mut rng);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "String Encryptor"
    }
}

impl StringEncryptor {
    // Recursively encrypt all string literals in statements
    fn encrypt_strings_in_statements(&self, statements: &mut Vec<Statement>, rng: &mut impl Rng) {
        for statement in statements {
            match statement {
                Statement::Return(expr) => {
                    *expr = self.encrypt_strings_in_expression(expr.clone(), rng);
                }
                Statement::VariableDeclaration { initializer, .. } => {
                    *initializer = self.encrypt_strings_in_expression(initializer.clone(), rng);
                }
                Statement::ExpressionStatement(expr) => {
                    *expr = self.encrypt_strings_in_expression(expr.clone(), rng);
                }
                Statement::Block(stmts) => {
                    self.encrypt_strings_in_statements(stmts, rng);
                }
                Statement::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    *condition = self.encrypt_strings_in_expression(condition.clone(), rng);

                    if let Statement::Block(stmts) = then_block.as_mut() {
                        self.encrypt_strings_in_statements(stmts, rng);
                    } else {
                        // Handle non-block then statements
                        let new_then =
                            self.encrypt_strings_in_statement(then_block.as_ref().clone(), rng);
                        *then_block = Box::new(new_then);
                    }

                    if let Some(else_stmt) = else_block {
                        if let Statement::Block(stmts) = else_stmt.as_mut() {
                            self.encrypt_strings_in_statements(stmts, rng);
                        } else {
                            // Handle non-block else statements
                            let new_else =
                                self.encrypt_strings_in_statement(else_stmt.as_ref().clone(), rng);
                            *else_block = Some(Box::new(new_else));
                        }
                    }
                }
                Statement::While { condition, body } => {
                    *condition = self.encrypt_strings_in_expression(condition.clone(), rng);

                    let new_body = self.encrypt_strings_in_statement(body.as_ref().clone(), rng);
                    *body = Box::new(new_body);
                }
                Statement::For {
                    condition,
                    increment,
                    body,
                    ..
                } => {
                    if let Some(cond) = condition {
                        *cond = self.encrypt_strings_in_expression(cond.clone(), rng);
                    }

                    if let Some(inc) = increment {
                        *inc = self.encrypt_strings_in_expression(inc.clone(), rng);
                    }

                    let new_body = self.encrypt_strings_in_statement(body.as_ref().clone(), rng);
                    *body = Box::new(new_body);
                }
                Statement::DoWhile { body, condition } => {
                    *condition = self.encrypt_strings_in_expression(condition.clone(), rng);

                    let new_body = self.encrypt_strings_in_statement(body.as_ref().clone(), rng);
                    *body = Box::new(new_body);
                }
                Statement::Switch { expression, cases } => {
                    *expression = self.encrypt_strings_in_expression(expression.clone(), rng);

                    for case in cases {
                        if let Some(value) = &mut case.value {
                            *value = self.encrypt_strings_in_expression(value.clone(), rng);
                        }

                        self.encrypt_strings_in_statements(&mut case.statements, rng);
                    }
                }
                _ => {} // Other statement types may not contain expressions
            }
        }
    }

    // Process a single statement
    fn encrypt_strings_in_statement(&self, statement: Statement, rng: &mut impl Rng) -> Statement {
        match statement {
            Statement::Block(mut stmts) => {
                self.encrypt_strings_in_statements(&mut stmts, rng);
                Statement::Block(stmts)
            }
            Statement::Return(expr) => {
                Statement::Return(self.encrypt_strings_in_expression(expr, rng))
            }
            Statement::VariableDeclaration {
                name,
                data_type,
                initializer,
                is_global,
            } => Statement::VariableDeclaration {
                name,
                data_type,
                initializer: self.encrypt_strings_in_expression(initializer, rng),
                is_global,
            },
            Statement::ExpressionStatement(expr) => {
                Statement::ExpressionStatement(self.encrypt_strings_in_expression(expr, rng))
            }
            // Other statement types would need similar handling
            _ => statement,
        }
    }

    // Find and encrypt string literals in expressions
    #[allow(clippy::only_used_in_recursion)]
    fn encrypt_strings_in_expression(&self, expr: Expression, rng: &mut impl Rng) -> Expression {
        match expr {
            Expression::StringLiteral(s) => {
                // Apply XOR encryption on the string
                let key = rng.gen::<u8>() as char;
                let encrypted: String = s
                    .chars()
                    .map(|c| (c as u8 ^ key as u8) as char)
                    .collect();

                // We'd need to modify the compiler to handle this properly at runtime
                // For now we're just replacing the string with a placeholder
                // In a real implementation, we'd insert decryption code
                Expression::StringLiteral(format!("ENCRYPTED:{}:{}", key as u8, encrypted))
            }
            Expression::BinaryOperation {
                left,
                operator,
                right,
            } => Expression::BinaryOperation {
                left: Box::new(self.encrypt_strings_in_expression(*left, rng)),
                operator,
                right: Box::new(self.encrypt_strings_in_expression(*right, rng)),
            },
            Expression::UnaryOperation { operator, operand } => Expression::UnaryOperation {
                operator,
                operand: Box::new(self.encrypt_strings_in_expression(*operand, rng)),
            },
            Expression::FunctionCall {
                name,
                mut arguments,
            } => {
                for arg in &mut arguments {
                    *arg = self.encrypt_strings_in_expression(arg.clone(), rng);
                }

                Expression::FunctionCall { name, arguments }
            }
            Expression::Assignment { target, value } => Expression::Assignment {
                target: Box::new(self.encrypt_strings_in_expression(*target, rng)),
                value: Box::new(self.encrypt_strings_in_expression(*value, rng)),
            },
            Expression::TernaryIf {
                condition,
                then_expr,
                else_expr,
            } => Expression::TernaryIf {
                condition: Box::new(self.encrypt_strings_in_expression(*condition, rng)),
                then_expr: Box::new(self.encrypt_strings_in_expression(*then_expr, rng)),
                else_expr: Box::new(self.encrypt_strings_in_expression(*else_expr, rng)),
            },
            Expression::Cast { target_type, expr } => Expression::Cast {
                target_type,
                expr: Box::new(self.encrypt_strings_in_expression(*expr, rng)),
            },
            Expression::ArrayAccess { array, index } => Expression::ArrayAccess {
                array: Box::new(self.encrypt_strings_in_expression(*array, rng)),
                index: Box::new(self.encrypt_strings_in_expression(*index, rng)),
            },
            Expression::StructFieldAccess { object, field } => Expression::StructFieldAccess {
                object: Box::new(self.encrypt_strings_in_expression(*object, rng)),
                field,
            },
            Expression::PointerFieldAccess { pointer, field } => Expression::PointerFieldAccess {
                pointer: Box::new(self.encrypt_strings_in_expression(*pointer, rng)),
                field,
            },
            // Other expression types may not contain nested expressions
            _ => expr,
        }
    }
} 