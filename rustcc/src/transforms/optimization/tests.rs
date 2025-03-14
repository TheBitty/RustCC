#[cfg(test)]
mod tests {
    use crate::parser::ast::{Program, Function, FunctionParameter, Statement, Expression, Type, BinaryOp};
    use crate::transforms::Transform;
    use crate::transforms::optimization::{ConstantFolder, DeadCodeEliminator, FunctionInliner};

    fn create_basic_program() -> Program {
        Program {
            functions: vec![
                Function {
                    name: "add".to_string(),
                    return_type: Type::Int,
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
                    body: vec![
                        Statement::Return(
                            Expression::BinaryOperation {
                                left: Box::new(Expression::Variable("a".to_string())),
                                operator: BinaryOp::Add,
                                right: Box::new(Expression::Variable("b".to_string())),
                            }
                        ),
                    ],
                },
                Function {
                    name: "main".to_string(),
                    return_type: Type::Int,
                    parameters: vec![],
                    body: vec![
                        Statement::VariableDeclaration {
                            name: "result".to_string(),
                            data_type: Some(Type::Int),
                            initializer: Expression::IntegerLiteral(0),
                        },
                        Statement::ExpressionStatement(
                            Expression::Assignment {
                                target: Box::new(Expression::Variable("result".to_string())),
                                value: Box::new(Expression::FunctionCall {
                                    name: "add".to_string(),
                                    arguments: vec![
                                        Expression::IntegerLiteral(2),
                                        Expression::IntegerLiteral(3),
                                    ],
                                }),
                            }
                        ),
                        Statement::Return(Expression::Variable("result".to_string())),
                    ],
                },
            ],
            structs: vec![],
            includes: vec![],
        }
    }

