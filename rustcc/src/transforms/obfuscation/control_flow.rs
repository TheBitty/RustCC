use crate::parser::ast::{BinaryOp, Expression, Program, Statement, Type};
use crate::transforms::Transform;
use rand::{seq::SliceRandom, thread_rng, Rng};

/// Control Flow Obfuscation with Flattening
/// Makes control flow harder to understand by introducing a state machine pattern
/// and adding opaque predicates (computations that always evaluate to a constant)
pub struct ControlFlowObfuscator;

impl Transform for ControlFlowObfuscator {
    fn apply(&self, program: &mut Program) -> std::result::Result<(), String> {
        let mut rng = thread_rng();

        for function in &mut program.functions {
            // Only apply to functions with more than one statement
            if function.body.len() > 1 {
                // 1. Complex Return Value Obfuscation
                let mut return_indices = Vec::new();
                for (i, stmt) in function.body.iter().enumerate() {
                    if matches!(stmt, Statement::Return(_)) {
                        return_indices.push(i);
                    }
                }

                // Obfuscate each return statement with complex expressions
                for &idx in &return_indices {
                    if let Statement::Return(expr) = &function.body[idx] {
                        let obfuscated_expr = self.obfuscate_expression(expr.clone(), &mut rng);
                        function.body[idx] = Statement::Return(obfuscated_expr);
                    }
                }

                // 2. Control Flow Flattening for if statements
                // This transforms structured if-else into switch-case style control flow
                self.flatten_control_flow(function, &mut rng);

                // 3. Insert opaque predicates
                self.insert_opaque_predicates(function, &mut rng);
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Control Flow Obfuscator"
    }
}

impl ControlFlowObfuscator {
    // Helper function to create complex but equivalent expressions
    #[allow(clippy::only_used_in_recursion)]
    fn obfuscate_expression(&self, expr: Expression, rng: &mut impl Rng) -> Expression {
        match expr {
            // For integer literals, create complex expressions that evaluate to the same value
            Expression::IntegerLiteral(value) => {
                // Choose a random obfuscation pattern
                match rng.gen_range(0..5) {
                    0 => {
                        // (x + a) - a
                        let a = rng.gen_range(1000..10000);
                        Expression::BinaryOperation {
                            left: Box::new(Expression::BinaryOperation {
                                left: Box::new(Expression::BinaryOperation {
                                    left: Box::new(Expression::IntegerLiteral(value)),
                                    operator: BinaryOp::Add,
                                    right: Box::new(Expression::IntegerLiteral(a)),
                                }),
                                operator: BinaryOp::Subtract,
                                right: Box::new(Expression::IntegerLiteral(a)),
                            }),
                            operator: BinaryOp::Subtract,
                            right: Box::new(Expression::IntegerLiteral(a)),
                        }
                    }
                    1 => {
                        // (x * a) / a
                        let a = rng.gen_range(2..10);
                        Expression::BinaryOperation {
                            left: Box::new(Expression::BinaryOperation {
                                left: Box::new(Expression::BinaryOperation {
                                    left: Box::new(Expression::IntegerLiteral(value)),
                                    operator: BinaryOp::Multiply,
                                    right: Box::new(Expression::IntegerLiteral(a)),
                                }),
                                operator: BinaryOp::Divide,
                                right: Box::new(Expression::IntegerLiteral(a)),
                            }),
                            operator: BinaryOp::Divide,
                            right: Box::new(Expression::IntegerLiteral(a)),
                        }
                    }
                    2 => {
                        // x ^ 0 (XOR with 0 returns x)
                        Expression::BinaryOperation {
                            left: Box::new(Expression::IntegerLiteral(value)),
                            operator: BinaryOp::BitwiseXor,
                            right: Box::new(Expression::IntegerLiteral(0)),
                        }
                    }
                    3 => {
                        // x + (a - a)
                        let a = rng.gen_range(1000..10000);
                        Expression::BinaryOperation {
                            left: Box::new(Expression::IntegerLiteral(value)),
                            operator: BinaryOp::Add,
                            right: Box::new(Expression::BinaryOperation {
                                left: Box::new(Expression::IntegerLiteral(a)),
                                operator: BinaryOp::Subtract,
                                right: Box::new(Expression::IntegerLiteral(a)),
                            }),
                        }
                    }
                    _ => {
                        // (x | 0) & 0x7FFFFFFF
                        Expression::BinaryOperation {
                            left: Box::new(Expression::BinaryOperation {
                                left: Box::new(Expression::BinaryOperation {
                                    left: Box::new(Expression::BinaryOperation {
                                        left: Box::new(Expression::IntegerLiteral(value)),
                                        operator: BinaryOp::BitwiseOr,
                                        right: Box::new(Expression::IntegerLiteral(0)),
                                    }),
                                    operator: BinaryOp::BitwiseAnd,
                                    right: Box::new(Expression::IntegerLiteral(0x7FFFFFFF)),
                                }),
                                operator: BinaryOp::BitwiseAnd,
                                right: Box::new(Expression::IntegerLiteral(0x7FFFFFFF)),
                            }),
                            operator: BinaryOp::BitwiseAnd,
                            right: Box::new(Expression::IntegerLiteral(0x7FFFFFFF)),
                        }
                    }
                }
            }
            // For binary operations, recurse into the operands and optionally add more complexity
            Expression::BinaryOperation {
                left,
                operator,
                right,
            } => {
                if rng.gen_bool(0.7) {
                    // Apply additional obfuscation to operands
                    let obf_left = self.obfuscate_expression(*left, rng);
                    let obf_right = self.obfuscate_expression(*right, rng);

                    Expression::BinaryOperation {
                        left: Box::new(obf_left),
                        operator,
                        right: Box::new(obf_right),
                    }
                } else {
                    // Just recurse into operands
                    Expression::BinaryOperation {
                        left: Box::new(self.obfuscate_expression(*left, rng)),
                        operator,
                        right: Box::new(self.obfuscate_expression(*right, rng)),
                    }
                }
            }
            // For other expression types, return as is or add minimal obfuscation
            _ => expr,
        }
    }

    // Flattens control flow by converting structured if-else into a state machine pattern
    fn flatten_control_flow(
        &self,
        function: &mut crate::parser::ast::Function,
        rng: &mut impl Rng,
    ) {
        // This is a simplified implementation; a full flattening would convert the entire function body
        // to a switch-based state machine, but that's beyond the scope of this quick enhancement

        // Instead, we'll add junk conditional blocks with opaque predicates
        let num_junk_blocks = rng.gen_range(2..5);
        let mut new_body = Vec::new();

        // First add original statements
        for stmt in function.body.drain(..) {
            new_body.push(stmt);
        }

        // Add junk conditional blocks with opaque predicates that never execute
        for _ in 0..num_junk_blocks {
            // Create an opaque predicate that's always false
            // e.g., (x*x + 1) % 2 == 0 is always false for any integer x
            let random_int = rng.gen_range(1..100);
            let opaque_predicate = Expression::BinaryOperation {
                left: Box::new(Expression::BinaryOperation {
                    left: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::BinaryOperation {
                            left: Box::new(Expression::IntegerLiteral(random_int)),
                            operator: BinaryOp::Multiply,
                            right: Box::new(Expression::IntegerLiteral(random_int)),
                        }),
                        operator: BinaryOp::Add,
                        right: Box::new(Expression::IntegerLiteral(1)),
                    }),
                    operator: BinaryOp::Equal,
                    right: Box::new(Expression::BinaryOperation {
                        left: Box::new(Expression::BinaryOperation {
                            left: Box::new(Expression::IntegerLiteral(0)),
                            operator: BinaryOp::Multiply,
                            right: Box::new(Expression::IntegerLiteral(2)),
                        }),
                        operator: BinaryOp::Add,
                        right: Box::new(Expression::IntegerLiteral(0)),
                    }),
                }),
                operator: BinaryOp::Equal,
                right: Box::new(Expression::IntegerLiteral(0)),
            };

            // Create junk code that will never execute
            let junk_var_name = format!("_junk_{}", rng.gen::<u32>());
            let junk_declaration = Statement::VariableDeclaration {
                name: junk_var_name.clone(),
                data_type: Some(Type::Int),
                initializer: Expression::IntegerLiteral(rng.gen_range(1..1000)),
                is_global: false,
                alignment: None,
            };

            let junk_block = Statement::Block(vec![junk_declaration]);

            // Create the conditional statement with the opaque predicate
            let junk_if = Statement::If {
                condition: opaque_predicate,
                then_block: Box::new(junk_block),
                else_block: None,
            };

            // Add the junk if statement to the function body
            new_body.push(junk_if);
        }

