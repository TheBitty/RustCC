#[cfg(test)]
mod tests {
    use crate::parser::ast::{
        BinaryOp, Expression, Function, FunctionParameter, Program, Statement, Type,
    };
    use crate::transforms::optimization::{ConstantFolder, DeadCodeEliminator, FunctionInliner};
    use crate::transforms::Transform;

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
                    body: vec![Statement::Return(Expression::BinaryOperation {
                        left: Box::new(Expression::Variable("a".to_string())),
                        operator: BinaryOp::Add,
                        right: Box::new(Expression::Variable("b".to_string())),
                    })],
                    is_external: false,
                    is_variadic: false,
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
                            is_global: false,
                        },
                        Statement::ExpressionStatement(Expression::Assignment {
                            target: Box::new(Expression::Variable("result".to_string())),
                            value: Box::new(Expression::FunctionCall {
                                name: "add".to_string(),
                                arguments: vec![
                                    Expression::IntegerLiteral(2),
                                    Expression::IntegerLiteral(3),
                                ],
                            }),
                        }),
                        Statement::Return(Expression::Variable("result".to_string())),
                    ],
                    is_external: false,
                    is_variadic: false,
                },
            ],
            structs: vec![],
            includes: vec![],
            globals: vec![],
        }
    }

    #[test]
    fn test_constant_folder() {
        let mut program = create_basic_program();
        let constant_folder = ConstantFolder;
        constant_folder.apply(&mut program).unwrap();

        // Check that the function call was replaced with a constant
        if let Statement::ExpressionStatement(Expression::Assignment { value, .. }) =
            &program.functions[1].body[1]
        {
            if let Expression::FunctionCall { .. } = **value {
                panic!("Function call was not folded to a constant");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_dead_code_eliminator() {
        let mut program = Program {
            functions: vec![Function {
                name: "main".to_string(),
                return_type: Type::Int,
                parameters: vec![],
                body: vec![
                    Statement::VariableDeclaration {
                        name: "unused".to_string(),
                        data_type: Some(Type::Int),
                        initializer: Expression::IntegerLiteral(42),
                        is_global: false,
                    },
                    Statement::VariableDeclaration {
                        name: "used".to_string(),
                        data_type: Some(Type::Int),
                        initializer: Expression::IntegerLiteral(10),
                        is_global: false,
                    },
                    Statement::Return(Expression::Variable("used".to_string())),
                ],
                is_external: false,
                is_variadic: false,
            }],
            structs: vec![],
            includes: vec![],
            globals: vec![],
        };

        let dead_code_eliminator = DeadCodeEliminator;
        dead_code_eliminator.apply(&mut program).unwrap();

        // Check that the unused variable was removed
        assert_eq!(program.functions[0].body.len(), 2);
        if let Statement::VariableDeclaration { name, .. } = &program.functions[0].body[0] {
            assert_eq!(name, "used");
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_function_inliner() {
        // Create a program with a simple function to inline
        let mut program = Program {
            functions: vec![
                Function {
                    name: "square".to_string(),
                    return_type: Type::Int,
                    parameters: vec![FunctionParameter {
                        name: "x".to_string(),
                        data_type: Type::Int,
                    }],
                    body: vec![Statement::Return(Expression::BinaryOperation {
                        left: Box::new(Expression::Variable("x".to_string())),
                        operator: BinaryOp::Multiply,
                        right: Box::new(Expression::Variable("x".to_string())),
                    })],
                    is_external: false,
                    is_variadic: false,
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
                                name: "square".to_string(),
                                arguments: vec![Expression::IntegerLiteral(5)],
                            },
                            is_global: false,
                        },
                        Statement::Return(Expression::Variable("result".to_string())),
                    ],
                    is_external: false,
                    is_variadic: false,
                },
            ],
            structs: vec![],
            includes: vec![],
            globals: vec![],
        };

        let function_inliner = FunctionInliner::new(10, true);
        function_inliner.apply(&mut program).unwrap();

        // Check that the function call was inlined
        if let Statement::VariableDeclaration { initializer, .. } = &program.functions[1].body[0] {
            if let Expression::FunctionCall { .. } = initializer {
                panic!("Function call was not inlined");
            }
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_multiple_optimizations() {
        let mut program = Program {
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
                    body: vec![Statement::Return(Expression::BinaryOperation {
                        left: Box::new(Expression::Variable("a".to_string())),
                        operator: BinaryOp::Add,
                        right: Box::new(Expression::Variable("b".to_string())),
                    })],
                    is_external: false,
                    is_variadic: false,
                },
                Function {
                    name: "main".to_string(),
                    return_type: Type::Int,
                    parameters: vec![],
                    body: vec![
                        Statement::VariableDeclaration {
                            name: "unused".to_string(),
                            data_type: Some(Type::Int),
                            initializer: Expression::IntegerLiteral(100),
                            is_global: false,
                        },
                        Statement::VariableDeclaration {
                            name: "result".to_string(),
                            data_type: Some(Type::Int),
                            initializer: Expression::FunctionCall {
                                name: "add".to_string(),
                                arguments: vec![
                                    Expression::IntegerLiteral(2),
                                    Expression::IntegerLiteral(3),
                                ],
                            },
                            is_global: false,
                        },
                        Statement::Return(Expression::Variable("result".to_string())),
                    ],
                    is_external: false,
                    is_variadic: false,
                },
            ],
            structs: vec![],
            includes: vec![],
            globals: vec![],
        };

        // Apply multiple optimizations
        let function_inliner = FunctionInliner::new(10, true);
        function_inliner.apply(&mut program).unwrap();

        let constant_folder = ConstantFolder;
        constant_folder.apply(&mut program).unwrap();

        let dead_code_eliminator = DeadCodeEliminator;
        dead_code_eliminator.apply(&mut program).unwrap();

        // Check that the function call was inlined and constant-folded
        if let Statement::VariableDeclaration { initializer, .. } = &program.functions[1].body[0] {
            if let Expression::IntegerLiteral(value) = initializer {
                assert_eq!(*value, 5);
            } else {
                panic!("Expected integer literal after optimization");
            }
        } else {
            panic!("Expected variable declaration");
        }

        // Check that the unused variable was eliminated
        assert_eq!(program.functions[1].body.len(), 2);
    }
}
