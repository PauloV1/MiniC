# MiniC: Short Summary

MiniC is a minimal C-like language implemented in Rust. It provides a parser, type checker, and typed intermediate representation suitable for interpretation or code generation.

## Language Overview

- **Program structure** — Functions only; execution starts at `main`
- **Types** — `int`, `float`, `bool`, `str`, `void`, and arrays (`int[]`, `float[]`, etc.)
- **Variables** — Must be declared with initialization: `int x = expr`
- **Statements** — Declaration, assignment, blocks, `if`/`then`/`else`, `while`/`do`, function calls
- **Expressions** — Literals, identifiers, arithmetic, relational, logical, array literals, indexing, function calls

## Pipeline

```
Source  →  Parser  →  Unchecked AST  →  Type Checker  →  Checked AST  →  (Interpreter / Codegen)
```

The AST is parameterized by phase: unchecked (`Program<()>`) from the parser, checked (`Program<Type>`) from the type checker. Downstream phases accept only checked ASTs; Rust's type system enforces this.
