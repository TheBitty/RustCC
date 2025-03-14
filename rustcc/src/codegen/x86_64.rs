use crate::parser::ast::{BinaryOp, Expression, Function, Program, Statement};
use std::collections::HashMap;

pub struct X86_64Generator {
    output: String,
    variables: HashMap<String, i32>, // Maps variable names to stack offsets
    current_stack_offset: i32,
}

impl X86_64Generator {
    pub fn new() -> Self {
        X86_64Generator {
            output: String::new(),
            variables: HashMap::new(),
            current_stack_offset: 0,
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.output.clear();
        self.variables.clear();
        self.current_stack_offset = 0;

        // Add necessary assembly directives and headers
        self.emit_line(".section __TEXT,__text,regular,pure_instructions");
        self.emit_line(".globl _main");
        self.emit_line(".p2align 4, 0x90");

        // Generate code for each function
        for function in &program.functions {
            self.generate_function(function);
        }

        self.output.clone()
    }

    fn generate_function(&mut self, function: &Function) {
        self.emit_line(&format!("_{}: ", function.name));

        // Function prologue
        self.emit_line("    push %rbp");
        self.emit_line("    mov %rsp, %rbp");

        // Reserve stack space for local variables
        let stack_size = (function.body.len() * 8) as i32; // 8 bytes per variable
        if stack_size > 0 {
            self.emit_line(&format!("    sub ${}, %rsp", stack_size));
        }

        // Generate code for function body
        for statement in &function.body {
            self.generate_statement(statement);
        }

        // Check if the function already has a return statement
        let has_return = function
            .body
            .iter()
            .any(|stmt| matches!(stmt, Statement::Return(_)));

        // Function epilogue (if not already returned)
        if !has_return {
            self.emit_line("    mov %rbp, %rsp");
            self.emit_line("    pop %rbp");
            self.emit_line("    ret");
        }
    }

    fn generate_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Return(expr) => {
                // Evaluate expression and put result in %rax
                self.generate_expression(expr);
                self.emit_line("    mov %rbp, %rsp");
                self.emit_line("    pop %rbp");
                self.emit_line("    ret");
            }
            Statement::VariableDeclaration {
                name,
                initializer,
                data_type: _,
                is_global: _,
            } => {
                // Evaluate initializer
                self.generate_expression(initializer);

                // Store result in stack
                self.current_stack_offset -= 8;
                self.variables
                    .insert(name.clone(), self.current_stack_offset);
                self.emit_line(&format!(
                    "    mov %rax, {}(%rbp)",
                    self.current_stack_offset
                ));
            }
            _ => {
                // Other statement types not yet implemented
            }
        }
    }

    fn generate_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::IntegerLiteral(value) => {
                self.emit_line(&format!("    mov ${}, %rax", value));
            }
            Expression::BinaryOperation {
                left,
                operator,
                right,
            } => {
                // Generate code for right operand first
                self.generate_expression(right);
                // Save right operand
                self.emit_line("    push %rax");

                // Generate code for left operand
                self.generate_expression(left);

                // Restore right operand into %rcx
                self.emit_line("    pop %rcx");

                // Perform operation
                match operator {
                    BinaryOp::Add => self.emit_line("    add %rcx, %rax"),
                    BinaryOp::Subtract => self.emit_line("    sub %rcx, %rax"),
                    BinaryOp::Multiply => self.emit_line("    imul %rcx, %rax"),
                    BinaryOp::Divide => {
                        self.emit_line("    cqo"); // Sign extend %rax into %rdx
                        self.emit_line("    idiv %rcx");
                    }
                    _ => {
                        // Other operators not yet implemented
                    }
                }
            }
            Expression::Variable(name) => {
                if let Some(offset) = self.variables.get(name) {
                    self.emit_line(&format!("    mov {}(%rbp), %rax", offset));
                }
            }
            _ => {
                // Other expression types not yet implemented
            }
        }
    }

    fn emit_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }
}

impl Default for X86_64Generator {
    fn default() -> Self {
        Self::new()
    }
}
