# Hornet Architectural Foundation

This document preserves the core constraints used to design Hornet.

## Core Principles
- Speed: LLVM-backed native execution.
- Simplicity: Indentation-based, minimal punctuation.
- Beginner UX: High-quality diagnostics.

## Memory Management
Primary: COARI (Compile-Time Ownership with Automatic Region Inference).
Secondary: Deterministic ARC for shared ownership.
