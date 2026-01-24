//! LLVM IR code generation for ARIA.
//!
//! This module lowers ARIA MIR to LLVM IR using the inkwell crate.
//!
//! # Supported Features
//! - Integer arithmetic
//! - Boolean operations
//! - Function calls
//! - Control flow (if/else, while)
//! - Local variables
//!
//! # Usage
//! ```ignore
//! use aria::codegen::LLVMCodegen;
//! use aria::mir::Program;
//!
//! let codegen = LLVMCodegen::new("my_module");
//! codegen.compile(&program)?;
//! codegen.write_object_file("output.o")?;
//! ```

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::OptimizationLevel;
use inkwell::{AddressSpace, IntPredicate};
use std::collections::HashMap;
use std::path::Path;

use crate::mir::{BinOp, Block, Function, Operand, Program, Rvalue, Statement, Terminator, Ty};

/// Error during LLVM code generation.
#[derive(Debug)]
pub struct CodegenError {
    pub message: String,
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "codegen error: {}", self.message)
    }
}

impl std::error::Error for CodegenError {}

/// LLVM code generator for ARIA programs.
pub struct LLVMCodegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    /// Map from MIR function names to LLVM functions
    functions: HashMap<String, FunctionValue<'ctx>>,
    /// Map from local variable indices to stack allocations
    locals: HashMap<usize, PointerValue<'ctx>>,
    /// Current function being compiled
    current_function: Option<FunctionValue<'ctx>>,
}

