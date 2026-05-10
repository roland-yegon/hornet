# Hornet Programming Language

Hornet is a systems programming language designed for speed, simplicity, and beginner UX.

## The Trinity

| Constraint | Requirement | Implementation |
| :--- | :--- | :--- |
| **Speed** | C-level performance | LLVM native compilation, Rust-based compiler |
| **Simplicity** | Python-like syntax | Indentation-based scoping, no semicolons |
| **Beginner UX** | Humane errors | WHY/FIX error messages, zero-boilerplate |

## Quick Look

```hornet
fn greet(name):
    print("Hello, " + name)

for i in 1..3:
    greet("World " + i.str())
```

## Documentation Index
- [Memory Model](docs/01-memory-model.md)
- [Type System](docs/02-type-system.md)
- [Syntax Guide](docs/03-syntax.md)
- [Build Pipeline](docs/04-build-pipeline.md)
- [Philosophy](docs/05-beginner-philosophy.md)
- [Stdlib Reference](docs/06-stdlib.md)

## Project Roadmap
- [x] Language Specification
- [ ] Core Parser
- [ ] Type Inference Engine
- [ ] COARI Region Tracker
- [ ] LLVM IR Emitter
- [ ] HPI Package Manager

> "A language that forces you to understand the machine before you can greet the world has failed its most important user — the beginner."
