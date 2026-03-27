# MiniC

**[→ Short summary of MiniC](doc/summary.md)** — language overview, types, and pipeline.

---

## Quick Start

```bash
cargo build
cargo test
```

---

## Architecture

MiniC is organized into these main components:

| Component | Path | Description |
|-----------|------|-------------|
| [**AST**](doc/architecture/ast.md) | `src/ir/` | Abstract syntax tree parameterized by phase (unchecked vs checked). Defines `Expr`, `Statement`, `Program`, and type synonyms (`UncheckedProgram`, `CheckedProgram`, etc.). |
| [**Parser**](doc/architecture/parser.md) | `src/parser/` | Parser combinators using [nom](https://github.com/rust-bakery/nom). Parses literals, expressions, statements, and function declarations into an unchecked AST. |
| [**Type Checker**](doc/design/type-checker.md) | `src/semantic/` | Consumes unchecked AST, validates types, produces checked AST. Requires `main`; enforces variable declarations and type compatibility. |
| [**Environment**](doc/architecture/environment.md) | `src/environment/` | Unified parametric symbol table `Environment<V>`. Stores variable and function bindings in one map. Used by both the type checker (`V = Type`) and the interpreter (`V = Value`). |
| [**Interpreter**](doc/architecture/interpreter.md) | `src/interpreter/` | Tree-walking interpreter. Evaluates expressions and executes statements against `Environment<Value>`. Dispatches user-defined and native functions through the same lookup path. |
| [**Stdlib**](doc/architecture/stdlib.md) | `src/stdlib/` | Native functions (print, readInt, readFloat, readString, sqrt, pow). Registered via `NativeRegistry`; consumed internally by the type checker and interpreter. |

```
src/
├── ir/           # AST (ast.rs)
├── parser/       # Parser (expressions, statements, functions, literals, identifiers)
├── semantic/     # Type checker
├── environment/  # Unified Environment<V>
├── interpreter/  # Tree-walking interpreter (eval_expr, exec_stmt, value)
└── stdlib/       # Native function registry (io, math)
```

---

## Testing

MiniC uses **integration tests** in the `tests/` directory. All tests use only the public API; there are no `#[cfg(test)]` blocks in source modules.

| Test file | Purpose |
|-----------|---------|
| [**parser.rs**](tests/parser.rs) | Parser unit-style tests: literals, identifiers, expressions, statements. Uses inline strings. |
| [**program.rs**](tests/program.rs) | Full-program parsing from fixture files in `tests/fixtures/`. |
| [**type_checker.rs**](tests/type_checker.rs) | Semantic tests: parse + type-check, assert on success/failure or typed AST. |
| [**interpreter.rs**](tests/interpreter.rs) | End-to-end tests: parse + type-check + interpret. Covers arithmetic, control flow, recursion, arrays, and stdlib functions. |

**Run all tests:** `cargo test`

For details on test organization, patterns, and how to add new tests, see [**Test Architecture**](doc/architecture/tests.md).

---

## Specifications

Formal specs live under [openspec/specs/](openspec/specs/). Key specs:

- [AST](openspec/specs/ast/spec.md)
- [Functions](openspec/specs/functions/spec.md)
- [Arrays](openspec/specs/arrays/spec.md)
- [Type checker](openspec/specs/type-checker/spec.md)
- [Unified environment](openspec/specs/unified-environment/spec.md)
- [Interpreter core](openspec/specs/interpreter-core/spec.md)
- [Function dispatch](openspec/specs/function-dispatch/spec.md)
- [Parser documentation](openspec/specs/parser-docs/spec.md)
