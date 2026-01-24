//! Code generation module for FORMA.
//!
//! This module provides backends for compiling FORMA programs to native code.
//! Currently supports:
//! - LLVM IR generation (with the `llvm` feature)

#[cfg(feature = "llvm")]
pub mod llvm;

#[cfg(feature = "llvm")]
pub use llvm::LLVMCodegen;