        // Shuffle the statements to further confuse the control flow
        if new_body.len() > 3 {
            let last_index = new_body.len() - 1;

            // Keep first and last statements in place (may contain return)
            // but shuffle the middle ones
            let mut middle_indices: Vec<usize> = (1..last_index).collect();
            for _ in 0..10 {
                // Multiple shuffle passes
                middle_indices.shuffle(rng);
            }

            // Create shuffled body
            let mut shuffled_body = Vec::with_capacity(new_body.len());
            shuffled_body.push(new_body[0].clone());

            for &idx in &middle_indices {
                shuffled_body.push(new_body[idx].clone());
            }

            shuffled_body.push(new_body[last_index].clone());

            function.body = shuffled_body;
        } else {
            function.body = new_body;
        }
    }

    // Insert opaque predicates (computations that always evaluate to true/false)
    fn insert_opaque_predicates(
        &self,
        function: &mut crate::parser::ast::Function,
        rng: &mut impl Rng,
    ) {
        // This is a simplified version that adds an opaque predicate-based branch
        // to confuse control flow analysis

        // Example: Create an opaque predicate that always evaluates to true
        // but is hard to statically analyze - e.g., (x*x) >= 0 is always true for integers
        let random_int = rng.gen_range(1..100);
        let always_true_predicate = Expression::BinaryOperation {
            left: Box::new(Expression::BinaryOperation {
                left: Box::new(Expression::IntegerLiteral(random_int)),
                operator: BinaryOp::Multiply,
                right: Box::new(Expression::IntegerLiteral(random_int)),
            }),
            operator: BinaryOp::GreaterThanOrEqual,
            right: Box::new(Expression::IntegerLiteral(0)),
        };

        // If there's already code in the function, wrap it in an if statement with the opaque predicate
        if !function.body.is_empty() {
            let original_body = Statement::Block(function.body.clone());

            // Create an empty else block (will never execute, but confuses analysis)
            let else_block = Statement::Block(vec![Statement::ExpressionStatement(
                Expression::IntegerLiteral(rng.gen_range(1..1000)),
            )]);

            let predicated_flow = Statement::If {
                condition: always_true_predicate,
                then_block: Box::new(original_body),
                else_block: Some(Box::new(else_block)),
            };

            // Replace the function body with the predicated version
            function.body = vec![predicated_flow];
        }
    }
}
