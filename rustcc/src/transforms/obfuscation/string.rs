use crate::parser::ast::{Expression, Function, Program, Statement, Type};
use crate::transforms::Transform;
use rand::{thread_rng, Rng};

/// String Encryption Obfuscation
/// Encrypts string literals to make them harder to identify
pub struct StringEncryptor;

impl Transform for StringEncryptor {
    fn apply(&self, program: &mut Program) -> std::result::Result<(), String> {
        let mut rng = thread_rng();
        let mut encrypted_strings = Vec::new();
        let mut has_decrypt_function = false;

        // Check if the decrypt function already exists
        for function in &program.functions {
            if function.name == "__rustcc_decrypt_string" {
                has_decrypt_function = true;
                break;
            }
        }

        // Process all functions to encrypt strings
        for function in &mut program.functions {
            self.encrypt_strings_in_statements(&mut function.body, &mut rng, &mut encrypted_strings);
        }

        // If we encrypted any strings, add the decryption function
        if !encrypted_strings.is_empty() && !has_decrypt_function {
            program.functions.push(self.create_decrypt_function());
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "String Encryptor"
    }
}

impl StringEncryptor {
    // Create a decryption function that will be added to the program
    fn create_decrypt_function(&self) -> Function {
        // Create a simple XOR decryption function
        let decrypt_body = vec![
            // char* __rustcc_decrypt_string(const char* encrypted, unsigned char key)
            // {
            //     static char buffer[1024];  // Static buffer for simplicity
            //     int i = 0;
            //     while (encrypted[i] != '\0') {
            //         buffer[i] = encrypted[i] ^ key;
            //         i++;
            //     }
            //     buffer[i] = '\0';
            //     return buffer;
            // }
            Statement::VariableDeclaration {
                name: "buffer".to_string(),
                data_type: Some(Type::Array(Box::new(Type::Char), Some(1024))),
                initializer: Expression::ArrayLiteral(vec![Expression::CharLiteral('\0')]),
                is_global: false,
                alignment: None,
            },
            Statement::VariableDeclaration {
                name: "i".to_string(),
                data_type: Some(Type::Int),
                initializer: Expression::IntegerLiteral(0),
                is_global: false,
                alignment: None,
            },
            Statement::While {
                condition: Expression::BinaryOperation {
                    left: Box::new(Expression::UnaryOperation {
                        operator: crate::parser::ast::OperatorType::Unary(crate::parser::ast::UnaryOp::Dereference),
                        operand: Box::new(Expression::BinaryOperation {
                            left: Box::new(Expression::Variable("encrypted".to_string())),
                            operator: crate::parser::ast::BinaryOp::Add,
                            right: Box::new(Expression::Variable("i".to_string())),
                        }),
                    }),
                    operator: crate::parser::ast::BinaryOp::NotEqual,
                    right: Box::new(Expression::CharLiteral('\0')),
                },
                body: Box::new(Statement::Block(vec![
                    // buffer[i] = encrypted[i] ^ key;
                    Statement::ExpressionStatement(Expression::Assignment {
                        target: Box::new(Expression::ArrayAccess {
                            array: Box::new(Expression::Variable("buffer".to_string())),
                            index: Box::new(Expression::Variable("i".to_string())),
                        }),
                        value: Box::new(Expression::BinaryOperation {
                            left: Box::new(Expression::UnaryOperation {
                                operator: crate::parser::ast::OperatorType::Unary(crate::parser::ast::UnaryOp::Dereference),
                                operand: Box::new(Expression::BinaryOperation {
                                    left: Box::new(Expression::Variable("encrypted".to_string())),
                                    operator: crate::parser::ast::BinaryOp::Add,
                                    right: Box::new(Expression::Variable("i".to_string())),
                                }),
                            }),
                            operator: crate::parser::ast::BinaryOp::BitwiseXor,
                            right: Box::new(Expression::Variable("key".to_string())),
                        }),
                    }),
                    // i++;
                    Statement::ExpressionStatement(Expression::UnaryOperation {
                        operator: crate::parser::ast::OperatorType::Unary(crate::parser::ast::UnaryOp::PostIncrement),
                        operand: Box::new(Expression::Variable("i".to_string())),
                    }),
                ])),
            },
            // buffer[i] = '\0';
            Statement::ExpressionStatement(Expression::Assignment {
                target: Box::new(Expression::ArrayAccess {
                    array: Box::new(Expression::Variable("buffer".to_string())),
                    index: Box::new(Expression::Variable("i".to_string())),
                }),
                value: Box::new(Expression::CharLiteral('\0')),
            }),
            // return buffer;
            Statement::Return(Expression::Variable("buffer".to_string())),
        ];

        Function {
            name: "__rustcc_decrypt_string".to_string(),
            return_type: Type::Pointer(Box::new(Type::Char)),
            parameters: vec![
                crate::parser::ast::FunctionParameter {
                    name: "encrypted".to_string(),
                    data_type: Type::Pointer(Box::new(Type::Char)),
                },
                crate::parser::ast::FunctionParameter {
                    name: "key".to_string(),
                    data_type: Type::UnsignedChar,
                },
            ],
            body: decrypt_body,
            is_variadic: false,
            is_external: false,
        }
    }

