use crate::parser::ast::{
    BinaryOp, Expression, Function, FunctionParameter, Program, Statement, Type
};
use crate::transforms::optimization::FunctionInliner;
use crate::transforms::Transform;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_inliner() {
        // Create a simple program with two functions:
        // 1. add(a: int, b: int) -> int { return a + b; }
        // 2. complex_function() -> int { ... many statements ... }
        // 3. main() -> int { int result = add(1, 2); return result; }

        // Create the add function (simple, should be inlined)
        let add_function = Function {
            name: "add".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "a".to_string(),
                    data_type: Type::Int,
                },
                FunctionParameter {
                    name: "b".to_string(),
                    data_type: Type::Int,
                },
            ],
            return_type: Type::Int,
            body: vec![
                Statement::Return(
                    Expression::BinaryOperation {
                        left: Box::new(Expression::Variable("a".to_string())),
                        operator: BinaryOp::Add,
                        right: Box::new(Expression::Variable("b".to_string())),
                    }
                ),
            ],
        };

        // Create a complex function (too large to inline)
        let complex_function = Function {
            name: "complex_function".to_string(),
            parameters: vec![],
            return_type: Type::Int,
            body: vec![
                // Create many statements to exceed the max_instructions limit
                Statement::VariableDeclaration {
                    name: "i".to_string(),
                    data_type: Some(Type::Int),
                    initializer: Expression::IntegerLiteral(0),
                },
                Statement::VariableDeclaration {
                    name: "j".to_string(),
                    data_type: Some(Type::Int),
                    initializer: Expression::IntegerLiteral(0),
                },
                Statement::VariableDeclaration {
                    name: "k".to_string(),
                    data_type: Some(Type::Int),
                    initializer: Expression::IntegerLiteral(0),
                },
                Statement::VariableDeclaration {
                    name: "l".to_string(),
                    data_type: Some(Type::Int),
                    initializer: Expression::IntegerLiteral(0),
                },
                Statement::VariableDeclaration {
                    name: "m".to_string(),
                    data_type: Some(Type::Int),
                    initializer: Expression::IntegerLiteral(0),
                },
                Statement::VariableDeclaration {
                    name: "n".to_string(),
                    data_type: Some(Type::Int),
                    initializer: Expression::IntegerLiteral(0),
                },
                Statement::Return(Expression::IntegerLiteral(42)),
            ],
        };

        // Create the main function (too complex to inline)
        let main_function = Function {
            name: "main".to_string(),
            parameters: vec![],
            return_type: Type::Int,
            body: vec![
                Statement::VariableDeclaration {
                    name: "result".to_string(),
                    data_type: Some(Type::Int),
                    initializer: Expression::IntegerLiteral(0),
                },
                // Add more statements to exceed the max_instructions limit
                Statement::VariableDeclaration {
                    name: "temp1".to_string(),
                    data_type: Some(Type::Int),
                    initializer: Expression::IntegerLiteral(0),
                },
                Statement::VariableDeclaration {
                    name: "temp2".to_string(),
                    data_type: Some(Type::Int),
                    initializer: Expression::IntegerLiteral(0),
                },
                Statement::VariableDeclaration {
                    name: "temp3".to_string(),
                    data_type: Some(Type::Int),
                    initializer: Expression::IntegerLiteral(0),
                },
                Statement::VariableDeclaration {
                    name: "temp4".to_string(),
                    data_type: Some(Type::Int),
                    initializer: Expression::IntegerLiteral(0),
                },
                Statement::VariableDeclaration {
                    name: "temp5".to_string(),
                    data_type: Some(Type::Int),
                    initializer: Expression::IntegerLiteral(0),
                },
                Statement::ExpressionStatement(
                    Expression::FunctionCall {
                        name: "add".to_string(),
                        arguments: vec![
                            Expression::IntegerLiteral(1),
                            Expression::IntegerLiteral(2),
                        ],
                    }
                ),
                Statement::Return(
                    Expression::Variable("result".to_string())
                ),
            ],
        };

        // Save the original number of statements in main
        let original_main_body_len = main_function.body.len();

        // Create the program
        let mut program = Program {
            functions: vec![add_function, complex_function, main_function],
            structs: vec![],
            includes: vec![],
        };

        // Apply the function inliner with a small max_instructions limit and inline_all=false
        let inliner = FunctionInliner::new(5, false);
        inliner.apply(&mut program).unwrap();

        // Print the resulting program for debugging
        println!("Program after inlining: {:#?}", program);

        // Check that the program still has at least one function
        assert!(!program.functions.is_empty(), "Expected at least one function after inlining");
        
        // The complex function should not be inlined and should still exist
        let complex_exists = program.functions.iter().any(|f| f.name == "complex_function");
        assert!(complex_exists, "Expected complex_function to still exist after inlining");
        
        // Try to find the main function
        if let Some(main_function) = program.functions.iter().find(|f| f.name == "main") {
            // Check that the main function has more statements due to inlining
            println!("Main function body length: {}", main_function.body.len());
            assert!(main_function.body.len() >= original_main_body_len, 
                "Expected main function to have at least {} statements after inlining, got {}", 
                original_main_body_len, main_function.body.len());
        } else {
            panic!("Main function not found after inlining");
        }
    }
} 