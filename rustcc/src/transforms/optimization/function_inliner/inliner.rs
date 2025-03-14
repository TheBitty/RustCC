use super::variable_renamer::VariableRenamer;
use crate::parser::ast::{Expression, Function, Statement};

/// Handles the actual inlining of function calls
pub struct Inliner;

impl Inliner {
    /// Inline function calls in a list of statements
    pub fn inline_function_calls(statements: &mut Vec<Statement>, inline_candidates: &[&Function]) {
        // Process each statement in the block to find expressions containing function calls
        for i in 0..statements.len() {
            match &mut statements[i] {
                Statement::ExpressionStatement(expr) => {
                    Self::inline_function_calls_in_expr(expr, inline_candidates);
                }
                Statement::Return(expr) => {
                    Self::inline_function_calls_in_expr(expr, inline_candidates);
                }
                Statement::Block(block_statements) => {
                    Self::inline_function_calls(block_statements, inline_candidates);
                }
                Statement::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    Self::inline_function_calls_in_expr(condition, inline_candidates);

                    match then_block.as_mut() {
                        Statement::Block(statements) => {
                            Self::inline_function_calls(statements, inline_candidates);
                        }
                        _ => {
                            // Handle single statement
                            Self::inline_function_calls_in_statement(
                                then_block.as_mut(),
                                inline_candidates,
                            );
                        }
                    }

                    if let Some(else_stmt) = else_block {
                        match else_stmt.as_mut() {
                            Statement::Block(statements) => {
                                Self::inline_function_calls(statements, inline_candidates);
                            }
                            _ => {
                                // Handle single statement
                                Self::inline_function_calls_in_statement(
                                    else_stmt.as_mut(),
                                    inline_candidates,
                                );
                            }
                        }
                    }
                }
                Statement::While { condition, body } => {
                    Self::inline_function_calls_in_expr(condition, inline_candidates);
                    match body.as_mut() {
                        Statement::Block(block_statements) => {
                            Self::inline_function_calls(block_statements, inline_candidates);
                        }
                        _ => {
                            // Handle single statement
                            Self::inline_function_calls_in_statement(
                                body.as_mut(),
                                inline_candidates,
                            );
                        }
                    }
                }
                Statement::DoWhile { body, condition } => {
                    Self::inline_function_calls_in_expr(condition, inline_candidates);
                    match body.as_mut() {
                        Statement::Block(block_statements) => {
                            Self::inline_function_calls(block_statements, inline_candidates);
                        }
                        _ => {
                            // Handle single statement
                            Self::inline_function_calls_in_statement(
                                body.as_mut(),
                                inline_candidates,
                            );
                        }
                    }
                }
                Statement::For {
                    initializer,
                    condition,
                    increment,
                    body,
                } => {
                    if let Some(init) = initializer {
                        if let Statement::Block(block_statements) = init.as_mut() {
                            Self::inline_function_calls(block_statements, inline_candidates);
                        }
                    }

                    if let Some(cond) = condition {
                        Self::inline_function_calls_in_expr(cond, inline_candidates);
                    }

                    if let Some(inc) = increment {
                        Self::inline_function_calls_in_expr(inc, inline_candidates);
                    }

                    match body.as_mut() {
                        Statement::Block(block_statements) => {
                            Self::inline_function_calls(block_statements, inline_candidates);
                        }
                        _ => {
                            // Handle single statement
                            Self::inline_function_calls_in_statement(
                                body.as_mut(),
                                inline_candidates,
                            );
                        }
                    }
                }
                Statement::Switch { expression, cases } => {
                    Self::inline_function_calls_in_expr(expression, inline_candidates);
                    for case in cases {
                        Self::inline_function_calls(&mut case.statements, inline_candidates);
                    }
                }
                Statement::VariableDeclaration { initializer, .. } => {
                    Self::inline_function_calls_in_expr(initializer, inline_candidates);
                }
                Statement::ArrayDeclaration {
                    initializer, size, ..
                } => {
                    Self::inline_function_calls_in_expr(initializer, inline_candidates);
                    if let Some(size_expr) = size {
                        Self::inline_function_calls_in_expr(size_expr, inline_candidates);
                    }
                }
                // No function calls in break/continue
                Statement::Break | Statement::Continue => {}
            }
        }

