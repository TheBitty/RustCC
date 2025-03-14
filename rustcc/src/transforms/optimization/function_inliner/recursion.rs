use crate::parser::ast::{Expression, Function, Statement};

/// Methods for detecting recursive function calls
pub struct RecursionDetector;

impl RecursionDetector {
    /// Check if a function is recursive (calls itself directly)
    pub fn is_recursive(function: &Function) -> bool {
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
