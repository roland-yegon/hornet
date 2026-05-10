#![allow(dead_code)]

pub const CORE: &str = r#"
# Core primitives
fn print(val): pass
fn str(val): pass
"#;

pub const MATH: &str = r#"
# Math and Data Science
struct Matrix:
    rows: Int
    cols: Int
    data: List

fn dot(a: Matrix, b: Matrix):
    # Linear algebra dot product
    pass
"#;

pub const WEB: &str = r#"
# Web and Networking
struct Request:
    method: String
    path: String
    headers: Map

fn serve(port: Int, handler: Function):
    # Start HTTP server
    pass
"#;

pub const SYS: &str = r#"
# Systems Programming
fn ffi_call(lib: String, func: String, args: List):
    # Foreign Function Interface
    pass

fn alloc(size: Int):
    # Direct memory allocation
    pass
"#;
