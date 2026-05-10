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

## Installation (Standalone Compiler)

### 1. Prerequisites (Linux)
Ensure you have the basic build tools:
```bash
sudo apt update && sudo apt install build-essential libglib2.0-dev
```

### 2. Build and Install
```bash
git clone https://github.com/roland-yegon/hornet.git
cd hornet
chmod +x install.sh
./install.sh
```
This will build the compiler and install it to `/usr/local/bin/hornet`, making it available globally.

## The Hive IDE (Recommended)
For the best experience, use the [Hornet Hive IDE](https://github.com/roland-yegon/hornet-hive), which provides a standalone GUI and bundles the compiler automatically.
