//! Tree-walking interpreter for MiniC.
//!
//! Entry point: `interpret(program: &CheckedProgram) -> Result<(), RuntimeError>`

pub mod env;
pub mod eval_expr;
pub mod exec_stmt;
pub mod value;

use crate::ir::ast::CheckedProgram;

use env::RuntimeEnv;
use eval_expr::eval_call;
use value::RuntimeError;

/// Interpret a type-checked MiniC program, starting execution at `main`.
pub fn interpret(program: &CheckedProgram) -> Result<(), RuntimeError> {
    let mut env = RuntimeEnv::new();

    // Register all user-defined functions
    for fun in &program.functions {
        env.register_fn(fun.name.clone(), fun.clone());
    }

    // Locate and call main
    if env.get_fn("main").is_none() {
        return Err(RuntimeError::new("no 'main' function found"));
    }

    eval_call("main", vec![], &mut env)?;
    Ok(())
}
