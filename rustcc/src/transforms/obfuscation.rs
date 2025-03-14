use crate::parser::ast::{BinaryOp, Expression, Program, Statement, Type, UnaryOp};
use crate::transforms::Transform;
use rand::{distributions::Alphanumeric, seq::SliceRandom, thread_rng, Rng};
use std::collections::HashMap;

/// Variable Name Obfuscation
/// Replaces all variable names with random strings
pub struct VariableObfuscator;

impl Transform for VariableObfuscator {
    fn apply(&self, program: &mut Program) -> std::result::Result<(), String> {
        let mut rng = thread_rng();

        for function in &mut program.functions {
            let mut var_map: HashMap<String, String> = HashMap::new();

            // Gather all variable names from the function body
            for statement in &function.body {
                if let Statement::VariableDeclaration { name, .. } = statement {
                    if !var_map.contains_key(name) {
                        // Generate a random name with mixed case for increased confusion
                        let new_name: String = std::iter::repeat(())
                            .map(|()| {
                                let c = rng.sample(Alphanumeric) as char;
                                if rng.gen_bool(0.5) {
                                    c.to_uppercase().next().unwrap_or(c)
                                } else {
                                    c
                                }
                            })
                            .take(12) // Longer names increase confusion
                            .collect();

                        // Add prefixes that resemble internal compiler symbols
                        var_map.insert(
                            name.clone(),
                            format!("_${}_{}", rng.gen::<u32>() % 1000, new_name),
                        );
                    }
                }
            }

            // Replace variable names in statements
            for statement in &mut function.body {
                self.obfuscate_statement(statement, &var_map);
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Variable Obfuscator"
    }
}

impl VariableObfuscator {
    fn obfuscate_statement(&self, statement: &mut Statement, var_map: &HashMap<String, String>) {
        match statement {
            Statement::VariableDeclaration {
                name,
                initializer,
                data_type: _,
            } => {
                if let Some(new_name) = var_map.get(name) {
                    *name = new_name.clone();
                }
                self.obfuscate_expression(initializer, var_map);
            }
            Statement::Return(expr) => {
                self.obfuscate_expression(expr, var_map);
            }
            Statement::ExpressionStatement(expr) => {
                self.obfuscate_expression(expr, var_map);
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.obfuscate_statement(stmt, var_map);
                }
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.obfuscate_expression(condition, var_map);
                self.obfuscate_statement(then_block, var_map);
                if let Some(else_stmt) = else_block {
                    self.obfuscate_statement(else_stmt, var_map);
                }
            }
            Statement::While { condition, body } => {
                self.obfuscate_expression(condition, var_map);
                self.obfuscate_statement(body, var_map);
            }
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(init) = initializer {
                    self.obfuscate_statement(init, var_map);
                }
                if let Some(cond) = condition {
                    self.obfuscate_expression(cond, var_map);
                }
                if let Some(inc) = increment {
                    self.obfuscate_expression(inc, var_map);
                }
                self.obfuscate_statement(body, var_map);
            }
            Statement::DoWhile { body, condition } => {
                self.obfuscate_statement(body, var_map);
                self.obfuscate_expression(condition, var_map);
            }
            Statement::Break | Statement::Continue => {}
            Statement::Switch { expression, cases } => {
                self.obfuscate_expression(expression, var_map);
                for case in cases {
                    if let Some(value) = &mut case.value {
                        self.obfuscate_expression(value, var_map);
                    }
                    for stmt in &mut case.statements {
                        self.obfuscate_statement(stmt, var_map);
                    }
                }
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn obfuscate_expression(&self, expr: &mut Expression, var_map: &HashMap<String, String>) {
        match expr {
            Expression::Variable(name) => {
                if let Some(new_name) = var_map.get(name) {
                    *name = new_name.clone();
                }
            }
            Expression::BinaryOperation { left, right, .. } => {
                self.obfuscate_expression(left, var_map);
                self.obfuscate_expression(right, var_map);
            }
            Expression::IntegerLiteral(_) => {}
            Expression::StringLiteral(_s) => {
                // We could encrypt strings here too, but we'll leave it for the StringEncryptor
                // which is specifically designed for that purpose
            }
            Expression::CharLiteral(_) => {}
            Expression::UnaryOperation { operand, .. } => {
                self.obfuscate_expression(operand, var_map);
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.obfuscate_expression(arg, var_map);
                }
            }
            Expression::Assignment { target, value } => {
                self.obfuscate_expression(target, var_map);
                self.obfuscate_expression(value, var_map);
            }
            Expression::TernaryIf {
                condition,
                then_expr,
                else_expr,
            } => {
                self.obfuscate_expression(condition, var_map);
                self.obfuscate_expression(then_expr, var_map);
                self.obfuscate_expression(else_expr, var_map);
            }
            Expression::Cast { expr, .. } => {
                self.obfuscate_expression(expr, var_map);
            }
            Expression::SizeOf { .. } => {}
            Expression::ArrayAccess { array, index } => {
                self.obfuscate_expression(array, var_map);
                self.obfuscate_expression(index, var_map);
            }
            Expression::StructFieldAccess { object, .. } => {
                self.obfuscate_expression(object, var_map);
            }
            Expression::PointerFieldAccess { pointer, .. } => {
                self.obfuscate_expression(pointer, var_map);
            }
        }
    }
}

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
                                left: Box::new(Expression::IntegerLiteral(value)),
                                operator: BinaryOp::Add,
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
                                left: Box::new(Expression::IntegerLiteral(value)),
                                operator: BinaryOp::Multiply,
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
                                left: Box::new(Expression::IntegerLiteral(value)),
                                operator: BinaryOp::BitwiseOr,
                                right: Box::new(Expression::IntegerLiteral(0)),
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
            };

