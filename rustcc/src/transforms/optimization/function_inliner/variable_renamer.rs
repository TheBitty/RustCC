use crate::parser::ast::{Expression, FunctionParameter, Statement};

/// Handles variable renaming during function inlining to avoid name conflicts
pub struct VariableRenamer;

impl VariableRenamer {
    /// Rename variables in a statement with a given prefix
    pub fn rename_variables_in_statement(
        statement: &mut Statement,
        prefix: &str,
        parameters: &[FunctionParameter],
    ) {
        match statement {
            Statement::VariableDeclaration {
                name, initializer, ..
            } => {
                // Rename the variable declaration
                *name = format!("{}{}", prefix, name);
                Self::rename_variables_in_expr(initializer, prefix, parameters);
            }
            Statement::ExpressionStatement(expr) => {
                Self::rename_variables_in_expr(expr, prefix, parameters);
            }
            Statement::Return(expr) => {
                Self::rename_variables_in_expr(expr, prefix, parameters);
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    Self::rename_variables_in_statement(stmt, prefix, parameters);
                }
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                Self::rename_variables_in_expr(condition, prefix, parameters);
                Self::rename_variables_in_statement(then_block, prefix, parameters);
                if let Some(else_stmt) = else_block {
                    Self::rename_variables_in_statement(else_stmt, prefix, parameters);
                }
            }
            Statement::While { condition, body } => {
                Self::rename_variables_in_expr(condition, prefix, parameters);
                Self::rename_variables_in_statement(body, prefix, parameters);
            }
            Statement::DoWhile { body, condition } => {
                Self::rename_variables_in_statement(body, prefix, parameters);
                Self::rename_variables_in_expr(condition, prefix, parameters);
            }
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(init) = initializer {
                    Self::rename_variables_in_statement(init, prefix, parameters);
                }
                if let Some(cond) = condition {
                    Self::rename_variables_in_expr(cond, prefix, parameters);
                }
                if let Some(inc) = increment {
                    Self::rename_variables_in_expr(inc, prefix, parameters);
                }
                Self::rename_variables_in_statement(body, prefix, parameters);
            }
            Statement::Switch { expression, cases } => {
                Self::rename_variables_in_expr(expression, prefix, parameters);
                for case in cases {
                    for stmt in &mut case.statements {
                        Self::rename_variables_in_statement(stmt, prefix, parameters);
                    }
                }
            }
            Statement::ArrayDeclaration {
                name,
                initializer,
                size,
                ..
            } => {
                // Rename the array declaration
                *name = format!("{}{}", prefix, name);
                Self::rename_variables_in_expr(initializer, prefix, parameters);
                if let Some(size_expr) = size {
                    Self::rename_variables_in_expr(size_expr, prefix, parameters);
                }
            }
            // No variables to rename in break/continue
            Statement::Break | Statement::Continue => {}
        }
    }

    /// Rename variables in an expression with a given prefix
    pub fn rename_variables_in_expr(
        expr: &mut Expression,
        prefix: &str,
        parameters: &[FunctionParameter],
    ) {
        match expr {
            Expression::Variable(name) => {
                // Check if this is a parameter name that needs to be renamed
                if parameters.iter().any(|p| p.name == *name) {
                    *name = format!("{}{}", prefix, name);
                }
            }
            Expression::BinaryOperation { left, right, .. } => {
                Self::rename_variables_in_expr(left, prefix, parameters);
                Self::rename_variables_in_expr(right, prefix, parameters);
            }
            Expression::UnaryOperation { operand, .. } => {
                Self::rename_variables_in_expr(operand, prefix, parameters);
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    Self::rename_variables_in_expr(arg, prefix, parameters);
                }
            }
            Expression::Assignment { target, value } => {
                Self::rename_variables_in_expr(target, prefix, parameters);
                Self::rename_variables_in_expr(value, prefix, parameters);
            }
            Expression::TernaryIf {
                condition,
                then_expr,
                else_expr,
            } => {
                Self::rename_variables_in_expr(condition, prefix, parameters);
                Self::rename_variables_in_expr(then_expr, prefix, parameters);
                Self::rename_variables_in_expr(else_expr, prefix, parameters);
            }
            Expression::Cast {
                expr: inner_expr, ..
            } => {
                Self::rename_variables_in_expr(inner_expr, prefix, parameters);
            }
            Expression::SizeOf(expr_box) => {
                Self::rename_variables_in_expr(expr_box, prefix, parameters);
            }
            Expression::ArrayAccess { array, index } => {
                Self::rename_variables_in_expr(array, prefix, parameters);
                Self::rename_variables_in_expr(index, prefix, parameters);
            }
            Expression::ArrayLiteral(elements) => {
                for element in elements {
                    Self::rename_variables_in_expr(element, prefix, parameters);
                }
            }
            Expression::StructFieldAccess { object, .. } => {
                Self::rename_variables_in_expr(object, prefix, parameters);
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                Self::rename_variables_in_expr(pointer, prefix, parameters);
            }
            // Other expressions don't contain variables that need renaming
            _ => {}
        }
    }
}
