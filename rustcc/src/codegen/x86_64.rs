use crate::parser::ast::{BinaryOp, Expression, Function, Program, Statement, Type, OperatorType, UnaryOp, Struct};
use std::collections::HashMap;

pub struct X86_64Generator {
    output: String,
    variables: HashMap<String, i32>, // Maps variable names to stack offsets
    current_stack_offset: i32,
    strings: Vec<String>,
    label_counter: usize,
    current_loop_end_label: Option<String>,
    current_loop_start_label: Option<String>,
    structs: HashMap<String, Vec<(String, Type, usize)>>, // struct name -> [(field name, type, offset)]
}

impl X86_64Generator {
    pub fn new() -> Self {
        X86_64Generator {
            output: String::new(),
            variables: HashMap::new(),
            current_stack_offset: 0,
            strings: Vec::new(),
            label_counter: 0,
            current_loop_end_label: None,
            current_loop_start_label: None,
            structs: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.output.clear();
        self.variables.clear();
        self.current_stack_offset = 0;
        self.strings.clear();
        self.label_counter = 0;

        // Add necessary assembly directives and headers
        self.emit_line(".section __TEXT,__text,regular,pure_instructions");
        
        // Register struct types
        for struct_def in &program.structs {
            self.register_struct(struct_def);
        }
        
        // Process global variables
        for global in &program.globals {
            self.process_global(global);
        }
        
        // Generate code for each function
        for function in &program.functions {
            self.generate_function(function);
        }
        
        // Add string literals at the end
        if !self.strings.is_empty() {
            self.emit_line("");
            self.emit_line(".section __TEXT,__cstring,cstring_literals");
            
            // Create a copy of the strings to avoid borrowing issues
            let strings_copy = self.strings.clone();
            for (i, string) in strings_copy.iter().enumerate() {
                self.emit_line(&format!("L.str.{}:", i));
                // Escape the string before formatting
                let escaped = Self::escape_string(string);
                self.emit_line(&format!("    .asciz \"{}\"", escaped));
            }
        }

        self.output.clone()
    }
    
    fn register_struct(&mut self, struct_def: &Struct) {
        let mut offset = 0;
        let mut field_info = Vec::new();
        
        for field in &struct_def.fields {
            let size = self.get_type_size(&field.data_type);
            let alignment = self.get_type_alignment(&field.data_type);
            
            // Apply field alignment
            offset = (offset + alignment - 1) / alignment * alignment;
            
            field_info.push((field.name.clone(), field.data_type.clone(), offset));
            offset += size;
        }
        
        self.structs.insert(struct_def.name.clone(), field_info);
    }
    
    fn get_type_size(&self, typ: &Type) -> usize {
        match typ {
            Type::Void => 0,
            Type::Bool => 1,
            Type::Char | Type::UnsignedChar => 1,
            Type::Short | Type::UnsignedShort => 2,
            Type::Int | Type::UnsignedInt => 4,
            Type::Long | Type::UnsignedLong => 8,
            Type::LongLong | Type::UnsignedLongLong => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::Pointer(_) => 8, // 64-bit pointers
            Type::Array(elem_type, Some(size)) => elem_type.size() * size,
            Type::Array(elem_type, None) => elem_type.size(), // VLA
            Type::Struct(name) => {
                // Get the struct size from the registry
                if let Some(fields) = self.structs.get(name) {
                    if fields.is_empty() {
                        0
                    } else {
                        // Last field offset + last field size
                        let (_, field_type, offset) = fields.last().unwrap();
                        offset + self.get_type_size(field_type)
                    }
                } else {
                    // Unknown struct, assume size 0
                    0
                }
            },
            Type::Union(name) => {
                // For unions, size is the size of the largest field
                if let Some(fields) = self.structs.get(name) {
                    fields
                        .iter()
                        .map(|(_, field_type, _)| self.get_type_size(field_type))
                        .max()
                        .unwrap_or(0)
                } else {
                    // Unknown union, assume size 0
                    0
                }
            },
            Type::Function { .. } => 8, // Function pointers are 8 bytes
            Type::Const(inner) => self.get_type_size(inner),
            Type::Volatile(inner) => self.get_type_size(inner),
            Type::Restrict(inner) => self.get_type_size(inner),
            Type::TypeDef(_) => 8, // Assume 8 bytes, should be resolved during semantic analysis
            Type::Complex => 16,    // _Complex
            Type::Imaginary => 8,   // _Imaginary
            Type::Atomic(inner) => self.get_type_size(inner),   // _Atomic type - C11
            Type::Generic { .. } => 8,   // Generic type
        }
    }
    
    fn get_type_alignment(&self, typ: &Type) -> usize {
        match typ {
            Type::Void => 1,
            Type::Bool => 1,
            Type::Char | Type::UnsignedChar => 1,
            Type::Short | Type::UnsignedShort => 2,
            Type::Int | Type::UnsignedInt => 4,
            Type::Long | Type::UnsignedLong => 8,
            Type::LongLong | Type::UnsignedLongLong => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::Pointer(_) => 8,
            Type::Array(elem_type, _) => self.get_type_alignment(elem_type),
            Type::Struct(_) => 8, // Simplified, should use field alignments
            Type::Union(_) => 8,  // Simplified, should use field alignments
            Type::Function { .. } => 8,
            Type::Const(inner) => self.get_type_alignment(inner),
            Type::Volatile(inner) => self.get_type_alignment(inner),
            Type::Restrict(inner) => self.get_type_alignment(inner),
            Type::TypeDef(_) => 8,
            Type::Complex => 16,    // _Complex
            Type::Imaginary => 8,   // _Imaginary
            Type::Atomic(inner) => self.get_type_alignment(inner),   // _Atomic type - C11
            Type::Generic { .. } => 8,   // Generic type
        }
    }
    
    fn process_global(&mut self, global: &Statement) {
        match global {
            Statement::VariableDeclaration { name, data_type, initializer, is_global, alignment: _ } => {
                if !is_global {
                    return;
                }
                
                // Start the data section if not already
                self.emit_line("");
                self.emit_line(".section __DATA,__data");
                self.emit_line(&format!(".globl _{}", name));
                self.emit_line(&format!("_{}: ", name));
                
                // Generate initial value based on the type
                if let Some(typ) = data_type {
                    match typ {
                        Type::Int => {
                            if let Expression::IntegerLiteral(value) = initializer {
                                self.emit_line(&format!("    .long {}", value));
                            } else {
                                self.emit_line("    .long 0");
                            }
                        }
                        Type::Char => {
                            if let Expression::CharLiteral(value) = initializer {
                                self.emit_line(&format!("    .byte {}", *value as u8));
                            } else {
                                self.emit_line("    .byte 0");
                            }
                        }
                        Type::Pointer(_) => {
                            self.emit_line("    .quad 0");
                        }
                        _ => {
                            // Default to 8 bytes of zeros for other types
                            self.emit_line("    .quad 0");
                        }
                    }
                } else {
                    // No type specified, assume int
                    self.emit_line("    .long 0");
                }
            }
            _ => {
                // Only variable declarations can be global
            }
        }
    }

    fn generate_function(&mut self, function: &Function) {
        // Reset function state
        self.variables.clear();
        self.current_stack_offset = 0;
        self.current_loop_start_label = None;
        self.current_loop_end_label = None;
        
        // Function label
        self.emit_line("");
        self.emit_line(&format!(".globl _{}", function.name));
        self.emit_line(&format!("_{}: ", function.name));

        // Function prologue
        self.emit_line("    push %rbp");
        self.emit_line("    mov %rsp, %rbp");

        // First pass: analyze the function to determine stack space needed
        let mut stack_size = 0;
        
        // Account for parameters
        for (i, param) in function.parameters.iter().enumerate() {
            if i < 6 {  // First 6 parameters are in registers
                stack_size += 8;  // 8 bytes per parameter
            }
        }
        
        // Account for local variables by analyzing the function body
        stack_size += self.calculate_stack_size(&function.body);
        
        // Align stack to 16 bytes (ABI requirement)
        if stack_size % 16 != 0 {
            stack_size = (stack_size / 16 + 1) * 16;
        }
        
        // Reserve stack space for parameters and local variables
        if stack_size > 0 {
            self.emit_line(&format!("    sub ${}, %rsp", stack_size));
        }
        
        // Store parameter values in the stack
        for (i, param) in function.parameters.iter().enumerate() {
            // The first 6 parameters use registers in System V ABI
            // (rdi, rsi, rdx, rcx, r8, r9)
            let reg = match i {
                0 => "%rdi",
                1 => "%rsi",
                2 => "%rdx",
                3 => "%rcx",
                4 => "%r8",
                5 => "%r9",
                _ => continue, // Stack parameters not handled yet
            };
            
            // Allocate stack space for the parameter
            self.current_stack_offset -= 8;
            self.variables.insert(param.name.clone(), self.current_stack_offset);
            self.emit_line(&format!("    mov {}, {}(%rbp)", reg, self.current_stack_offset));
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

    // Calculate the stack size needed for a list of statements
    fn calculate_stack_size(&self, statements: &[Statement]) -> usize {
        let mut size = 0;
        
        for stmt in statements {
            match stmt {
                Statement::VariableDeclaration { .. } => {
                    // Each variable takes 8 bytes
                    size += 8;
                }
                Statement::ArrayDeclaration { size: Some(expr), .. } => {
                    // For array declarations with constant size
                    if let Expression::IntegerLiteral(n) = expr {
                        size += (*n as usize) * 8;  // 8 bytes per element
                    } else {
                        // For non-constant sizes, allocate a reasonable default
                        size += 64;  // Default to 8 elements
                    }
                }
                Statement::Block(block_stmts) => {
                    // Recursively calculate size for nested blocks
                    size += self.calculate_stack_size(block_stmts);
                }
                Statement::If { then_block, else_block, .. } => {
                    // Calculate size for both branches and take the maximum
                    let then_size = self.calculate_stack_size(&[*then_block.clone()]);
                    let else_size = if let Some(else_stmt) = else_block {
                        self.calculate_stack_size(&[*else_stmt.clone()])
                    } else {
                        0
                    };
                    size += then_size.max(else_size);
                }
                Statement::While { body, .. } | Statement::DoWhile { body, .. } => {
                    size += self.calculate_stack_size(&[*body.clone()]);
                }
                Statement::For { body, initializer, .. } => {
                    // Account for initializer if it's a variable declaration
                    if let Some(init) = initializer {
                        size += self.calculate_stack_size(&[*init.clone()]);
                    }
                    size += self.calculate_stack_size(&[*body.clone()]);
                }
                Statement::Switch { cases, .. } => {
                    // Calculate size for all cases and take the maximum
                    let mut max_case_size = 0;
                    for case in cases {
                        let case_size = self.calculate_stack_size(&case.statements);
                        max_case_size = max_case_size.max(case_size);
                    }
                    size += max_case_size;
                }
                _ => {} // Other statement types don't allocate stack space
            }
        }
        
        size
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
            Statement::VariableDeclaration { name, data_type, initializer, is_global, alignment: _ } => {
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
            Statement::ExpressionStatement(expr) => {
                self.generate_expression(expr);
                // Result is discarded
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.generate_statement(stmt);
                }
            }
            Statement::If { condition, then_block, else_block } => {
                let label_else = self.next_label("if_else");
                let label_end = self.next_label("if_end");
                
                // Generate condition code
                self.generate_expression(condition);
                self.emit_line("    cmp $0, %rax");
                
                if else_block.is_some() {
                    self.emit_line(&format!("    je {}", label_else));
                } else {
                    self.emit_line(&format!("    je {}", label_end));
                }
                
                // Then block
                self.generate_statement(then_block);
                
                if else_block.is_some() {
                    self.emit_line(&format!("    jmp {}", label_end));
                    self.emit_line(&format!("{}:", label_else));
                    self.generate_statement(else_block.as_ref().unwrap());
                }
                
                self.emit_line(&format!("{}:", label_end));
            }
            Statement::While { condition, body } => {
                let label_start = self.next_label("while_start");
                let label_end = self.next_label("while_end");
                
                // Save the previous loop labels
                let prev_start = self.current_loop_start_label.clone();
                let prev_end = self.current_loop_end_label.clone();
                
                // Set current loop labels for break/continue
                self.current_loop_start_label = Some(label_start.clone());
                self.current_loop_end_label = Some(label_end.clone());
                
                self.emit_line(&format!("{}:", label_start));
                
                // Generate condition code
                self.generate_expression(condition);
                self.emit_line("    cmp $0, %rax");
                self.emit_line(&format!("    je {}", label_end));
                
                // Loop body
                self.generate_statement(body);
                
                // Jump back to start
                self.emit_line(&format!("    jmp {}", label_start));
                self.emit_line(&format!("{}:", label_end));
                
                // Restore previous loop labels
                self.current_loop_start_label = prev_start;
                self.current_loop_end_label = prev_end;
            }
            Statement::For { body, initializer, condition, increment } => {
                let label_start = self.next_label("for_start");
                let label_check = self.next_label("for_check");
                let label_end = self.next_label("for_end");
                
                // Save the previous loop labels
                let prev_start = self.current_loop_start_label.clone();
                let prev_end = self.current_loop_end_label.clone();
                
                // Set current loop labels for break/continue
                self.current_loop_start_label = Some(label_check.clone());
                self.current_loop_end_label = Some(label_end.clone());
                
                // Initializer
                if let Some(init) = initializer {
                    self.generate_statement(init);
                }
                
                self.emit_line(&format!("    jmp {}", label_check));
                self.emit_line(&format!("{}:", label_start));
                
                // Loop body
                self.generate_statement(body);
                
                // Increment
                if let Some(inc) = increment {
                    self.generate_expression(inc);
                }
                
                // Condition check
                self.emit_line(&format!("{}:", label_check));
                if let Some(cond) = condition {
                    self.generate_expression(cond);
                    self.emit_line("    cmp $0, %rax");
                    self.emit_line(&format!("    jne {}", label_start));
                } else {
                    // No condition means loop forever (until break)
                    self.emit_line(&format!("    jmp {}", label_start));
                }
                
                self.emit_line(&format!("{}:", label_end));
                
                // Restore previous loop labels
                self.current_loop_start_label = prev_start;
                self.current_loop_end_label = prev_end;
            }
            Statement::Break => {
                if let Some(label) = &self.current_loop_end_label {
                    self.emit_line(&format!("    jmp {}", label));
                }
            }
            Statement::Continue => {
                if let Some(label) = &self.current_loop_start_label {
                    self.emit_line(&format!("    jmp {}", label));
                }
            }
            _ => {
                // Other statement types not yet implemented
                self.emit_line(&format!("    # Unimplemented statement: {:?}", statement));
            }
        }
    }

    fn generate_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::IntegerLiteral(value) => {
                self.emit_line(&format!("    mov ${}, %rax", value));
            }
            Expression::StringLiteral(value) => {
                // Add string to literals section and store the index
                let string_index = self.strings.len();
                self.strings.push(value.clone());
                
                // Load the address of the string
                self.emit_line(&format!("    leaq L.str.{}(%rip), %rax", string_index));
            }
            Expression::CharLiteral(value) => {
                self.emit_line(&format!("    mov ${}, %rax", *value as u8));
            }
            Expression::Variable(name) => {
                if let Some(offset) = self.variables.get(name) {
                    self.emit_line(&format!("    mov {}(%rbp), %rax", offset));
                } else {
                    // Could be a global variable
                    self.emit_line(&format!("    mov _{}(%rip), %rax", name));
                }
            }
            Expression::BinaryOperation {
                left,
                operator,
                right,
            } => {
                match operator {
                    // Simple operations
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide |
                    BinaryOp::Modulo | BinaryOp::BitwiseAnd | BinaryOp::BitwiseOr | BinaryOp::BitwiseXor |
                    BinaryOp::LeftShift | BinaryOp::RightShift => {
                        // Generate right operand first and push to stack
                        self.generate_expression(right);
                        self.emit_line("    push %rax");
                        
                        // Generate left operand into %rax
                        self.generate_expression(left);
                        
                        // Move right operand to %rcx
                        self.emit_line("    pop %rcx");
                        
                        // Perform operation
                        match operator {
                            BinaryOp::Add => self.emit_line("    add %rcx, %rax"),
                            BinaryOp::Subtract => self.emit_line("    sub %rcx, %rax"),
                            BinaryOp::Multiply => self.emit_line("    imul %rcx, %rax"),
                            BinaryOp::Divide => {
                                self.emit_line("    cqo"); // Sign-extend RAX into RDX:RAX
                                self.emit_line("    idiv %rcx");
                            },
                            BinaryOp::Modulo => {
                                self.emit_line("    cqo"); // Sign-extend RAX into RDX:RAX
                                self.emit_line("    idiv %rcx");
                                self.emit_line("    mov %rdx, %rax"); // Remainder is in %rdx
                            },
                            BinaryOp::BitwiseAnd => self.emit_line("    and %rcx, %rax"),
                            BinaryOp::BitwiseOr => self.emit_line("    or %rcx, %rax"),
                            BinaryOp::BitwiseXor => self.emit_line("    xor %rcx, %rax"),
                            BinaryOp::LeftShift => self.emit_line("    shl %cl, %eax"),
                            BinaryOp::RightShift => self.emit_line("    sar %cl, %eax"),
                            _ => unreachable!(),
                        }
                    },
                    
                    // Comparison operations
                    BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::LessThan | 
                    BinaryOp::LessThanOrEqual | BinaryOp::GreaterThan | BinaryOp::GreaterThanOrEqual => {
                        // Generate right operand first and push to stack
                        self.generate_expression(right);
                        self.emit_line("    push %rax");
                        
                        // Generate left operand into %rax
                        self.generate_expression(left);
                        
                        // Move right operand to %rcx
                        self.emit_line("    pop %rcx");
                        
                        // Compare left and right
                        self.emit_line("    cmp %rcx, %rax");
                        
                        // Set result based on comparison
                        match operator {
                            BinaryOp::Equal => self.emit_line("    sete %al"),
                            BinaryOp::NotEqual => self.emit_line("    setne %al"),
                            BinaryOp::LessThan => self.emit_line("    setl %al"),
                            BinaryOp::LessThanOrEqual => self.emit_line("    setle %al"),
                            BinaryOp::GreaterThan => self.emit_line("    setg %al"),
                            BinaryOp::GreaterThanOrEqual => self.emit_line("    setge %al"),
                            _ => unreachable!(),
                        }
                        
                        // Zero-extend result to 64 bits
                        self.emit_line("    movzx %al, %rax");
                    },
                    
                    // Logical operations with short-circuit behavior
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
                        let end_label = self.next_label("logical_end");
                        
                        // Generate left operand
                        self.generate_expression(left);
                        
                        if matches!(operator, BinaryOp::LogicalAnd) {
                            // Short-circuit if left is false
                            self.emit_line("    cmp $0, %rax");
                            self.emit_line(&format!("    je {}", end_label));
                        } else {
                            // Short-circuit if left is true (non-zero)
                            self.emit_line("    cmp $0, %rax");
                            self.emit_line(&format!("    jne {}", end_label));
                        }
                        
                        // Generate right operand
                        self.generate_expression(right);
                        
                        // For LogicalAnd, result is already correct
                        // For LogicalOr, we need to ensure it's 1 if non-zero
                        if matches!(operator, BinaryOp::LogicalOr) {
                            self.emit_line("    cmp $0, %rax");
                            self.emit_line("    setne %al");
                            self.emit_line("    movzx %al, %rax");
                        }
                        
                        self.emit_line(&format!("{}:", end_label));
                    },
                    _ => {
                        // Unsupported binary operation
                        self.emit_line(&format!("    # Unsupported binary op: {:?}", operator));
                        // Default to 0
                        self.emit_line("    xor %rax, %rax");
                    }
                }
            }
            Expression::UnaryOperation { operator, operand } => {
                // Generate operand value first
                self.generate_expression(operand);
                
                match operator {
                    OperatorType::Unary(UnaryOp::Negate) => {
                        self.emit_line("    neg %rax");
                    },
                    OperatorType::Unary(UnaryOp::LogicalNot) => {
                        self.emit_line("    cmp $0, %rax");
                        self.emit_line("    sete %al");
                        self.emit_line("    movzx %al, %rax");
                    },
                    OperatorType::Unary(UnaryOp::BitwiseNot) => {
                        self.emit_line("    not %rax");
                    },
                    OperatorType::Unary(UnaryOp::AddressOf) => {
                        // This would require tracking where values are stored
                        // For now, it's a placeholder that depends on operand type
                        self.emit_line("    # Address-of operator not fully implemented");
                    },
                    OperatorType::Unary(UnaryOp::Dereference) => {
                        // Dereference a pointer (load from address in %rax)
                        self.emit_line("    mov (%rax), %rax");
                    },
                    OperatorType::Unary(UnaryOp::PreIncrement) | OperatorType::Unary(UnaryOp::PostIncrement) => {
                        // For post increment, we need to save the original value
                        if matches!(operator, OperatorType::Unary(UnaryOp::PostIncrement)) {
                            self.emit_line("    mov %rax, %rcx"); // Save original value
                        }
                        
                        // Generate increment operation (depends on variable location)
                        if let Expression::Variable(name) = operand.as_ref() {
                            // Get a copy of the offset to avoid borrowing issues
                            let offset = if let Some(&offset) = self.variables.get(name) {
                                offset
                            } else {
                                0 // Default if not found
                            };
                            
                            if offset != 0 {
                                self.emit_line(&format!("    add $1, {}(%rbp)", offset));
                                
                                if matches!(operator, OperatorType::Unary(UnaryOp::PreIncrement)) {
                                    self.emit_line(&format!("    mov {}(%rbp), %rax", offset));
                                }
                            }
                        }
                        
                        // For post increment, restore the original value
                        if matches!(operator, OperatorType::Unary(UnaryOp::PostIncrement)) {
                            self.emit_line("    mov %rcx, %rax");
                        }
                    },
                    OperatorType::Unary(UnaryOp::PreDecrement) | OperatorType::Unary(UnaryOp::PostDecrement) => {
                        // Similar to increment
                        if matches!(operator, OperatorType::Unary(UnaryOp::PostDecrement)) {
                            self.emit_line("    mov %rax, %rcx"); // Save original value
                        }
                        
                        if let Expression::Variable(name) = operand.as_ref() {
                            // Get a copy of the offset to avoid borrowing issues
                            let offset = if let Some(&offset) = self.variables.get(name) {
                                offset
                            } else {
                                0 // Default if not found
                            };
                            
                            if offset != 0 {
                                self.emit_line(&format!("    sub $1, {}(%rbp)", offset));
                                
                                if matches!(operator, OperatorType::Unary(UnaryOp::PreDecrement)) {
                                    self.emit_line(&format!("    mov {}(%rbp), %rax", offset));
                                }
                            }
                        }
                        
                        if matches!(operator, OperatorType::Unary(UnaryOp::PostDecrement)) {
                            self.emit_line("    mov %rcx, %rax");
                        }
                    },
                    _ => {
                        self.emit_line(&format!("    # Unsupported unary op: {:?}", operator));
                    }
                }
            }
            Expression::Assignment { target, value } => {
                // Generate the value to assign
                self.generate_expression(value);
                
                match target.as_ref() {
                    Expression::Variable(name) => {
                        if let Some(offset) = self.variables.get(name) {
                            // Local variable
                            self.emit_line(&format!("    mov %rax, {}(%rbp)", offset));
                        } else {
                            // Global variable
                            self.emit_line(&format!("    mov %rax, _{}(%rip)", name));
                        }
                    }
                    Expression::ArrayAccess { array, index } => {
                        // Calculate address of the array element
                        // For optimization, save the value to assign
                        self.emit_line("    push %rax");
                        
                        // Generate array base address
                        self.generate_expression(array);
                        self.emit_line("    mov %rax, %rcx");
                        
                        // Generate index and scale by element size (assume 8 bytes)
                        self.generate_expression(index);
                        self.emit_line("    shl $3, %rax"); // Scale by 8 (element size)
                        
                        // Calculate effective address
                        self.emit_line("    add %rcx, %rax");
                        
                        // Restore value to assign and store it
                        self.emit_line("    mov %rax, %rcx");
                        self.emit_line("    pop %rax");
                        self.emit_line("    mov %rax, (%rcx)");
                    }
                    Expression::PointerFieldAccess { pointer, field } => {
                        // Save the value to assign
                        self.emit_line("    push %rax");
                        
                        // Generate pointer address
                        self.generate_expression(pointer);
                        
                        // Find the field offset (simplified)
                        // In a real implementation, we'd need to access the struct info
                        self.emit_line(&format!("    # Accessing field {} (offset hardcoded)", field));
                        self.emit_line("    add $8, %rax"); // Placeholder offset
                        
                        // Restore value and store
                        self.emit_line("    mov %rax, %rcx");
                        self.emit_line("    pop %rax");
                        self.emit_line("    mov %rax, (%rcx)");
                    }
                    _ => {
                        self.emit_line("    # Unsupported assignment target");
                    }
                }
                
                // Assignment expression returns the assigned value, which is already in %rax
            }
            Expression::FunctionCall { name, arguments } => {
                // Save caller-saved registers that we're using
                let arg_count = arguments.len();
                if arg_count > 0 {
                    self.emit_line("    # Save caller-saved registers");
                    self.emit_line("    push %rcx");
                    self.emit_line("    push %rdx");
                    self.emit_line("    push %rsi");
                    self.emit_line("    push %rdi");
                    self.emit_line("    push %r8");
                    self.emit_line("    push %r9");
                }
                
                // Evaluate arguments in reverse order (for stack args)
                for (i, arg) in arguments.iter().enumerate().rev() {
                    self.generate_expression(arg);
                    
                    // First 6 args go in registers, rest on stack
                    match i {
                        0 => self.emit_line("    mov %rax, %rdi"),
                        1 => self.emit_line("    mov %rax, %rsi"),
                        2 => self.emit_line("    mov %rax, %rdx"),
                        3 => self.emit_line("    mov %rax, %rcx"),
                        4 => self.emit_line("    mov %rax, %r8"),
                        5 => self.emit_line("    mov %rax, %r9"),
                        _ => self.emit_line("    push %rax"), // Stack arg
                    }
                }
                
                // Call the function
                self.emit_line(&format!("    call _{}", name));
                
                // Clean up stack arguments
                if arg_count > 6 {
                    let stack_arg_count = arg_count - 6;
                    self.emit_line(&format!("    add ${}, %rsp", stack_arg_count * 8));
                }
                
                // Restore caller-saved registers
                if arg_count > 0 {
                    self.emit_line("    # Restore caller-saved registers");
                    self.emit_line("    pop %r9");
                    self.emit_line("    pop %r8");
                    self.emit_line("    pop %rdi");
                    self.emit_line("    pop %rsi");
                    self.emit_line("    pop %rdx");
                    self.emit_line("    pop %rcx");
                }
                
                // Result is already in %rax
            }
            Expression::ArrayAccess { array, index } => {
                // Generate array base address
                self.generate_expression(array);
                self.emit_line("    mov %rax, %rcx");
                
                // Generate index and scale by element size (assume 8 bytes)
                self.generate_expression(index);
                self.emit_line("    shl $3, %rax"); // Scale by 8 (element size)
                
                // Calculate effective address
                self.emit_line("    add %rcx, %rax");
                
                // Load value from calculated address
                self.emit_line("    mov (%rax), %rax");
            }
            Expression::TernaryIf { condition, then_expr, else_expr } => {
                let label_else = self.next_label("ternary_else");
                let label_end = self.next_label("ternary_end");
                
                // Generate condition
                self.generate_expression(condition);
                self.emit_line("    cmp $0, %rax");
                self.emit_line(&format!("    je {}", label_else));
                
                // Generate then expression
                self.generate_expression(then_expr);
                self.emit_line(&format!("    jmp {}", label_end));
                
                // Generate else expression
                self.emit_line(&format!("{}:", label_else));
                self.generate_expression(else_expr);
                
                self.emit_line(&format!("{}:", label_end));
            }
            _ => {
                // Other expression types not yet implemented
                self.emit_line(&format!("    # Unimplemented expression: {:?}", expr));
                self.emit_line("    mov $0, %rax"); // Default to 0
            }
        }
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let label = format!(".L{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }
    
    fn escape_string(s: &str) -> String {
        s.replace("\\", "\\\\")
         .replace("\n", "\\n")
         .replace("\t", "\\t")
         .replace("\"", "\\\"")
         .replace("\0", "\\0")
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

