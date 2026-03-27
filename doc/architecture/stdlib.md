# Standard Library Architecture

This document describes the MiniC standard library: how native functions are registered, how they are typed, and what functions are available.

*See also:* [Interpreter Architecture](interpreter.md), [Environment Architecture](environment.md)

---

## 1. Overview

MiniC's standard library is a set of **native functions** implemented in Rust and made available to every MiniC program. Native functions are registered in `NativeRegistry` at startup; both the type checker and the interpreter consume the registry to populate their environments before execution begins.

---

## 2. NativeRegistry

```rust
pub struct NativeRegistry {
    entries: HashMap<String, NativeEntry>,
}

pub struct NativeEntry {
    pub params: Vec<Type>,
    pub return_type: Type,
    pub func: NativeFn,           // fn(Vec<Value>) -> Result<Value, RuntimeError>
}
```

`NativeRegistry::default()` builds the registry with all built-in functions. Neither `type_check` nor `interpret` accept a registry parameter — they construct it internally. This keeps the public API clean.

### Registration in the type checker

Each entry is registered as a `Type::Fun(params, return_type)` binding:

```rust
for (name, entry) in registry.iter() {
    env.declare(name, Type::Fun(entry.params.clone(), Box::new(entry.return_type.clone())));
}
```

### Registration in the interpreter

Each entry is registered as a `Value::Fn(FnValue::Native(f))` binding:

```rust
for (name, entry) in registry.iter() {
    env.declare(name, Value::Fn(FnValue::Native(entry.func)));
}
```

After registration, native functions are indistinguishable from user-defined functions at call sites — they go through the same `env.get(name)` dispatch path.

---

## 3. Type::Any

Some native functions accept arguments of any MiniC type. Rather than overloading, MiniC uses a special `Type::Any` variant:

```rust
pub enum Type {
    // ...
    Any,   // Matches any argument type; only used in native registrations
}
```

`types_compatible(t, Type::Any)` returns `true` for every `t`. This allows `print` to accept `int`, `float`, `bool`, `str`, or an array without a type error.

`Type::Any` is never produced by the parser and never inferred as an expression type; it exists solely for native function parameter declarations.

---

## 4. Available Functions

### 4.1 I/O — `src/stdlib/io.rs`

| Name | Signature | Description |
|------|-----------|-------------|
| `print` | `(Any) -> void` | Formats the argument using its `Display` impl and writes it to stdout followed by a newline |
| `readInt` | `() -> int` | Reads a line from stdin and parses it as `i64` |
| `readFloat` | `() -> float` | Reads a line from stdin and parses it as `f64` |
| `readString` | `() -> str` | Reads a line from stdin and returns it as a trimmed string |

`print` uses `Type::Any` so it accepts any MiniC value.

`readInt` and `readFloat` return a `RuntimeError` if the input cannot be parsed.

### 4.2 Math — `src/stdlib/math.rs`

| Name | Signature | Description |
|------|-----------|-------------|
| `sqrt` | `(float) -> float` | Square root via `f64::sqrt` |
| `pow` | `(float, float) -> float` | Exponentiation via `f64::powf` |

Both functions coerce `Int` arguments to `Float` before applying the operation, so `sqrt(4)` and `pow(2, 10)` work without explicit casts in MiniC source.

---

## 5. Adding a New Native Function

1. Implement the function in `src/stdlib/math.rs` or `src/stdlib/io.rs` (or a new file):

   ```rust
   pub fn abs_fn(args: Vec<Value>) -> Result<Value, RuntimeError> {
       match args.as_slice() {
           [Value::Int(n)]   => Ok(Value::Int(n.abs())),
           [Value::Float(f)] => Ok(Value::Float(f.abs())),
           _ => Err(RuntimeError::new("abs: expected one numeric argument")),
       }
   }
   ```

2. Register it in `NativeRegistry::default()` in `src/stdlib/mod.rs`:

   ```rust
   registry.register("abs", NativeEntry {
       params: vec![Type::Float],
       return_type: Type::Float,
       func: abs_fn,
   });
   ```

   Use `Type::Any` in `params` for functions that should accept multiple types.

3. The function is now available to all MiniC programs with no further changes.
