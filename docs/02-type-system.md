# Type System

Hornet uses a bidirectional type inference system. 

## Inference Examples

```hornet
x = 5
```
- **Beginner view**: A simple variable holding a number.
- **Compiler view**: `x` is inferred as `core.Int64` based on the literal value.

## Annotation Requirements
Annotations are required when:
1. Parameters in a public module function: `fn add(a: Int, b: Int)`.
2. Global constants: `const PI: Float = 3.14`.
3. Disambiguating generic types.

## Polymorphism
Hornet uses 'Traits' for polymorphism. A type implements a trait by providing the required functions. No explicit `implements` keyword is needed (structural typing with nominal safety).