        // Second pass to actually perform inlining at the statement level
        let mut i = 0;
        while i < statements.len() {
            // Create a clone of the statement for analysis
            let stmt_clone = statements[i].clone();

            // Check if this is a function call that should be inlined
            if let Statement::ExpressionStatement(Expression::FunctionCall {
                ref name,
                arguments: _,
            }) = stmt_clone
            {
                if let Some(function_to_inline) = inline_candidates.iter().find(|f| &f.name == name)
                {
                    // Found a function to inline, remove the original statement
                    let original_statement = statements.remove(i);

                    // Get the name and args from the removed statement
                    let (function_name, args) =
                        if let Statement::ExpressionStatement(Expression::FunctionCall {
                            name,
                            arguments,
                        }) = original_statement
                        {
                            (name, arguments)
                        } else {
                            // This should never happen since we just checked the type
                            (String::new(), Vec::new())
                        };

                    println!("Inlining function call to {}", function_name);

                    // Create inlined statements
                    let mut inlined_statements = Vec::new();

                    // Create a variable name prefix to avoid name collisions
                    let prefix = format!("__inline_{}_", function_name);

                    // 1. Declare parameters and assign arguments
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
                            is_global: false,
                        };

                        inlined_statements.push(param_var);
                    }

                    // 2. Check if we need a return value variable
                    let has_return = function_to_inline
                        .body
                        .iter()
                        .any(|stmt| matches!(stmt, Statement::Return(_)));

                    let return_var_name = format!("{}return_val", prefix);
                    if has_return {
                        // Declare the return variable
                        inlined_statements.push(Statement::VariableDeclaration {
                            name: return_var_name.clone(),
                            data_type: Some(function_to_inline.return_type.clone()),
                            initializer: Expression::IntegerLiteral(0), // Default initialization
                            is_global: false,
                        });
                    }

                    // 3. Process the function body, handling returns
                    let mut _had_early_return = false;
                    let mut processed_body = Vec::new();

                    for statement in &function_to_inline.body {
                        if let Statement::Return(expr) = statement {
                            // Convert return to an assignment to the return variable
                            if has_return {
                                let mut expr_clone = expr.clone();
                                VariableRenamer::rename_variables_in_expr(
                                    &mut expr_clone,
                                    &prefix,
                                    &function_to_inline.parameters,
                                );
                                processed_body.push(Statement::ExpressionStatement(
                                    Expression::Assignment {
                                        target: Box::new(Expression::Variable(
                                            return_var_name.clone(),
                                        )),
                                        value: Box::new(expr_clone),
                                    },
                                ));
                            }
                            _had_early_return = true;
                            break;
                        } else {
                            // Clone and rename variables in the statement
                            let mut stmt_clone = statement.clone();
                            VariableRenamer::rename_variables_in_statement(
                                &mut stmt_clone,
                                &prefix,
                                &function_to_inline.parameters,
                            );
                            processed_body.push(stmt_clone);
                        }
                    }

                    // Add the processed body to the inlined statements
                    inlined_statements.extend(processed_body);

                    // Insert all inlined statements
                    for (idx, stmt) in inlined_statements.into_iter().enumerate() {
                        statements.insert(i + idx, stmt);
                    }

                    // Don't increment i since we need to process the newly inserted statements
                    continue;
                }
            }

            // If it wasn't a function call or wasn't inlined, move to the next statement
            i += 1;
        }
    }

    /// Inline function calls in a statement
    fn inline_function_calls_in_statement(
        statement: &mut Statement,
        inline_candidates: &[&Function],
    ) {
        match statement {
            Statement::ExpressionStatement(expr) => {
                Self::inline_function_calls_in_expr(expr, inline_candidates);
            }
            Statement::Return(expr) => {
                Self::inline_function_calls_in_expr(expr, inline_candidates);
            }
            Statement::Block(statements) => {
                Self::inline_function_calls(statements, inline_candidates);
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                Self::inline_function_calls_in_expr(condition, inline_candidates);
                Self::inline_function_calls_in_statement(then_block, inline_candidates);
                if let Some(else_stmt) = else_block {
                    Self::inline_function_calls_in_statement(else_stmt, inline_candidates);
                }
            }
            Statement::While { condition, body } => {
                Self::inline_function_calls_in_expr(condition, inline_candidates);
                Self::inline_function_calls_in_statement(body, inline_candidates);
            }
            Statement::DoWhile { body, condition } => {
                Self::inline_function_calls_in_statement(body, inline_candidates);
                Self::inline_function_calls_in_expr(condition, inline_candidates);
            }
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(init) = initializer {
                    Self::inline_function_calls_in_statement(init, inline_candidates);
                }
                if let Some(cond) = condition {
                    Self::inline_function_calls_in_expr(cond, inline_candidates);
                }
                if let Some(inc) = increment {
                    Self::inline_function_calls_in_expr(inc, inline_candidates);
                }
                Self::inline_function_calls_in_statement(body, inline_candidates);
            }
            Statement::Switch { expression, cases } => {
                Self::inline_function_calls_in_expr(expression, inline_candidates);
                for case in cases {
                    for stmt in &mut case.statements {
                        Self::inline_function_calls_in_statement(stmt, inline_candidates);
                    }
                }
            }
            Statement::VariableDeclaration { initializer, .. } => {
                Self::inline_function_calls_in_expr(initializer, inline_candidates);
            }
            Statement::ArrayDeclaration {
                initializer, size, ..
            } => {
                Self::inline_function_calls_in_expr(initializer, inline_candidates);
                if let Some(size_expr) = size {
                    Self::inline_function_calls_in_expr(size_expr, inline_candidates);
                }
            }
            // No function calls in break/continue
            Statement::Break | Statement::Continue => {}
        }
    }

    /// Inline function calls in an expression
    fn inline_function_calls_in_expr(expr: &mut Expression, inline_candidates: &[&Function]) {
        match expr {
            Expression::FunctionCall { arguments, .. } => {
                // First inline any function calls in the arguments
                for arg in arguments {
                    Self::inline_function_calls_in_expr(arg, inline_candidates);
                }
                // The actual inlining of the function call is done in perform_inlining_at_statement
            }
            Expression::BinaryOperation { left, right, .. } => {
                Self::inline_function_calls_in_expr(left, inline_candidates);
                Self::inline_function_calls_in_expr(right, inline_candidates);
            }
            Expression::UnaryOperation { operand, .. } => {
                Self::inline_function_calls_in_expr(operand, inline_candidates);
            }
            Expression::Assignment { target, value } => {
                Self::inline_function_calls_in_expr(target, inline_candidates);
                Self::inline_function_calls_in_expr(value, inline_candidates);
            }
            Expression::TernaryIf {
                condition,
                then_expr,
                else_expr,
            } => {
                Self::inline_function_calls_in_expr(condition, inline_candidates);
                Self::inline_function_calls_in_expr(then_expr, inline_candidates);
                Self::inline_function_calls_in_expr(else_expr, inline_candidates);
            }
            Expression::Cast {
                expr: inner_expr, ..
            } => {
                Self::inline_function_calls_in_expr(inner_expr, inline_candidates);
            }
            Expression::ArrayAccess { array, index } => {
                Self::inline_function_calls_in_expr(array, inline_candidates);
                Self::inline_function_calls_in_expr(index, inline_candidates);
            }
            Expression::StructFieldAccess { object, .. } => {
                Self::inline_function_calls_in_expr(object, inline_candidates);
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                Self::inline_function_calls_in_expr(pointer, inline_candidates);
            }
            Expression::SizeOf(expr) => {
                Self::inline_function_calls_in_expr(expr, inline_candidates);
            }
            Expression::ArrayLiteral(elements) => {
                for element in elements {
                    Self::inline_function_calls_in_expr(element, inline_candidates);
                }
            }
            // Other expression types don't contain function calls
            _ => {}
        }
    }
}