    // Recursively encrypt all string literals in statements
    fn encrypt_strings_in_statements(
        &self, 
        statements: &mut Vec<Statement>, 
        rng: &mut impl Rng,
        encrypted_strings: &mut Vec<(String, u8)>
    ) {
        for statement in statements {
            match statement {
                Statement::Return(expr) => {
                    *expr = self.encrypt_strings_in_expression(expr.clone(), rng, encrypted_strings);
                }
                Statement::VariableDeclaration { initializer, .. } => {
                    *initializer = self.encrypt_strings_in_expression(initializer.clone(), rng, encrypted_strings);
                }
                Statement::ExpressionStatement(expr) => {
                    *expr = self.encrypt_strings_in_expression(expr.clone(), rng, encrypted_strings);
                }
                Statement::Block(stmts) => {
                    self.encrypt_strings_in_statements(stmts, rng, encrypted_strings);
                }
                Statement::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    *condition = self.encrypt_strings_in_expression(condition.clone(), rng, encrypted_strings);

                    if let Statement::Block(stmts) = then_block.as_mut() {
                        self.encrypt_strings_in_statements(stmts, rng, encrypted_strings);
                    } else {
                        // Handle non-block then statements
                        let new_then =
                            self.encrypt_strings_in_statement(then_block.as_ref().clone(), rng, encrypted_strings);
                        *then_block = Box::new(new_then);
                    }

                    if let Some(else_stmt) = else_block {
                        if let Statement::Block(stmts) = else_stmt.as_mut() {
                            self.encrypt_strings_in_statements(stmts, rng, encrypted_strings);
                        } else {
                            // Handle non-block else statements
                            let new_else =
                                self.encrypt_strings_in_statement(else_stmt.as_ref().clone(), rng, encrypted_strings);
                            *else_block = Some(Box::new(new_else));
                        }
                    }
                }
                Statement::While { condition, body } => {
                    *condition = self.encrypt_strings_in_expression(condition.clone(), rng, encrypted_strings);

                    let new_body = self.encrypt_strings_in_statement(body.as_ref().clone(), rng, encrypted_strings);
                    *body = Box::new(new_body);
                }
                Statement::For {
                    condition,
                    increment,
                    body,
                    ..
                } => {
                    if let Some(cond) = condition {
                        *cond = self.encrypt_strings_in_expression(cond.clone(), rng, encrypted_strings);
                    }

                    if let Some(inc) = increment {
                        *inc = self.encrypt_strings_in_expression(inc.clone(), rng, encrypted_strings);
                    }

                    let new_body = self.encrypt_strings_in_statement(body.as_ref().clone(), rng, encrypted_strings);
                    *body = Box::new(new_body);
                }
                Statement::DoWhile { body, condition } => {
                    *condition = self.encrypt_strings_in_expression(condition.clone(), rng, encrypted_strings);

                    let new_body = self.encrypt_strings_in_statement(body.as_ref().clone(), rng, encrypted_strings);
                    *body = Box::new(new_body);
                }
                Statement::Switch { expression, cases } => {
                    *expression = self.encrypt_strings_in_expression(expression.clone(), rng, encrypted_strings);

                    for case in cases {
                        if let Some(value) = &mut case.value {
                            *value = self.encrypt_strings_in_expression(value.clone(), rng, encrypted_strings);
                        }

                        self.encrypt_strings_in_statements(&mut case.statements, rng, encrypted_strings);
                    }
                }
                _ => {} // Other statement types may not contain expressions
            }
        }
    }

