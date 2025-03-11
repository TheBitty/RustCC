use std::collections::HashMap;
use crate::parser::ast::{Program, Function, Statement, Expression};

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
        let main_exists = program.functions.iter()
            .any(|f| f.name == "main");
        
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
        let has_return = function.body.iter()
            .any(|stmt| matches!(stmt, Statement::Return(_)));
        
        if !has_return {
            return Err(format!("Function '{}' must have a return statement", function.name));
        }

        Ok(())
    }

    fn analyze_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Return(expr) => self.analyze_expression(expr),
            Statement::VariableDeclaration { name, initializer } => {
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
        }
    }

    fn analyze_expression(&self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::IntegerLiteral(_) => Ok(()),
            Expression::BinaryOperation { left, right, .. } => {
                self.analyze_expression(left)?;
                self.analyze_expression(right)
            }
            Expression::Variable(name) => {
                if !self.variables.contains_key(name) {
                    Err(format!("Variable '{}' is used before being defined", name))
                } else {
                    Ok(())
                }
            }
        }
    }
} 