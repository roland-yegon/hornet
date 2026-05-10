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

## Installation

Hornet is cross-platform. For detailed instructions on your specific Operating System or Linux distribution, please see the **[Full Installation Guide](docs/00-installation.md)**.

### Quick Start (Linux & macOS)
```bash
git clone https://github.com/roland-yegon/hornet.git
cd hornet
./install.sh
```

### Windows
1. Build with `cargo build --release`.
2. Add `target/release/hornet.exe` to your PATH.

## The Hive IDE (Recommended)
For the best experience, use the [Hornet Hive IDE](https://github.com/roland-yegon/hornet-hive), which provides a standalone GUI and bundles the compiler automatically.
