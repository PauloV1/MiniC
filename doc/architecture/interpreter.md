# Interpreter Architecture

This document describes the MiniC tree-walking interpreter: how it represents runtime values, evaluates expressions, executes statements, and dispatches function calls.

*See also:* [AST Architecture](ast.md), [Environment Architecture](environment.md), [Stdlib Architecture](stdlib.md)

---

## 1. Overview

MiniC uses a **tree-walking interpreter**: it walks the `CheckedProgram` AST directly, evaluating each node as it goes. There is no bytecode compilation or intermediate representation.

Entry point:

```rust
pub fn interpret(program: &CheckedProgram) -> Result<(), RuntimeError>
```

The function:

1. Constructs `NativeRegistry::default()` and an `Environment<Value>`.
2. Registers all native functions as `Value::Fn(FnValue::Native(...))` bindings.
3. Registers all user-defined functions as `Value::Fn(FnValue::UserDefined(...))` bindings.
4. Calls `eval_call("main", vec![], &mut env)` to start execution.

---

## 2. Value Representation

All runtime values are represented by the `Value` enum:

```rust
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Vec<Value>),
    Void,
    Fn(FnValue),
}
```

`Value::Fn` is used only for environment storage and dispatch; it is never produced by evaluating a MiniC expression directly. User-visible values are the scalar and composite variants (`Int`, `Float`, `Bool`, `Str`, `Array`, `Void`).

### FnValue

```rust
pub enum FnValue {
    UserDefined(CheckedFunDecl),
    Native(NativeFn),              // NativeFn = fn(Vec<Value>) -> Result<Value, RuntimeError>
}
```

Both user-defined and native functions are stored as `Value::Fn` in `Environment<Value>`, so dispatch goes through the same `env.get(name)` lookup for both.

### RuntimeError

```rust
pub struct RuntimeError { pub message: String }
```

Runtime errors are returned as `Err(RuntimeError)` and never panic. They carry a human-readable message identifying the cause (undefined variable, out-of-bounds index, arity mismatch, etc.).

---

## 3. Expression Evaluation

`eval_expr(expr: &CheckedExpr, env: &mut Environment<Value>) -> Result<Value, RuntimeError>`

Expression evaluation is a straightforward recursive walk. Key behaviours:

### 3.1 Arithmetic and type coercion

Binary arithmetic (`+`, `-`, `*`, `/`) coerces operands to float whenever one side is `Float`:

| Left | Right | Result |
|------|-------|--------|
| `Int` | `Int` | `Int` |
| `Int` | `Float` | `Float` |
| `Float` | `Int` | `Float` |
| `Float` | `Float` | `Float` |

String concatenation is handled by `+` when both operands are `Str`.

### 3.2 Short-circuit boolean evaluation

`and` and `or` use short-circuit evaluation. For `a and b`, the right side is only evaluated if `a` is `true`. For `a or b`, the right side is only evaluated if `a` is `false`.

### 3.3 Array operations

- **Array literal** `[e1, e2, ...]`: evaluates each element, returns `Value::Array(...)`.
- **Index read** `base[i]`: evaluates `base` to `Array(elems)` and `i` to `Int(n)`, then returns `elems[n]`. Returns `RuntimeError` if `n` is out of bounds.

### 3.4 Function call dispatch

`eval_call(name: &str, args: Vec<Value>, env: &mut Environment<Value>) -> Result<Value, RuntimeError>`

1. Look up `name` in the environment via `env.get(name)`.
2. Dispatch on the result:
   - `Value::Fn(FnValue::Native(f))` — call `f(args)` directly.
   - `Value::Fn(FnValue::UserDefined(decl))` — see section 5.
   - `Some(_)` (non-function) — return `RuntimeError` (`not a function`).
   - `None` — return `RuntimeError` (`undefined function`).

---

## 4. Statement Execution

`exec_stmt(stmt: &CheckedStmt, env: &mut Environment<Value>) -> ExecResult`

where `ExecResult = Result<Option<Value>, RuntimeError>`. The `Option<Value>` models early return: `None` means normal completion; `Some(v)` means a `return v` was encountered. This signal propagates up through nested statements until `eval_call` catches it.

### Statement types

| Statement | Behaviour |
|-----------|-----------|
| `Decl { name, init }` | Evaluates `init`, calls `env.declare(name, value)` |
| `Assign { target, value }` | Evaluates `value`, calls `env.set(name, value)` (or nested array assignment) |
| `Block { seq }` | Executes statements in order; uses `names`/`remove_new` for block scoping |
| `If { cond, then, else_ }` | Evaluates `cond`, executes the appropriate branch; propagates early return |
| `While { cond, body }` | Loops while `cond` is `Bool(true)`; propagates early return |
| `Return(Some(e))` | Evaluates `e`, returns `Ok(Some(value))` — the early-return signal |
| `Return(None)` | Returns `Ok(Some(Value::Void))` |
| `Call { name, args }` | Evaluates args, calls `eval_call`, discards return value |

### Block scoping

Blocks use the `names` / `remove_new` pattern (see [Environment Architecture](environment.md)):

```rust
let outer = env.names();
for stmt in seq { exec_stmt(stmt, env)?; }
env.remove_new(&outer);
```

This removes variables declared inside the block while preserving outer-variable assignments and function bindings.

### Nested array assignment

`arr[i][j] = v` is handled recursively: the interpreter reads the array at `arr[i]`, updates element `[j]`, and writes the updated array back. This extends to arbitrary nesting depth.

---

## 5. Function Call Scoping

When a user-defined function is called:

1. Arity is checked; return `RuntimeError` if mismatched.
2. `env.snapshot()` captures the current environment.
3. Each parameter name is bound to its argument value via `env.declare`.
4. The function body is executed with `exec_stmt`.
5. `env.restore(snapshot)` reverts the environment to its pre-call state.
6. The return value is the `Some(v)` carried by an early-return signal, or `Value::Void` if the body completes normally.

Snapshot/restore ensures that:
- Parameter bindings do not escape the function.
- Recursive calls each get their own independent bindings.
- All function bindings remain available inside the call (they were already in the snapshot).
