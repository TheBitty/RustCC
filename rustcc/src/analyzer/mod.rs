use crate::parser::ast::{Expression, Function, Program, Statement};
use std::collections::HashMap;

pub struct SemanticAnalyzer {
    variables: HashMap<String, bool>, // tracks if variables are defined
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            variables: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), String> {
        // Check if main function exists
        let main_exists = program.functions.iter().any(|f| f.name == "main");

        if !main_exists {
            return Err("Program must have a main function".to_string());
        }

        // Analyze each function
        for function in &program.functions {
            self.analyze_function(function)?;
        }

        Ok(())
    }

    fn analyze_function(&mut self, function: &Function) -> Result<(), String> {
        self.variables.clear(); // Clear variables for new function scope

        // Analyze each statement in the function body
        for statement in &function.body {
            self.analyze_statement(statement)?;
        }

        // Check that function has a return statement
        let has_return = function
            .body
            .iter()
            .any(|stmt| matches!(stmt, Statement::Return(_)));

        if !has_return {
            return Err(format!(
                "Function '{}' must have a return statement",
                function.name
            ));
        }

        Ok(())
    }

    fn analyze_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Return(expr) => self.analyze_expression(expr),
            Statement::VariableDeclaration {
                name,
                initializer,
                data_type: _,
            } => {
                // Check if variable is already defined
                if self.variables.contains_key(name) {
                    return Err(format!("Variable '{}' is already defined", name));
                }

                // Analyze the initializer expression
                self.analyze_expression(initializer)?;

                // Mark variable as defined
                self.variables.insert(name.clone(), true);
                Ok(())
            }
            Statement::ArrayDeclaration {
                name,
                initializer,
                size,
                data_type: _,
            } => {
                // Check if variable is already defined
                if self.variables.contains_key(name) {
                    return Err(format!("Array '{}' is already defined", name));
                }

                // Analyze the initializer expression
                self.analyze_expression(initializer)?;

                // Analyze the size expression if provided
                if let Some(size_expr) = size {
                    self.analyze_expression(size_expr)?;
                }

                // Mark array as defined
                self.variables.insert(name.clone(), true);
                Ok(())
            }
            Statement::ExpressionStatement(expr) => self.analyze_expression(expr),
            Statement::Block(statements) => {
                for stmt in statements {
                    self.analyze_statement(stmt)?;
                }
                Ok(())
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.analyze_expression(condition)?;
                self.analyze_statement(then_block)?;
                if let Some(else_stmt) = else_block {
                    self.analyze_statement(else_stmt)?;
                }
                Ok(())
            }
            Statement::While { condition, body } => {
                self.analyze_expression(condition)?;
                self.analyze_statement(body)
            }
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(init) = initializer {
                    self.analyze_statement(init)?;
                }
                if let Some(cond) = condition {
                    self.analyze_expression(cond)?;
                }
                if let Some(inc) = increment {
                    self.analyze_expression(inc)?;
                }
                self.analyze_statement(body)
            }
            Statement::DoWhile { body, condition } => {
                self.analyze_statement(body)?;
                self.analyze_expression(condition)
            }
            Statement::Break | Statement::Continue => Ok(()),
            Statement::Switch { expression, cases } => {
                self.analyze_expression(expression)?;
                for case in cases {
                    if let Some(value) = &case.value {
                        self.analyze_expression(value)?;
                    }
                    for stmt in &case.statements {
                        self.analyze_statement(stmt)?;
                    }
                }
                Ok(())
            }
        }
    }

    fn analyze_expression(&self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::IntegerLiteral(_) => Ok(()),
            Expression::StringLiteral(_) => Ok(()),
            Expression::CharLiteral(_) => Ok(()),
            Expression::BinaryOperation { left, right, .. } => {
                self.analyze_expression(left)?;
                self.analyze_expression(right)
            }
            Expression::UnaryOperation { operand, .. } => self.analyze_expression(operand),
            Expression::Variable(name) => {
                if !self.variables.contains_key(name) {
                    Err(format!("Variable '{}' is used before being defined", name))
                } else {
                    Ok(())
                }
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.analyze_expression(arg)?;
                }
                Ok(())
            }
            Expression::Assignment { target, value } => {
                self.analyze_expression(target)?;
                self.analyze_expression(value)
            }
            Expression::TernaryIf {
                condition,
                then_expr,
                else_expr,
            } => {
                self.analyze_expression(condition)?;
                self.analyze_expression(then_expr)?;
                self.analyze_expression(else_expr)
            }
            Expression::Cast { expr, .. } => self.analyze_expression(expr),
            Expression::SizeOf(expr) => self.analyze_expression(expr),
            Expression::ArrayAccess { array, index } => {
                self.analyze_expression(array)?;
                self.analyze_expression(index)
            }
            Expression::ArrayLiteral(elements) => {
                for element in elements {
                    self.analyze_expression(element)?;
                }
                Ok(())
            }
            Expression::StructFieldAccess { object, .. } => self.analyze_expression(object),
            Expression::PointerFieldAccess { pointer, .. } => self.analyze_expression(pointer),
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