            // Create junk code that will never execute
            let junk_var_name = format!("_junk_{}", rng.gen::<u32>());
            let junk_declaration = Statement::VariableDeclaration {
                name: junk_var_name.clone(),
                data_type: Some(Type::Int),
                initializer: Expression::IntegerLiteral(rng.gen_range(1..1000)),
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
                    operator: UnaryOp::Negate,
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
                };

                statements.push(stmt);
            }
        }
    }
}

/// String Encryption Obfuscation
/// Encrypts string literals to make them harder to identify
pub struct StringEncryptor;

impl Transform for StringEncryptor {
    fn apply(&self, program: &mut Program) -> std::result::Result<(), String> {
        let mut rng = thread_rng();

        for function in &mut program.functions {
            self.encrypt_strings_in_statements(&mut function.body, &mut rng);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "String Encryptor"
    }
}

impl StringEncryptor {
    // Recursively encrypt all string literals in statements
    fn encrypt_strings_in_statements(&self, statements: &mut Vec<Statement>, rng: &mut impl Rng) {
        for statement in statements {
            match statement {
                Statement::Return(expr) => {
                    *expr = self.encrypt_strings_in_expression(expr.clone(), rng);
                }
                Statement::VariableDeclaration { initializer, .. } => {
                    *initializer = self.encrypt_strings_in_expression(initializer.clone(), rng);
                }
                Statement::ExpressionStatement(expr) => {
                    *expr = self.encrypt_strings_in_expression(expr.clone(), rng);
                }
                Statement::Block(stmts) => {
                    self.encrypt_strings_in_statements(stmts, rng);
                }
                Statement::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    *condition = self.encrypt_strings_in_expression(condition.clone(), rng);

                    if let Statement::Block(stmts) = then_block.as_mut() {
                        self.encrypt_strings_in_statements(stmts, rng);
                    } else {
                        // Handle non-block then statements
                        let new_then =
                            self.encrypt_strings_in_statement(then_block.as_ref().clone(), rng);
                        *then_block = Box::new(new_then);
                    }

                    if let Some(else_stmt) = else_block {
                        if let Statement::Block(stmts) = else_stmt.as_mut() {
                            self.encrypt_strings_in_statements(stmts, rng);
                        } else {
                            // Handle non-block else statements
                            let new_else =
                                self.encrypt_strings_in_statement(else_stmt.as_ref().clone(), rng);
                            *else_block = Some(Box::new(new_else));
                        }
                    }
                }
                Statement::While { condition, body } => {
                    *condition = self.encrypt_strings_in_expression(condition.clone(), rng);

                    let new_body = self.encrypt_strings_in_statement(body.as_ref().clone(), rng);
                    *body = Box::new(new_body);
                }
                Statement::For {
                    condition,
                    increment,
                    body,
                    ..
                } => {
                    if let Some(cond) = condition {
                        *cond = self.encrypt_strings_in_expression(cond.clone(), rng);
                    }

                    if let Some(inc) = increment {
                        *inc = self.encrypt_strings_in_expression(inc.clone(), rng);
                    }

                    let new_body = self.encrypt_strings_in_statement(body.as_ref().clone(), rng);
                    *body = Box::new(new_body);
                }
                Statement::DoWhile { body, condition } => {
                    *condition = self.encrypt_strings_in_expression(condition.clone(), rng);

                    let new_body = self.encrypt_strings_in_statement(body.as_ref().clone(), rng);
                    *body = Box::new(new_body);
                }
                Statement::Switch { expression, cases } => {
                    *expression = self.encrypt_strings_in_expression(expression.clone(), rng);

                    for case in cases {
                        if let Some(value) = &mut case.value {
                            *value = self.encrypt_strings_in_expression(value.clone(), rng);
                        }

                        self.encrypt_strings_in_statements(&mut case.statements, rng);
                    }
                }
                _ => {} // Other statement types may not contain expressions
            }
        }
    }

