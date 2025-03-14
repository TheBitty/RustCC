#[cfg(feature = "llvm-backend")]
use crate::parser::ast::{BinaryOp, Expression, Function, Program, Statement, Type};
#[cfg(feature = "llvm-backend")]
use inkwell::builder::Builder;
#[cfg(feature = "llvm-backend")]
use inkwell::context::Context;
#[cfg(feature = "llvm-backend")]
use inkwell::module::Module;
#[cfg(feature = "llvm-backend")]
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
#[cfg(feature = "llvm-backend")]
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
#[cfg(feature = "llvm-backend")]
use inkwell::AddressSpace;
#[cfg(feature = "llvm-backend")]
use std::collections::HashMap;
#[cfg(feature = "llvm-backend")]
use std::path::Path;

#[cfg(feature = "llvm-backend")]
#[allow(dead_code)]
pub struct LLVMCodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

#[cfg(feature = "llvm-backend")]
#[allow(dead_code)]
impl<'ctx> LLVMCodeGenerator<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        LLVMCodeGenerator {
            context,
            module: context.create_module(module_name),
            builder: context.create_builder(),
            variables: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<(), String> {
        // Generate code for each function
        for function in &program.functions {
            self.compile_function(function)?;
        }

        // Verify the module
        if self.module.verify().is_err() {
            return Err("Failed to verify LLVM module".to_string());
        }

        Ok(())
    }

    pub fn get_llvm_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn write_to_file(&self, filename: &str) -> Result<(), String> {
        // Updated to handle boolean return value instead of Result
        // Use explicit Path conversion
        let path = Path::new(filename);
        if !self.module.write_bitcode_to_path(path) {
            return Err(format!("Failed to write LLVM bitcode to {}", filename));
        }
        Ok(())
    }

    fn compile_function(&mut self, function: &Function) -> Result<FunctionValue<'ctx>, String> {
        // Create function return type
        let return_type = self.convert_type(&function.return_type)?;

        // Create function parameter types - convert to BasicMetadataTypeEnum
        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = function
            .parameters
            .iter()
            .map(|param| self.convert_type(&param.data_type))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|ty| ty.into()) // Convert BasicTypeEnum to BasicMetadataTypeEnum
            .collect();

        // Create LLVM function type
        let fn_type = match return_type {
            BasicTypeEnum::IntType(int_type) => int_type.fn_type(&param_types, false),
            // Add other types as needed
            _ => return Err("Unsupported return type".to_string()),
        };

        // Create the function
        let function_value = self.module.add_function(&function.name, fn_type, None);

        // Create a basic block for the function
        let basic_block = self.context.append_basic_block(function_value, "entry");
        self.builder.position_at_end(basic_block);

        // Clear variables from previous compilation
        self.variables.clear();

        // Allocate space for parameters
        for (i, param) in function.parameters.iter().enumerate() {
            let param_value = function_value
                .get_nth_param(i as u32)
                .ok_or_else(|| format!("Failed to get parameter {}", i))?;

            let alloca = self.create_entry_block_alloca(&param.name, param_value.get_type())?;
            self.builder
                .build_store(alloca, param_value)
                .map_err(|e| format!("Failed to store parameter {}: {}", param.name, e))?;
            self.variables.insert(param.name.clone(), alloca);
        }

        // Compile the function body
        for statement in &function.body {
            self.compile_statement(statement, function_value)?;
        }

        // Verify the function
        if function_value.verify(true) {
            Ok(function_value)
        } else {
            Err(format!("Failed to verify function {}", function.name))
        }
    }

    fn create_entry_block_alloca(
        &self,
        name: &str,
        ty: BasicTypeEnum<'ctx>,
    ) -> Result<PointerValue<'ctx>, String> {
        let builder = self.context.create_builder();
        let entry = self
            .builder
            .get_insert_block()
            .ok_or_else(|| "No current block to insert into".to_string())?;

        // Replace position_at_start with position_at
        // If entry has no instructions yet, use position_at_end instead
        let first_instruction = entry.get_first_instruction();
        if let Some(instr) = first_instruction {
            builder.position_before(&instr);
        } else {
            builder.position_at_end(entry);
        }

        // Add proper error handling with expect
        let alloca = builder
            .build_alloca(ty, name)
            .map_err(|e| format!("Failed to allocate memory: {}", e))?;

        Ok(alloca)
    }

    fn compile_statement(
        &mut self,
        statement: &Statement,
        _function: FunctionValue<'ctx>,
    ) -> Result<(), String> {
        match statement {
            Statement::Return(expr) => {
                let return_value = self.compile_expression(expr)?;
                self.builder
                    .build_return(Some(&return_value))
                    .map_err(|e| format!("Failed to build return: {}", e))?;
            }
            Statement::VariableDeclaration {
                name,
                data_type: _,
                initializer,
            } => {
                let init_val = self.compile_expression(initializer)?;
                let alloca = self.create_entry_block_alloca(name, init_val.get_type())?;
                self.builder
                    .build_store(alloca, init_val)
                    .map_err(|e| format!("Failed to store variable {}: {}", name, e))?;
                self.variables.insert(name.clone(), alloca);
            }
            // Implement other statement types
            _ => return Err("Statement type not yet implemented for LLVM".to_string()),
        }

        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expression::IntegerLiteral(value) => {
                let int_type = self.context.i32_type();
                Ok(int_type.const_int(*value as u64, true).into())
            }
            Expression::Variable(name) => {
                if let Some(ptr) = self.variables.get(name) {
                    // Updated to use the correct build_load signature
                    // Use the IntType directly without converting to BasicTypeEnum
                    let int_type = self.context.i32_type();
                    let load_value = self
                        .builder
                        .build_load::<inkwell::types::IntType>(
                            int_type,
                            *ptr,
                            &format!("{}_val", name),
                        )
                        .map_err(|e| format!("Failed to load variable {}: {}", name, e))?;

                    Ok(load_value)
                } else {
                    Err(format!("Variable {} not found", name))
                }
            }
            Expression::BinaryOperation {
                left,
                operator,
                right,
            } => {
                let lhs = self.compile_expression(left)?;
                let rhs = self.compile_expression(right)?;

                // Ensure proper type conversions between BasicValueEnum and IntValue
                if let (BasicValueEnum::IntValue(lhs_val), BasicValueEnum::IntValue(rhs_val)) =
                    (lhs, rhs)
                {
                    let result = match operator {
                        BinaryOp::Add => self
                            .builder
                            .build_int_add(lhs_val, rhs_val, "addtmp")
                            .map_err(|e| format!("Failed to build addition: {}", e))?,
                        BinaryOp::Subtract => self
                            .builder
                            .build_int_sub(lhs_val, rhs_val, "subtmp")
                            .map_err(|e| format!("Failed to build subtraction: {}", e))?,
                        BinaryOp::Multiply => self
                            .builder
                            .build_int_mul(lhs_val, rhs_val, "multmp")
                            .map_err(|e| format!("Failed to build multiplication: {}", e))?,
                        BinaryOp::Divide => self
                            .builder
                            .build_int_signed_div(lhs_val, rhs_val, "divtmp")
                            .map_err(|e| format!("Failed to build division: {}", e))?,
                        // Implement other operators as needed
                        _ => return Err("Binary operator not yet implemented for LLVM".to_string()),
                    };
                    Ok(result.into())
                } else {
                    Err("Type mismatch in binary operation".to_string())
                }
            }
            // Implement other expression types
            _ => Err("Expression type not yet implemented for LLVM".to_string()),
        }
    }

    fn convert_type(&self, ty: &Type) -> Result<BasicTypeEnum<'ctx>, String> {
        match ty {
            Type::Int => Ok(self.context.i32_type().into()),
            Type::Char => Ok(self.context.i8_type().into()),
            Type::Pointer(_) => {
                // Use context.ptr_type directly instead of pointee_type.ptr_type
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }
            // Add other type conversions as needed
            _ => Err("Type not yet implemented for LLVM".to_string()),
        }
    }
}