    // Process a single statement
    fn encrypt_strings_in_statement(
        &self, 
        statement: Statement, 
        rng: &mut impl Rng,
        encrypted_strings: &mut Vec<(String, u8)>
    ) -> Statement {
        match statement {
            Statement::Block(mut stmts) => {
                self.encrypt_strings_in_statements(&mut stmts, rng, encrypted_strings);
                Statement::Block(stmts)
            }
            Statement::Return(expr) => {
                Statement::Return(self.encrypt_strings_in_expression(expr, rng, encrypted_strings))
            }
            Statement::VariableDeclaration {
                name,
                data_type,
                initializer,
                is_global,
                alignment,
            } => Statement::VariableDeclaration {
                name: name.clone(),
                data_type: data_type.clone(),
                initializer: self.encrypt_strings_in_expression(initializer, rng, encrypted_strings),
                is_global,
                alignment: alignment.clone(),
            },
            Statement::ExpressionStatement(expr) => {
                Statement::ExpressionStatement(self.encrypt_strings_in_expression(expr, rng, encrypted_strings))
            }
            // Other statement types would need similar handling
            _ => statement,
        }
    }

    // Find and encrypt string literals in expressions
    #[allow(clippy::only_used_in_recursion)]
    fn encrypt_strings_in_expression(
        &self, 
        expr: Expression, 
        rng: &mut impl Rng,
        encrypted_strings: &mut Vec<(String, u8)>
    ) -> Expression {
        match expr {
            Expression::StringLiteral(s) => {
                // Skip empty strings
                if s.is_empty() {
                    return Expression::StringLiteral(s);
                }
                
                // Apply XOR encryption on the string
                let key = rng.gen::<u8>();
                let encrypted: String = s.chars().map(|c| (c as u8 ^ key) as char).collect();
                
                // Store the encrypted string and key for later use
                encrypted_strings.push((encrypted.clone(), key));
                
                // Replace with a call to the decrypt function
                Expression::FunctionCall {
                    name: "__rustcc_decrypt_string".to_string(),
                    arguments: vec![
                        // Pass the encrypted string literal
                        Expression::StringLiteral(encrypted),
                        // Pass the encryption key
                        Expression::IntegerLiteral(key as i32),
                    ],
                }
            }
            Expression::BinaryOperation {
                left,
                operator,
                right,
            } => Expression::BinaryOperation {
                left: Box::new(self.encrypt_strings_in_expression(*left, rng, encrypted_strings)),
                operator,
                right: Box::new(self.encrypt_strings_in_expression(*right, rng, encrypted_strings)),
            },
            Expression::UnaryOperation { operator, operand } => Expression::UnaryOperation {
                operator,
                operand: Box::new(self.encrypt_strings_in_expression(*operand, rng, encrypted_strings)),
            },
            Expression::FunctionCall {
                name,
                mut arguments,
            } => {
                // Don't encrypt strings in calls to our decrypt function
                if name != "__rustcc_decrypt_string" {
                    for arg in &mut arguments {
                        *arg = self.encrypt_strings_in_expression(arg.clone(), rng, encrypted_strings);
                    }
                }

                Expression::FunctionCall { name, arguments }
            }
            Expression::Assignment { target, value } => Expression::Assignment {
                target: Box::new(self.encrypt_strings_in_expression(*target, rng, encrypted_strings)),
                value: Box::new(self.encrypt_strings_in_expression(*value, rng, encrypted_strings)),
            },
            Expression::TernaryIf {
                condition,
                then_expr,
                else_expr,
            } => Expression::TernaryIf {
                condition: Box::new(self.encrypt_strings_in_expression(*condition, rng, encrypted_strings)),
                then_expr: Box::new(self.encrypt_strings_in_expression(*then_expr, rng, encrypted_strings)),
                else_expr: Box::new(self.encrypt_strings_in_expression(*else_expr, rng, encrypted_strings)),
            },
            Expression::Cast { target_type, expr } => Expression::Cast {
                target_type,
                expr: Box::new(self.encrypt_strings_in_expression(*expr, rng, encrypted_strings)),
            },
            Expression::ArrayAccess { array, index } => Expression::ArrayAccess {
                array: Box::new(self.encrypt_strings_in_expression(*array, rng, encrypted_strings)),
                index: Box::new(self.encrypt_strings_in_expression(*index, rng, encrypted_strings)),
            },
            Expression::StructFieldAccess { object, field } => Expression::StructFieldAccess {
                object: Box::new(self.encrypt_strings_in_expression(*object, rng, encrypted_strings)),
                field,
            },
            Expression::PointerFieldAccess { pointer, field } => Expression::PointerFieldAccess {
                pointer: Box::new(self.encrypt_strings_in_expression(*pointer, rng, encrypted_strings)),
                field,
            },
            // Other expression types may not contain nested expressions
            _ => expr,
        }
    }
}