impl<'ctx> LLVMCodegen<'ctx> {
    /// Create a new code generator with the given module name.
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            functions: HashMap::new(),
            locals: HashMap::new(),
            current_function: None,
        }
    }

    /// Compile a MIR program to LLVM IR.
    pub fn compile(&mut self, program: &Program) -> Result<(), CodegenError> {
        // First pass: declare all functions
        for func in &program.functions {
            self.declare_function(func)?;
        }

        // Second pass: compile function bodies
        for func in &program.functions {
            self.compile_function(func)?;
        }

        Ok(())
    }

    /// Declare a function (create signature without body).
    fn declare_function(&mut self, func: &Function) -> Result<(), CodegenError> {
        let return_type = self.lower_type(&func.return_ty)?;
        let param_types: Vec<BasicMetadataTypeEnum> = func
            .params
            .iter()
            .map(|p| self.lower_type(&p.ty).map(|t| t.into()))
            .collect::<Result<Vec<_>, _>>()?;

        let fn_type = match return_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            _ => {
                // Default to i64 for unknown types
                self.context.i64_type().fn_type(&param_types, false)
            }
        };

        let fn_value = self.module.add_function(&func.name, fn_type, None);
        self.functions.insert(func.name.clone(), fn_value);

        Ok(())
    }

    /// Compile a function body.
    fn compile_function(&mut self, func: &Function) -> Result<(), CodegenError> {
        let fn_value = self
            .functions
            .get(&func.name)
            .copied()
            .ok_or_else(|| CodegenError {
                message: format!("Function {} not declared", func.name),
            })?;

        self.current_function = Some(fn_value);
        self.locals.clear();

        // Create entry block
        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        // Allocate locals
        for (i, local) in func.locals.iter().enumerate() {
            let ty = self.lower_type(&local.ty)?;
            let alloca = self.builder.build_alloca(ty, &format!("local_{}", i))
                .map_err(|e| CodegenError { message: format!("alloca failed: {:?}", e) })?;
            self.locals.insert(i, alloca);
        }

        // Store function parameters into their locals
        for (i, param) in fn_value.get_param_iter().enumerate() {
            if let Some(alloca) = self.locals.get(&i) {
                self.builder.build_store(*alloca, param)
                    .map_err(|e| CodegenError { message: format!("store failed: {:?}", e) })?;
            }
        }

        // Create basic blocks for each MIR block
        let mut blocks: HashMap<usize, inkwell::basic_block::BasicBlock> = HashMap::new();
        for (i, _) in func.blocks.iter().enumerate() {
            let bb = self
                .context
                .append_basic_block(fn_value, &format!("bb_{}", i));
            blocks.insert(i, bb);
        }

        // Jump from entry to first block
        if let Some(&first_block) = blocks.get(&0) {
            self.builder.build_unconditional_branch(first_block)
                .map_err(|e| CodegenError { message: format!("branch failed: {:?}", e) })?;
        }

        // Compile each block
        for (i, block) in func.blocks.iter().enumerate() {
            if let Some(&bb) = blocks.get(&i) {
                self.builder.position_at_end(bb);
                self.compile_block(block, &blocks)?;
            }
        }

        self.current_function = None;
        Ok(())
    }

    /// Compile a basic block.
    fn compile_block(
        &mut self,
        block: &Block,
        blocks: &HashMap<usize, inkwell::basic_block::BasicBlock>,
    ) -> Result<(), CodegenError> {
        // Compile statements
        for stmt in &block.statements {
            self.compile_statement(stmt)?;
        }

        // Compile terminator
        self.compile_terminator(&block.terminator, blocks)?;

        Ok(())
    }

    /// Compile a statement.
    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), CodegenError> {
        match stmt {
            Statement::Assign(place, rvalue) => {
                let value = self.compile_rvalue(rvalue)?;
                if let Some(alloca) = self.locals.get(&place.local) {
                    self.builder.build_store(*alloca, value)
                        .map_err(|e| CodegenError { message: format!("store failed: {:?}", e) })?;
                }
            }
            Statement::Nop => {}
        }
        Ok(())
    }

    /// Compile an rvalue to produce a value.
    fn compile_rvalue(&mut self, rvalue: &Rvalue) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        match rvalue {
            Rvalue::Use(operand) => self.compile_operand(operand),
            Rvalue::BinaryOp(op, left, right) => {
                let lhs = self.compile_operand(left)?;
                let rhs = self.compile_operand(right)?;
                self.compile_binop(*op, lhs, rhs)
            }
            Rvalue::UnaryOp(op, operand) => {
                let val = self.compile_operand(operand)?;
                self.compile_unaryop(*op, val)
            }
            Rvalue::Call(func_name, args) => {
                let fn_value = self
                    .functions
                    .get(func_name)
                    .copied()
                    .ok_or_else(|| CodegenError {
                        message: format!("Unknown function: {}", func_name),
                    })?;

                let compiled_args: Vec<BasicMetadataTypeEnum> = args
                    .iter()
                    .map(|a| self.compile_operand(a).map(|v| v.into()))
                    .collect::<Result<Vec<_>, _>>()?;

                let call = self
                    .builder
                    .build_call(fn_value, &compiled_args, "call")
                    .map_err(|e| CodegenError { message: format!("call failed: {:?}", e) })?;

                call.try_as_basic_value()
                    .left()
                    .ok_or_else(|| CodegenError {
                        message: "Function returned void".into(),
                    })
            }
            _ => Err(CodegenError {
                message: format!("Unsupported rvalue: {:?}", rvalue),
            }),
        }
    }

    /// Compile an operand.
    fn compile_operand(&mut self, operand: &Operand) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                if let Some(alloca) = self.locals.get(&place.local) {
                    let ty = self.context.i64_type();
                    self.builder.build_load(ty, *alloca, "load")
                        .map_err(|e| CodegenError { message: format!("load failed: {:?}", e) })
                } else {
                    Err(CodegenError {
                        message: format!("Unknown local: {}", place.local),
                    })
                }
            }
            Operand::Constant(constant) => {
                // For now, assume all constants are i64
                let val = self.context.i64_type().const_int(*constant as u64, true);
                Ok(val.into())
            }
        }
    }

    /// Compile a binary operation.
    fn compile_binop(
        &mut self,
        op: BinOp,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let lhs_int = lhs.into_int_value();
        let rhs_int = rhs.into_int_value();

        let result: IntValue = match op {
            BinOp::Add => self.builder.build_int_add(lhs_int, rhs_int, "add")
                .map_err(|e| CodegenError { message: format!("add failed: {:?}", e) })?,
            BinOp::Sub => self.builder.build_int_sub(lhs_int, rhs_int, "sub")
                .map_err(|e| CodegenError { message: format!("sub failed: {:?}", e) })?,
            BinOp::Mul => self.builder.build_int_mul(lhs_int, rhs_int, "mul")
                .map_err(|e| CodegenError { message: format!("mul failed: {:?}", e) })?,
            BinOp::Div => self.builder.build_int_signed_div(lhs_int, rhs_int, "div")
                .map_err(|e| CodegenError { message: format!("div failed: {:?}", e) })?,
            BinOp::Mod => self.builder.build_int_signed_rem(lhs_int, rhs_int, "mod")
                .map_err(|e| CodegenError { message: format!("mod failed: {:?}", e) })?,
            BinOp::Eq => self.builder.build_int_compare(IntPredicate::EQ, lhs_int, rhs_int, "eq")
                .map_err(|e| CodegenError { message: format!("eq failed: {:?}", e) })?,
            BinOp::Ne => self.builder.build_int_compare(IntPredicate::NE, lhs_int, rhs_int, "ne")
                .map_err(|e| CodegenError { message: format!("ne failed: {:?}", e) })?,
            BinOp::Lt => self.builder.build_int_compare(IntPredicate::SLT, lhs_int, rhs_int, "lt")
                .map_err(|e| CodegenError { message: format!("lt failed: {:?}", e) })?,
            BinOp::Le => self.builder.build_int_compare(IntPredicate::SLE, lhs_int, rhs_int, "le")
                .map_err(|e| CodegenError { message: format!("le failed: {:?}", e) })?,
            BinOp::Gt => self.builder.build_int_compare(IntPredicate::SGT, lhs_int, rhs_int, "gt")
                .map_err(|e| CodegenError { message: format!("gt failed: {:?}", e) })?,
            BinOp::Ge => self.builder.build_int_compare(IntPredicate::SGE, lhs_int, rhs_int, "ge")
                .map_err(|e| CodegenError { message: format!("ge failed: {:?}", e) })?,
            BinOp::And => self.builder.build_and(lhs_int, rhs_int, "and")
                .map_err(|e| CodegenError { message: format!("and failed: {:?}", e) })?,
            BinOp::Or => self.builder.build_or(lhs_int, rhs_int, "or")
                .map_err(|e| CodegenError { message: format!("or failed: {:?}", e) })?,
            _ => {
                return Err(CodegenError {
                    message: format!("Unsupported binary operator: {:?}", op),
                })
            }
        };

        Ok(result.into())
    }

    /// Compile a unary operation.
    fn compile_unaryop(
        &mut self,
        op: crate::mir::UnaryOp,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let int_val = val.into_int_value();
        let result = match op {
            crate::mir::UnaryOp::Neg => {
                self.builder.build_int_neg(int_val, "neg")
                    .map_err(|e| CodegenError { message: format!("neg failed: {:?}", e) })?
            }
            crate::mir::UnaryOp::Not => {
                self.builder.build_not(int_val, "not")
                    .map_err(|e| CodegenError { message: format!("not failed: {:?}", e) })?
            }
        };
        Ok(result.into())
    }

    /// Compile a block terminator.
    fn compile_terminator(
        &mut self,
        terminator: &Terminator,
        blocks: &HashMap<usize, inkwell::basic_block::BasicBlock>,
    ) -> Result<(), CodegenError> {
        match terminator {
            Terminator::Return(operand) => {
                if let Some(op) = operand {
                    let val = self.compile_operand(op)?;
                    self.builder.build_return(Some(&val))
                        .map_err(|e| CodegenError { message: format!("return failed: {:?}", e) })?;
                } else {
                    self.builder.build_return(None)
                        .map_err(|e| CodegenError { message: format!("return failed: {:?}", e) })?;
                }
            }
            Terminator::Goto(target) => {
                if let Some(&bb) = blocks.get(target) {
                    self.builder.build_unconditional_branch(bb)
                        .map_err(|e| CodegenError { message: format!("branch failed: {:?}", e) })?;
                }
            }
            Terminator::SwitchInt {
                discriminant,
                targets,
                otherwise,
            } => {
                let val = self.compile_operand(discriminant)?.into_int_value();
                let else_bb = blocks.get(otherwise).copied().ok_or_else(|| CodegenError {
                    message: "Missing otherwise block".into(),
                })?;

                // For now, handle simple if/else (2 targets)
                if targets.len() == 1 {
                    let (_, then_target) = &targets[0];
                    let then_bb = blocks.get(then_target).copied().ok_or_else(|| CodegenError {
                        message: "Missing then block".into(),
                    })?;
                    let zero = self.context.i64_type().const_int(0, false);
                    let cond = self.builder.build_int_compare(IntPredicate::NE, val, zero, "cond")
                        .map_err(|e| CodegenError { message: format!("cmp failed: {:?}", e) })?;
                    self.builder.build_conditional_branch(cond, then_bb, else_bb)
                        .map_err(|e| CodegenError { message: format!("branch failed: {:?}", e) })?;
                } else {
                    // Use switch instruction for multiple targets
                    let switch = self.builder.build_switch(val, else_bb, targets.len() as u32);
                    for (value, target) in targets {
                        if let Some(&bb) = blocks.get(target) {
                            let const_val = self.context.i64_type().const_int(*value as u64, false);
                            switch.add_case(const_val, bb);
                        }
                    }
                }
            }
            Terminator::Unreachable => {
                self.builder.build_unreachable()
                    .map_err(|e| CodegenError { message: format!("unreachable failed: {:?}", e) })?;
            }
        }
        Ok(())
    }

    /// Lower an ARIA type to an LLVM type.
    fn lower_type(&self, ty: &Ty) -> Result<BasicTypeEnum<'ctx>, CodegenError> {
        match ty {
            Ty::Int | Ty::I64 => Ok(self.context.i64_type().into()),
            Ty::I32 => Ok(self.context.i32_type().into()),
            Ty::I16 => Ok(self.context.i16_type().into()),
            Ty::I8 => Ok(self.context.i8_type().into()),
            Ty::Bool => Ok(self.context.bool_type().into()),
            Ty::Float | Ty::F64 => Ok(self.context.f64_type().into()),
            Ty::F32 => Ok(self.context.f32_type().into()),
            Ty::Unit => Ok(self.context.i8_type().into()), // Unit as i8
            Ty::Str => Ok(self.context.ptr_type(AddressSpace::default()).into()),
            _ => {
                // Default to i64 for complex types
                Ok(self.context.i64_type().into())
            }
        }
    }

    /// Write the module to an object file.
    pub fn write_object_file(&self, path: &Path) -> Result<(), CodegenError> {
        Target::initialize_native(&InitializationConfig::default()).map_err(|e| CodegenError {
            message: format!("Failed to initialize LLVM: {}", e),
        })?;

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|e| CodegenError {
            message: format!("Failed to get target: {:?}", e),
        })?;

        let machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| CodegenError {
                message: "Failed to create target machine".into(),
            })?;

        machine
            .write_to_file(&self.module, FileType::Object, path)
            .map_err(|e| CodegenError {
                message: format!("Failed to write object file: {:?}", e),
            })?;

        Ok(())
    }

    /// Write the module to LLVM IR text file.
    pub fn write_llvm_ir(&self, path: &Path) -> Result<(), CodegenError> {
        self.module.print_to_file(path).map_err(|e| CodegenError {
            message: format!("Failed to write IR: {:?}", e),
        })?;
        Ok(())
    }

    /// Get the LLVM IR as a string.
    pub fn get_llvm_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }
}
