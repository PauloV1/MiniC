# Environment Architecture

This document describes the unified `Environment<V>` struct: MiniC's single symbol table that stores both variable and function bindings in one map.

*See also:* [AST Architecture](ast.md), [Type Checker Design](../design/type-checker.md), [Interpreter Architecture](interpreter.md)

---

## 1. Purpose

Every phase that needs to track name-to-value mappings — the type checker and the interpreter — requires a **symbol table**: a structure that maps names to values and supports scoping.

Earlier designs used separate maps for variables and functions. The unified design collapses these into one parametric type: `Environment<V>`, where `V` is the kind of value being stored.

---

## 2. The Struct

```rust
pub struct Environment<V> {
    bindings: HashMap<String, V>,
}
```

A single `HashMap<String, V>` holds all bindings. Both variable bindings and function bindings live in the same map.

### Two instantiations

| Instantiation | V | Used by |
|---|---|---|
| `Environment<Type>` | `Type` (from `ir::ast`) | Type checker — maps names to their declared type |
| `Environment<Value>` | `Value` (from `interpreter::value`) | Interpreter — maps names to their runtime value |

Function bindings use the same map:

- In the type checker, a function `foo(int, float) -> bool` is stored as `Type::Fun(vec![Int, Float], Box::new(Bool))`.
- In the interpreter, the same function is stored as `Value::Fn(FnValue::UserDefined(decl))` or `Value::Fn(FnValue::Native(f))`.

---

## 3. API

```rust
impl<V: Clone> Environment<V> {
    pub fn new() -> Self
    pub fn declare(&mut self, name: impl Into<String>, value: V)
    pub fn get(&self, name: &str) -> Option<&V>
    pub fn set(&mut self, name: &str, value: V) -> bool
    pub fn snapshot(&self) -> HashMap<String, V>
    pub fn restore(&mut self, snapshot: HashMap<String, V>)
    pub fn names(&self) -> HashSet<String>
    pub fn remove_new(&mut self, outer: &HashSet<String>)
}
```

| Method | Purpose |
|--------|---------|
| `declare(name, v)` | Insert a new binding; overwrites if already present |
| `get(name)` | Look up a binding; returns `None` if not found |
| `set(name, v)` | Update an existing binding; returns `false` if not found |
| `snapshot()` | Clone all current bindings into a `HashMap` |
| `restore(snap)` | Replace all bindings with a previously taken snapshot |
| `names()` | Return the current set of bound names as a `HashSet<String>` |
| `remove_new(outer)` | Remove every binding whose name is not in `outer` |

---

## 4. Scoping Patterns

There are two distinct scoping situations with different requirements:

### 4.1 Function Call Scoping — snapshot / restore

When a function is called, the interpreter needs a clean scope for the callee: parameters are bound, the body runs, then everything added during the call is discarded. The key requirement is that **all prior state is restored exactly**, including the function bindings themselves (which enables recursion).

```rust
let snapshot = env.snapshot();
for ((param_name, _), val) in decl.params.iter().zip(args) {
    env.declare(param_name.clone(), val);
}
let result = exec_stmt(&decl.body, env)?;
env.restore(snapshot);
```

After `restore`, the environment is identical to what it was before the call.

### 4.2 Block Scoping — names / remove_new

Inside a block `{ stmt; stmt; ... }`, variables declared in the block must not be visible after the closing brace. But **assignments to outer variables must persist** — if a block does `x = 5` and `x` was declared before the block, that assignment must survive block exit.

`snapshot / restore` would undo those outer assignments. Instead, `remove_new` removes only the *new* names:

```rust
let outer = env.names();          // capture names before block
exec_stmts(block, env)?;          // block may declare new names, assign old ones
env.remove_new(&outer);           // remove names not in outer
```

Because function bindings are registered before any blocks execute, they are always part of `outer` and are never removed.

---

## 5. Registration Sequence

Both the type checker and interpreter follow the same registration sequence at startup:

1. Create a fresh `Environment`.
2. Register all native stdlib functions.
3. Register all user-defined functions from the program.
4. (Type checker only) Take a `fn_snapshot` after step 3. Restore to it before checking each function body so that variable bindings from one function do not leak into the next.
5. Execute / type-check `main`.

This ensures that every function — native or user-defined — is accessible from any call site, regardless of source order.
