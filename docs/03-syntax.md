# Syntax Specification

## Rules
- **Indentation**: 4 spaces strictly. Tabs are forbidden.
- **Termination**: Newlines terminate statements. No semicolons.
- **Functions**: Declared with `fn`.
- **Ranges**: `1..5` is 1,2,3,4,5. `1..<5` is 1,2,3,4.

## Comparisons

| Language | Hello World | Loop 1 to 5 |
| :--- | :--- | :--- |
| **C** | `printf("Hello\n");` | `for(int i=1; i<=5; i++)` |
| **Python** | `print("Hello")` | `for i in range(1, 6):` |
| **Hornet** | `print("Hello")` | `for i in 1..5:` |

## Control Flow
```hornet
if x > 10:
    print("Big")
else if x > 5:
    print("Mid")
else:
    print("Small")

match val:
    1 => print("One")
    _ => print("Other")
```
