# Memory Model: COARI

Hornet uses **Compile-Time Ownership with Automatic Region Inference (COARI)**. 

## How it Works
COARI tracks the 'Region' of every variable. A Region is a lexical or dynamic span where a piece of memory is guaranteed to be valid. 

1. **Allocation**: Happens at the point of variable initialization.
2. **Deallocation**: The compiler inserts a `drop` instruction at the exact point where the variable's Region expires.
3. **Safety**: Borrowing rules prevent data races and use-after-free at compile time.

## [SECONDARY: D-ARC]
When data must be shared across threads or complex graphs where a single owner cannot be determined, Hornet uses **Deterministic ARC**. 

- **Activation**: Triggered by the `shared` keyword.
- **Cycle Handling**: Hornet uses a trial-deletion strategy for `shared` references to handle cycles without a global GC pause.

## Beginner Impact
Because COARI is automatic, beginners do not need to learn about `malloc`, `free`, or complex lifetime annotations found in other systems languages.
