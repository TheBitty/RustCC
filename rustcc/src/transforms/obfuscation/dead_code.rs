use crate::parser::ast::{BinaryOp, Expression, Program, Statement, Type, UnaryOp};
use crate::transforms::Transform;
use rand::{thread_rng, Rng};

/// Advanced Dead Code Insertion
/// Adds various types of meaningless but complex code to confuse reverse engineers
pub struct DeadCodeInserter;

impl Transform for DeadCodeInserter {
    fn apply(&self, program: &mut Program) -> std::result::Result<(), String> {
        let mut rng = thread_rng();

        for function in &mut program.functions {
            // Generate a list of dummy variable names with convincing patterns
            let dummy_var_count = rng.gen_range(3..8); // Increase the number of dummy variables
            let mut dummy_vars = Vec::new();

            // Generate variables that look like they have meaningful purposes
            let prefixes = [
                "counter", "index", "temp", "buffer", "size", "len", "offset", "ptr", "flag",
            ];

            for _ in 0..dummy_var_count {
                let prefix = prefixes[rng.gen_range(0..prefixes.len())];
                let suffix = rng.gen_range(1..100);
                dummy_vars.push(format!("{}_{}", prefix, suffix));
            }

            // Insert dummy declarations and more complex dead code
            let original_len = function.body.len();
            let mut new_statements = Vec::new();

            // Add complex initialization at the start
            self.add_complex_initialization(&mut new_statements, &dummy_vars, &mut rng);

            for (i, stmt) in function.body.drain(..).enumerate() {
                // Decide whether to insert dead code before this statement
                // Higher probability for more code insertion
                if rng.gen_bool(0.4) && i < original_len - 1 {
                    // Insert more complex dead code
                    self.insert_complex_dead_code(&mut new_statements, &dummy_vars, &mut rng);
                }

                // Add the original statement
                new_statements.push(stmt);
            }

            function.body = new_statements;
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Dead Code Inserter"
    }
}

impl DeadCodeInserter {
    // Helper method to add complex initialization code
    fn add_complex_initialization(
        &self,
        statements: &mut Vec<Statement>,
        dummy_vars: &[String],
        rng: &mut impl Rng,
    ) {
        // Add a few variable declarations with complex initializers
        for var_name in dummy_vars.iter().take(3) {
            // Create a complex initialization expression
            let initializer = self.create_complex_expression(rng);

            // Create the variable declaration
            let decl = Statement::VariableDeclaration {
                name: var_name.clone(),
                data_type: Some(Type::Int),
                initializer,
                is_global: false,
            };

            statements.push(decl);
        }
    }