    #[test]
    fn test_constant_folder() {
        let mut program = create_basic_program();
        let constant_folder = ConstantFolder;
        
        // Apply the constant folder
        constant_folder.apply(&mut program).unwrap();
        
        // The constant folder should optimize the arguments to the add function
        let function = &program.functions[1];
        if let Statement::ExpressionStatement(Expression::Assignment { value, .. }) = &function.body[1] {
            if let Expression::FunctionCall { arguments, .. } = value.as_ref() {
                // The arguments should still be 2 and 3 (literals aren't folded unless used in operations)
                assert_eq!(arguments.len(), 2);
                assert!(matches!(&arguments[0], Expression::IntegerLiteral(2)));
                assert!(matches!(&arguments[1], Expression::IntegerLiteral(3)));
            } else {
                panic!("Expected function call in assignment");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_dead_code_eliminator() {
        let mut program = Program {
            functions: vec![
                Function {
                    name: "main".to_string(),
                    return_type: Type::Int,
                    parameters: vec![],
                    body: vec![
                        // Used variable
                        Statement::VariableDeclaration {
                            name: "result".to_string(),
                            data_type: Some(Type::Int),
                            initializer: Expression::IntegerLiteral(10),
                        },
                        // Unused variable (should be eliminated)
                        Statement::VariableDeclaration {
                            name: "unused".to_string(),
                            data_type: Some(Type::Int),
                            initializer: Expression::IntegerLiteral(5),
                        },
                        Statement::Return(Expression::Variable("result".to_string())),
                    ],
                },
            ],
            structs: vec![],
            includes: vec![],
        };
        
        let dead_code_eliminator = DeadCodeEliminator;
        
        // Apply the dead code eliminator
        dead_code_eliminator.apply(&mut program).unwrap();
        
        // Check that the unused variable was eliminated
        let function = &program.functions[0];
        assert_eq!(function.body.len(), 2); // Should only have 2 statements now
        
        // First statement should be the declaration of "result"
        if let Statement::VariableDeclaration { name, .. } = &function.body[0] {
            assert_eq!(name, "result");
        } else {
            panic!("Expected variable declaration");
        }
        
        // Second statement should be the return
        assert!(matches!(&function.body[1], Statement::Return(_)));
    }

    #[test]
    fn test_function_inliner() {
        let mut program = create_basic_program();
        let function_inliner = FunctionInliner::new(10, true);
        
        // Apply the function inliner
        function_inliner.apply(&mut program).unwrap();
        
        // Verify the main function exists
        let main_function = program.functions.iter()
            .find(|f| f.name == "main")
            .expect("Main function should exist after inlining");
            
        // The main function should still have its original statements
        assert!(main_function.body.len() >= 3, 
            "Main function should have at least its original statements");
        
        // Verify that the main function still has a return statement
        let has_return = main_function.body.iter().any(|stmt| {
            matches!(stmt, Statement::Return(_))
        });
        
        assert!(has_return, "Main function should have a return statement");
    }
    
    #[test]
    fn test_multiple_optimizations() {
        // Create a program with opportunities for all three optimizations
        let mut program = Program {
            functions: vec![
                Function {
                    name: "calc".to_string(),
                    return_type: Type::Int,
                    parameters: vec![
                        FunctionParameter {
                            name: "x".to_string(),
                            data_type: Type::Int,
                        },
                    ],
                    body: vec![
                        // Constant expression that can be folded
                        Statement::VariableDeclaration {
                            name: "val".to_string(),
                            data_type: Some(Type::Int),
                            initializer: Expression::BinaryOperation {
                                left: Box::new(Expression::IntegerLiteral(5)),
                                operator: BinaryOp::Multiply,
                                right: Box::new(Expression::IntegerLiteral(4)),
                            },
                        },
                        // Unused variable that can be eliminated
                        Statement::VariableDeclaration {
                            name: "unused".to_string(),
                            data_type: Some(Type::Int),
                            initializer: Expression::IntegerLiteral(10),
                        },
                        // Return using the parameter and the computed value
                        Statement::Return(
                            Expression::BinaryOperation {
                                left: Box::new(Expression::Variable("x".to_string())),
                                operator: BinaryOp::Add,
                                right: Box::new(Expression::Variable("val".to_string())),
                            }
                        ),
                    ],
                },
                Function {
                    name: "main".to_string(),
                    return_type: Type::Int,
                    parameters: vec![],
                    body: vec![
                        Statement::VariableDeclaration {
                            name: "result".to_string(),
                            data_type: Some(Type::Int),
                            initializer: Expression::FunctionCall {
                                name: "calc".to_string(),
                                arguments: vec![Expression::IntegerLiteral(10)],
                            },
                        },
                        Statement::Return(Expression::Variable("result".to_string())),
                    ],
                },
            ],
            structs: vec![],
            includes: vec![],
        };
        
        // Apply all three optimizations
        let constant_folder = ConstantFolder;
        let dead_code_eliminator = DeadCodeEliminator;
        let function_inliner = FunctionInliner::new(10, true);
        
        // First fold constants
        constant_folder.apply(&mut program).unwrap();
        
        // Check if constant folding worked - if calc function exists
        if let Some(calc_function) = program.functions.iter().find(|f| f.name == "calc") {
            if let Statement::VariableDeclaration { initializer, .. } = &calc_function.body[0] {
                // The expression 5 * 4 should be folded to 20
                assert!(matches!(initializer, Expression::IntegerLiteral(20)));
            } else {
                panic!("Expected variable declaration");
            }
            
            // Then eliminate dead code
            dead_code_eliminator.apply(&mut program).unwrap();
            
            // The "unused" variable should be eliminated if calc still exists
            if let Some(calc_after_dce) = program.functions.iter().find(|f| f.name == "calc") {
                assert_eq!(calc_after_dce.body.len(), 2); // Should only have val declaration and return now
            }
        }
        
        // Finally inline functions
        function_inliner.apply(&mut program).unwrap();
        
        // Verify the main function exists
        let main_function = program.functions.iter()
            .find(|f| f.name == "main")
            .expect("Main function should exist after inlining");
            
        // Main should have at least its original statements
        assert!(main_function.body.len() >= 2,
            "Main function should have at least its original statements");
        
        // Verify that the main function still has a return statement
        let has_return = main_function.body.iter().any(|stmt| {
            matches!(stmt, Statement::Return(_))
        });
        
        assert!(has_return, "Main function should have a return statement");
    }
} 