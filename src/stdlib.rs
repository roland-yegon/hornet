use crate::interpreter::{Environment, Value};

/// Register the Hornet built-in functions in the interpreter global scope.
/// These functions are implemented by the runtime, not by user-defined Hornet code.
pub fn register_stdlib(env: &mut Environment) {
    let builtins = [
        "print",
        "println",
        "str",
        "int",
        "float",
        "bool",
        "len",
        "type_of",
        "range",
        "input",
        "assert",
    ];

    for name in builtins {
        let func = Value::Function {
            params: Vec::new(),
            body: Vec::new(),
            env: Environment::new(),
        };
        env.define(name, func);
    }
}
