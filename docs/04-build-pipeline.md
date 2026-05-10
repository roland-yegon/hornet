# Build Pipeline

## User Commands
- `hornet run file.hn`: Compiles to a temporary binary and executes. The binary is cached using a hash of the source code.
- `hornet build`: Generates an optimized native binary for the current architecture.
- `hornet add <package>`: Fetches and installs a dependency from the Hornet Package Index (HPI).

## Pipeline Stages
1. **Source**: Read `.hn` files.
2. **Parse**: Convert to AST based on `spec/grammar.ebnf`.
3. **Typecheck**: Apply bidirectional inference.
4. **IR**: Lower AST to LLVM-compatible Intermediate Representation.
5. **Optimize**: Run LLVM optimization passes.
6. **Native**: Emit machine code.

## Caching
Hornet uses hash-keyed caching. If the content of `file.hn` produces a hash already in `.hornet/cache`, the cached binary is used immediately.