    // Process a single statement
    fn encrypt_strings_in_statement(&self, statement: Statement, rng: &mut impl Rng) -> Statement {
        match statement {
            Statement::Block(mut stmts) => {
                self.encrypt_strings_in_statements(&mut stmts, rng);
                Statement::Block(stmts)
            }
            Statement::Return(expr) => {
                Statement::Return(self.encrypt_strings_in_expression(expr, rng))
            }
            Statement::VariableDeclaration {
                name,
                data_type,
                initializer,
            } => Statement::VariableDeclaration {
                name,
                data_type,
                initializer: self.encrypt_strings_in_expression(initializer, rng),
            },
            Statement::ExpressionStatement(expr) => {
                Statement::ExpressionStatement(self.encrypt_strings_in_expression(expr, rng))
            }
            // Other statement types would need similar handling
            _ => statement,
        }
    }

    // Find and encrypt string literals in expressions
    #[allow(clippy::only_used_in_recursion)]
    fn encrypt_strings_in_expression(&self, expr: Expression, rng: &mut impl Rng) -> Expression {
        match expr {
            Expression::StringLiteral(s) => {
                // Apply XOR encryption on the string
                let key = rng.gen::<u8>() as char;
                let encrypted: String = s.chars().map(|c| (c as u8 ^ key as u8) as char).collect();

                // We'd need to modify the compiler to handle this properly at runtime
                // For now we're just replacing the string with a placeholder
                // In a real implementation, we'd insert decryption code
                Expression::StringLiteral(format!("ENCRYPTED:{}:{}", key as u8, encrypted))
            }
            Expression::BinaryOperation {
                left,
                operator,
                right,
            } => Expression::BinaryOperation {
                left: Box::new(self.encrypt_strings_in_expression(*left, rng)),
                operator,
                right: Box::new(self.encrypt_strings_in_expression(*right, rng)),
            },
            Expression::UnaryOperation { operator, operand } => Expression::UnaryOperation {
                operator,
                operand: Box::new(self.encrypt_strings_in_expression(*operand, rng)),
            },
            Expression::FunctionCall {
                name,
                mut arguments,
            } => {
                for arg in &mut arguments {
                    *arg = self.encrypt_strings_in_expression(arg.clone(), rng);
                }

                Expression::FunctionCall { name, arguments }
            }
            Expression::Assignment { target, value } => Expression::Assignment {
                target: Box::new(self.encrypt_strings_in_expression(*target, rng)),
                value: Box::new(self.encrypt_strings_in_expression(*value, rng)),
            },
            Expression::TernaryIf {
                condition,
                then_expr,
                else_expr,
            } => Expression::TernaryIf {
                condition: Box::new(self.encrypt_strings_in_expression(*condition, rng)),
                then_expr: Box::new(self.encrypt_strings_in_expression(*then_expr, rng)),
                else_expr: Box::new(self.encrypt_strings_in_expression(*else_expr, rng)),
            },
            Expression::Cast { target_type, expr } => Expression::Cast {
                target_type,
                expr: Box::new(self.encrypt_strings_in_expression(*expr, rng)),
            },
            Expression::ArrayAccess { array, index } => Expression::ArrayAccess {
                array: Box::new(self.encrypt_strings_in_expression(*array, rng)),
                index: Box::new(self.encrypt_strings_in_expression(*index, rng)),
            },
            Expression::StructFieldAccess { object, field } => Expression::StructFieldAccess {
                object: Box::new(self.encrypt_strings_in_expression(*object, rng)),
                field,
            },
            Expression::PointerFieldAccess { pointer, field } => Expression::PointerFieldAccess {
                pointer: Box::new(self.encrypt_strings_in_expression(*pointer, rng)),
                field,
            },
            // Other expression types may not contain nested expressions
            _ => expr,
        }
    }
}
