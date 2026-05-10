# Beginner-First Design

## 1. Error Clarity
Errors follow a strict format:

**WHAT**: Type Mismatch
**WHY**: You passed an `Int` to a function expecting a `String`.
**FIX**: 
  1. Use `val.str()` to convert the number.
  2. Update function `fn print_name(name: Int)`.
**DOCS**: [https://docs.hornet.org/E001](https://docs.hornet.org/E001)

## 2. Boilerplate Reduction
In Hornet, a script is a valid program. No `main` function or `import core` is required for basic tasks.

## 3. Stdlib Organization

| Module | Intent |
| :--- | :--- |
| `web` | HTTP clients and servers |
| `file` | File system operations |
| `json` | Parsing and serialization |
| `math` | Common mathematical constants and ops |
