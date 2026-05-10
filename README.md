# Hornet Programming Language

Hornet is a native systems programming language built for speed, simplicity, and beginner-friendly developer experience.

## What Hornet Is

Hornet is a compiled, LLVM-backed language with Python-inspired syntax and C-like performance goals. It is designed for real-world systems development, backend services, embedded tooling, and general-purpose applications without sacrificing readability.

## Why Hornet Exists

Hornet solves the Trinity problem:
- **Speed**: native, LLVM-compiled binaries with minimal runtime
- **Simplicity**: indentation-based syntax, low boilerplate, readable semantics
- **Beginner friendliness**: clear error messages, predictable behavior, easy onboarding

## Feature List

- Native compilation via LLVM
- Indentation-based syntax with Python-style readability
- Minimal runtime, no garbage collector, no hidden overhead
- Strong compile-time checks and safe default semantics
- Designed for cross-platform use on Linux, macOS, Windows, x86_64, and ARM
- Core tooling for tokenize, parse, check, build, run, and LSP support

## Installation

Hornet is cross-platform. For specific OS instructions, see **[docs/00-installation.md](docs/00-installation.md)**.

### Quick Start
```bash
git clone https://github.com/roland-yegon/hornet.git
cd hornet
./install.sh
```

### Build from Source
```bash
cargo build --release
```

### Windows
```powershell
cargo build --release
```
Then add `target\release\hornet.exe` to your PATH.

## First Program

Create `hello.hn`:
```hornet
fn main():
    print("Hello, Hornet!")
```

Compile and run:
```bash
hornet build hello.hn
./hello.hn.ll
```

## CLI Quick Start

Basic commands:
- `hornet tokenize <file>` — lex source
- `hornet parse <file>` — parse into AST
- `hornet check <file>` — semantic/type check
- `hornet build <file>` — generate LLVM IR
- `hornet run <file>` — execute a source program stub
- `hornet lsp` — start language server mode

## Documentation Index

- `docs/00-installation.md` — installation guide
- `docs/01-memory-model.md` — Hornet memory and ownership
- `docs/02-type-system.md` — type system design
- `docs/03-syntax.md` — syntax and grammar
- `docs/04-build-pipeline.md` — compiler pipeline
- `docs/05-beginner-philosophy.md` — language philosophy
- `docs/06-stdlib.md` — standard library overview
- `docs/PROMPT.md` — language design prompt

## Standard Library Overview

Hornet standard library is designed to be small, safe, and efficient. It includes:
- Core: `io`, `fs`, `path`, `time`, `math`, `random`, `collections`, `text`
- Data: `json`, `csv`, `toml`, `yaml`
- Networking: `http`, `tcp`, `udp`, `websocket`, `tls`
- Concurrency: `async`, `thread`, `channel`, `sync`
- System: `process`, `os`, `env`, `signal`
- Security: `crypto`, `hash`, `secrets`

## Roadmap

1. Compiler correctness and validation suite
2. Expanded standard library with secure and performance-oriented APIs
3. Cross-platform ABI and platform compatibility layer
4. LSP, formatter, and package manager tooling
5. Production hardening and security model
6. Ecosystem support for deployment, testing, and debugging

## Safety Guarantees

Hornet targets:
- memory-safe-by-default semantics
- no garbage collection
- no hidden runtime allocations
- safe-by-default standard library APIs
- hardened parser/compiler error handling

## Performance Philosophy

Hornet preserves performance through:
- zero-cost abstractions
- LLVM-backed native compilation
- minimal runtime footprint
- explicit memory and ownership semantics
- incremental compiler design

## Contributing Guide

Contributions are welcome. Please follow the repository issue and pull request process, keep changes small and reviewable, and maintain the Hornet philosophy of speed, simplicity, and safety.

## Compiler Architecture Overview

Hornet is built as a Rust compiler frontend with modular phases:
- Lexer
- Parser
- AST
- Type system
- Semantic analysis
- IR generation
- Codegen
- Optional LSP and diagnostics

## Platform Support Matrix

| Platform | Status |
| --- | --- |
| Linux x86_64 | Supported |
| Linux ARM64 | Supported |
| macOS x86_64 | Supported |
| macOS ARM64 | Supported |
| Windows x86_64 | Supported |

## Philosophy Quote

> "Fast like C. Simple like Python. Beginner-friendly for real systems work."
