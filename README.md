# Hornet Programming Language

Hornet 2.0 is a beginner-friendly systems language that reads like structured English and compiles to native binaries.

## Quick Example

```hornet
define greet(name):
    print("Hello " + name)

for i from 1 to 3:
    print(greet("World"))
```

## New English-like Syntax

- `define` replaces `fn`
- `record` replaces `struct`
- `use` replaces `import`
- `check` replaces `match`
- `when` introduces pattern arms
- `otherwise` is the wildcard arm
- `repeat:` is the infinite loop form
- `for i from 1 to 5:` is inclusive
- `for i from 1 upto 5:` is exclusive
- `is`, `isnt`, `above`, `below`, `atleast`, `atmost` are comparison aliases
- `gives` replaces `->` for return type annotations

## CLI Reference

```
hornet version 0.2.0

Usage: hornet <command> [options] <file>

Commands:
  tokenize <file>            Tokenize source file and display tokens
  parse <file>               Parse source file and output AST as JSON
  check <file>               Type-check program without compilation
  build <file>               Compile to native binary
  run <file>                 Execute program immediately using the interpreter
  lsp                        Start language server protocol daemon
  --help, -h                 Show this help message
  --version, -v              Show version information

Build options:
  --release                  Enable optimizations (-O2)
  --emit-ir                  Also write LLVM IR to <file>.ll
```

## Compilation Pipeline

```
.hn source
    │
    ▼
 Lexer  ──────────────► token stream
    │
    ▼
 Parser ──────────────► AST
    │
    ▼
 TypeSystem ───────────► type-checked AST
    │
    ▼
 CoariAnalyzer ────────► ownership-verified AST
    │
    ├─── hornet run  ──► Interpreter ──► stdout
    │
    └─── hornet build ─► Codegen (LLVM IR)
                             │
                             ▼ llc
                        object file (.o)
                             │
                             ▼ cc/clang
                        native binary  ──► stdout
```

## Example Usage

```bash
hornet run examples/hello.hn
hornet build examples/hello.hn
./hello
hornet build --release examples/hello.hn
hornet build --emit-ir examples/hello.hn
```

## Native Build Requirements

- `llc` from the LLVM toolchain
- `clang`, `gcc`, `cc`, or `musl-gcc`

### Install LLVM

- Ubuntu/Debian: `sudo apt install llvm clang`
- macOS: `brew install llvm`
- Windows: `winget install LLVM.LLVM`

## Runtime vs Build

- `hornet run` uses the interpreter for fast development execution
- `hornet build` produces a native binary for distribution

## New Language Example

```hornet
record Point:
    x: Int
    y: Int

define describe(p):
    check p.x:
        when 0:
            print("On the y-axis")
        otherwise:
            print("Point: " + str(p.x) + ", " + str(p.y))
```

## Examples Included

- `examples/hello.hn`
- `examples/fizzbuzz.hn`
- `examples/greet.hn`
- `examples/countdown.hn`
- `examples/arithmetic.hn`
- `examples/web_server.hn`

## Philosophy

> "Fast like C. Simple like Python. Reads like English."
