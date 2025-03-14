use crate::parser::ast::{BinaryOp, Expression, Program, Statement, UnaryOp};
use crate::transforms::Transform;

/// Constant Folding Optimization
/// Evaluates constant expressions at compile time
pub struct ConstantFolder;

impl Transform for ConstantFolder {
    fn apply(&self, program: &mut Program) -> std::result::Result<(), String> {
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
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
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
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
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
            Expression::BinaryOperation {
                left,
                operator,
                right,
            } => {
                // Recursively fold the operands
                let folded_left = self.fold_expression(left);
                let folded_right = self.fold_expression(right);

                // Check if both operands are now constants
                if let (
                    Expression::IntegerLiteral(left_val),
                    Expression::IntegerLiteral(right_val),
                ) = (&folded_left, &folded_right)
                {
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
                let folded_args = arguments
                    .iter()
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