    // Helper method to create complex but meaningless expressions
    fn create_complex_expression(&self, rng: &mut impl Rng) -> Expression {
        // Choose a random pattern for the expression
        match rng.gen_range(0..5) {
            0 => {
                // Nested binary operations: (a + b) * (c - d)
                let a = rng.gen_range(1..100);
                let b = rng.gen_range(1..100);
                let c = rng.gen_range(1..100);
                let d = rng.gen_range(1..c); // Ensure c > d to avoid negative results

                Expression::BinaryOperation {
                    left: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::IntegerLiteral(a)),
                        operator: BinaryOp::Add,
                        right: Box::new(Expression::IntegerLiteral(b)),
                    }),
                    operator: BinaryOp::Multiply,
                    right: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::IntegerLiteral(c)),
                        operator: BinaryOp::Subtract,
                        right: Box::new(Expression::IntegerLiteral(d)),
                    }),
                }
            }
            1 => {
                // Bitwise operations: a & (b | c)
                let a = rng.gen_range(1..100);
                let b = rng.gen_range(1..100);
                let c = rng.gen_range(1..100);

                Expression::BinaryOperation {
                    left: Box::new(Expression::IntegerLiteral(a)),
                    operator: BinaryOp::BitwiseAnd,
                    right: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::IntegerLiteral(b)),
                        operator: BinaryOp::BitwiseOr,
                        right: Box::new(Expression::IntegerLiteral(c)),
                    }),
                }
            }
            2 => {
                // Ternary operation: a > b ? c : d
                let a = rng.gen_range(1..100);
                let b = rng.gen_range(1..100);
                let c = rng.gen_range(1..100);
                let d = rng.gen_range(1..100);

                Expression::TernaryIf {
                    condition: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::IntegerLiteral(a)),
                        operator: BinaryOp::GreaterThan,
                        right: Box::new(Expression::IntegerLiteral(b)),
                    }),
                    then_expr: Box::new(Expression::IntegerLiteral(c)),
                    else_expr: Box::new(Expression::IntegerLiteral(d)),
                }
            }
            3 => {
                // Unary operations: -(a * b)
                let a = rng.gen_range(1..100);
                let b = rng.gen_range(1..100);

                Expression::UnaryOperation {
                    operator: crate::parser::ast::OperatorType::Unary(UnaryOp::Negate),
                    operand: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::IntegerLiteral(a)),
                        operator: BinaryOp::Multiply,
                        right: Box::new(Expression::IntegerLiteral(b)),
                    }),
                }
            }
            _ => {
                // Modulo operation: (a * b) % c
                let a = rng.gen_range(1..20);
                let b = rng.gen_range(1..20);
                let c = rng.gen_range(1..100);

                Expression::BinaryOperation {
                    left: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::IntegerLiteral(a)),
                        operator: BinaryOp::Multiply,
                        right: Box::new(Expression::IntegerLiteral(b)),
                    }),
                    operator: BinaryOp::Modulo,
                    right: Box::new(Expression::IntegerLiteral(c)),
                }
            }
        }
    }

    // Insert more complex dead code
    fn insert_complex_dead_code(
        &self,
        statements: &mut Vec<Statement>,
        dummy_vars: &[String],
        rng: &mut impl Rng,
    ) {
        // Choose a random pattern of dead code to insert
        match rng.gen_range(0..5) {
            0 => {
                // Simple variable assignment with complex expression
                if !dummy_vars.is_empty() {
                    let var_idx = rng.gen_range(0..dummy_vars.len());
                    let var_name = dummy_vars[var_idx].clone();

                    let expr = self.create_complex_expression(rng);
                    let stmt = Statement::ExpressionStatement(Expression::Assignment {
                        target: Box::new(Expression::Variable(var_name)),
                        value: Box::new(expr),
                    });

                    statements.push(stmt);
                }
            }
            1 => {
                // Conditional block that does nothing meaningful
                if dummy_vars.len() >= 2 {
                    let var1_idx = rng.gen_range(0..dummy_vars.len());
                    let var2_idx = (var1_idx + 1) % dummy_vars.len(); // Ensure different from var1

                    let var1 = dummy_vars[var1_idx].clone();
                    let var2 = dummy_vars[var2_idx].clone();

                    // Create condition like: var1 > var2 || var1 < var2 (always true)
                    let condition = Expression::BinaryOperation {
                        left: Box::new(Expression::BinaryOperation {
                            left: Box::new(Expression::Variable(var1.clone())),
                            operator: BinaryOp::GreaterThan,
                            right: Box::new(Expression::Variable(var2.clone())),
                        }),
                        operator: BinaryOp::LogicalOr,
                        right: Box::new(Expression::BinaryOperation {
                            left: Box::new(Expression::Variable(var1)),
                            operator: BinaryOp::LessThan,
                            right: Box::new(Expression::Variable(var2)),
                        }),
                    };

                    // Create meaningless block
                    let block = Statement::Block(vec![Statement::ExpressionStatement(
                        Expression::IntegerLiteral(rng.gen_range(1..1000)),
                    )]);

                    // Create the if statement
                    let if_stmt = Statement::If {
                        condition,
                        then_block: Box::new(block),
                        else_block: None,
                    };

                    statements.push(if_stmt);
                }
            }
            2 => {
                // Loop with a fixed number of iterations
                if !dummy_vars.is_empty() {
                    let var_idx = rng.gen_range(0..dummy_vars.len());
                    let var_name = dummy_vars[var_idx].clone();

                    // Initialize counter
                    let init_stmt = Statement::VariableDeclaration {
                        name: format!("_loop_counter_{}", rng.gen::<u32>()),
                        data_type: Some(Type::Int),
                        initializer: Expression::IntegerLiteral(0),
                        is_global: false,
                    };

                    statements.push(init_stmt);

                    // Create a loop with a small number of iterations
                    let iterations = rng.gen_range(1..5);
                    let loop_var = format!("_loop_counter_{}", rng.gen::<u32>());

                    // Loop initialization
                    let init = Statement::VariableDeclaration {
                        name: loop_var.clone(),
                        data_type: Some(Type::Int),
                        initializer: Expression::IntegerLiteral(0),
                        is_global: false,
                    };

                    // Loop condition
                    let condition = Expression::BinaryOperation {
                        left: Box::new(Expression::Variable(loop_var.clone())),
                        operator: BinaryOp::LessThan,
                        right: Box::new(Expression::IntegerLiteral(iterations)),
                    };

                    // Loop increment
                    let increment = Expression::Assignment {
                        target: Box::new(Expression::Variable(loop_var.clone())),
                        value: Box::new(Expression::BinaryOperation {
                            left: Box::new(Expression::Variable(loop_var)),
                            operator: BinaryOp::Add,
                            right: Box::new(Expression::IntegerLiteral(1)),
                        }),
                    };

                    // Loop body
                    let body = Statement::Block(vec![Statement::ExpressionStatement(
                        Expression::Assignment {
                            target: Box::new(Expression::Variable(var_name)),
                            value: Box::new(self.create_complex_expression(rng)),
                        },
                    )]);

                    // Create the for loop
                    let for_stmt = Statement::For {
                        initializer: Some(Box::new(init)),
                        condition: Some(condition),
                        increment: Some(increment),
                        body: Box::new(body),
                    };

                    statements.push(for_stmt);
                }
            }
            3 => {
                // Nested expression statement
                statements.push(Statement::ExpressionStatement(
                    self.create_complex_expression(rng),
                ));
            }
            _ => {
                // Create a new dummy variable with complex initialization
                let var_name = format!("_complex_var_{}", rng.gen::<u32>());
                let expr = self.create_complex_expression(rng);

                let stmt = Statement::VariableDeclaration {
                    name: var_name,
                    data_type: Some(Type::Int),
                    initializer: expr,
                    is_global: false,
                };

                statements.push(stmt);
            }
        }
    }
} 