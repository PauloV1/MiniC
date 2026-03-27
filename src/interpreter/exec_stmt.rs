use crate::ir::ast::{CheckedExpr, CheckedStmt, Expr, Statement};

use super::env::RuntimeEnv;
use super::eval_expr::{eval_call, eval_expr};
use super::value::{RuntimeError, Value};

/// `None` = normal fall-through; `Some(v)` = early return with value.
pub type ExecResult = Result<Option<Value>, RuntimeError>;

/// Execute a checked statement. Returns `Some(v)` if a `return` was hit.
pub fn exec_stmt(stmt: &CheckedStmt, env: &mut RuntimeEnv) -> ExecResult {
    match &stmt.stmt {
        // --- Variable declaration ---
        Statement::Decl { name, init, .. } => {
            let val = eval_expr(init, env)?;
            env.declare_var(name.clone(), val);
            Ok(None)
        }

        // --- Assignment ---
        Statement::Assign { target, value } => {
            let val = eval_expr(value, env)?;
            assign_lvalue(target, val, env)?;
            Ok(None)
        }

        // --- Block ---
        // Only remove variables that were *declared* inside the block on exit.
        // Assignments to outer-scope variables must persist (e.g., loop counters).
        Statement::Block { seq } => {
            let outer_keys = env.var_names();
            for s in seq {
                if let Some(ret) = exec_stmt(s, env)? {
                    env.remove_new_vars(&outer_keys);
                    return Ok(Some(ret));
                }
            }
            env.remove_new_vars(&outer_keys);
            Ok(None)
        }

        // --- If ---
        Statement::If {
            cond,
            then_branch,
            else_branch,
        } => match eval_expr(cond, env)? {
            Value::Bool(true) => exec_stmt(then_branch, env),
            Value::Bool(false) => {
                if let Some(eb) = else_branch {
                    exec_stmt(eb, env)
                } else {
                    Ok(None)
                }
            }
            v => Err(RuntimeError::new(format!(
                "if condition must be bool, got: {}",
                v
            ))),
        },

        // --- While ---
        Statement::While { cond, body } => loop {
            match eval_expr(cond, env)? {
                Value::Bool(true) => {
                    if let Some(ret) = exec_stmt(body, env)? {
                        return Ok(Some(ret));
                    }
                }
                Value::Bool(false) => return Ok(None),
                v => {
                    return Err(RuntimeError::new(format!(
                        "while condition must be bool, got: {}",
                        v
                    )))
                }
            }
        },

        // --- Return ---
        Statement::Return(Some(expr)) => {
            let val = eval_expr(expr, env)?;
            Ok(Some(val))
        }
        Statement::Return(None) => Ok(Some(Value::Void)),

        // --- Statement-level function call ---
        Statement::Call { name, args } => {
            let arg_vals: Result<Vec<Value>, RuntimeError> =
                args.iter().map(|a| eval_expr(a, env)).collect();
            eval_call(name, arg_vals?, env)?;
            Ok(None)
        }
    }
}

/// Assign `val` to the lvalue described by `target`.
fn assign_lvalue(target: &CheckedExpr, val: Value, env: &mut RuntimeEnv) -> Result<(), RuntimeError> {
    match &target.exp {
        Expr::Ident(name) => {
            if env.set_var(name, val) {
                Ok(())
            } else {
                Err(RuntimeError::new(format!(
                    "assignment to undeclared variable '{}'",
                    name
                )))
            }
        }
        Expr::Index { base, index } => {
            // Resolve the index
            let idx = match eval_expr(index, &mut *env)? {
                Value::Int(i) => i as usize,
                v => {
                    return Err(RuntimeError::new(format!(
                        "array index must be int, got: {}",
                        v
                    )))
                }
            };
            assign_index(base, idx, val, env)
        }
        _ => Err(RuntimeError::new("invalid assignment target".to_string())),
    }
}

/// Recursively assign into a (possibly nested) array lvalue.
/// `base` is the expression to the left of `[idx]`, `idx` is already resolved.
fn assign_index(
    base: &CheckedExpr,
    idx: usize,
    val: Value,
    env: &mut RuntimeEnv,
) -> Result<(), RuntimeError> {
    match &base.exp {
        // Base case: simple identifier — mutate the array in the environment
        Expr::Ident(name) => {
            let arr = env
                .get_var(name)
                .cloned()
                .ok_or_else(|| RuntimeError::new(format!("undefined variable '{}'", name)))?;
            match arr {
                Value::Array(mut elems) => {
                    if idx >= elems.len() {
                        return Err(RuntimeError::new(format!(
                            "array index {} out of bounds (len {})",
                            idx,
                            elems.len()
                        )));
                    }
                    elems[idx] = val;
                    env.set_var(name, Value::Array(elems));
                    Ok(())
                }
                v => Err(RuntimeError::new(format!(
                    "cannot index non-array value: {}",
                    v
                ))),
            }
        }
        // Recursive case: nested index — e.g., matrix[i][j]
        Expr::Index {
            base: inner_base,
            index: inner_index,
        } => {
            let inner_idx = match eval_expr(inner_index, env)? {
                Value::Int(i) => i as usize,
                v => {
                    return Err(RuntimeError::new(format!(
                        "array index must be int, got: {}",
                        v
                    )))
                }
            };
            // Fetch the outer array, update the inner element, write back
            let outer_name = extract_ident_name(inner_base)?;
            let outer = env
                .get_var(&outer_name)
                .cloned()
                .ok_or_else(|| RuntimeError::new(format!("undefined variable '{}'", outer_name)))?;
            match outer {
                Value::Array(mut outer_elems) => {
                    if inner_idx >= outer_elems.len() {
                        return Err(RuntimeError::new(format!(
                            "array index {} out of bounds (len {})",
                            inner_idx,
                            outer_elems.len()
                        )));
                    }
                    match &mut outer_elems[inner_idx] {
                        Value::Array(ref mut inner_elems) => {
                            if idx >= inner_elems.len() {
                                return Err(RuntimeError::new(format!(
                                    "array index {} out of bounds (len {})",
                                    idx,
                                    inner_elems.len()
                                )));
                            }
                            inner_elems[idx] = val;
                        }
                        v => {
                            return Err(RuntimeError::new(format!(
                                "cannot index non-array value: {}",
                                v
                            )))
                        }
                    }
                    env.set_var(&outer_name, Value::Array(outer_elems));
                    Ok(())
                }
                v => Err(RuntimeError::new(format!(
                    "cannot index non-array value: {}",
                    v
                ))),
            }
        }
        _ => Err(RuntimeError::new("invalid assignment target".to_string())),
    }
}

/// Extract the identifier name from a simple `Expr::Ident` expression.
fn extract_ident_name(expr: &CheckedExpr) -> Result<String, RuntimeError> {
    match &expr.exp {
        Expr::Ident(name) => Ok(name.clone()),
        _ => Err(RuntimeError::new(
            "nested array assignment only supported for simple variable bases".to_string(),
        )),
    }
}
