use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Unit,
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Function {
        params: Vec<String>,
        body: Vec<crate::ast::Stmt>,
        env: crate::interpreter::Environment,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::Str(s) => write!(f, "{}", s),
            Value::Unit => write!(f, "()"),
            Value::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Map(m) => {
                let mut items: Vec<String> = m
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                items.sort();
                write!(f, "{{{}}}", items.join(", "))
            }
            Value::Function { params, .. } => write!(f, "<fn({})>", params.join(", ")),
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Unit => false,
            Value::Int(0) => false,
            Value::Float(n) if *n == 0.0 => false,
            Value::Str(s) if s.is_empty() => false,
            Value::Array(a) if a.is_empty() => false,
            Value::Map(m) if m.is_empty() => false,
            _ => true,
        }
    }

    pub fn to_string_value(&self) -> String {
        self.to_string()
    }

    pub fn to_int(&self) -> Result<i64, String> {
        match self {
            Value::Int(n) => Ok(*n),
            Value::Float(n) => Ok(*n as i64),
            Value::Str(s) => s.parse().map_err(|_| format!("Cannot convert '{}' to int", s)),
            Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err(format!("Cannot convert {} to int", self)),
        }
    }

    pub fn to_float(&self) -> Result<f64, String> {
        match self {
            Value::Int(n) => Ok(*n as f64),
            Value::Float(n) => Ok(*n),
            Value::Str(s) => s.parse().map_err(|_| format!("Cannot convert '{}' to float", s)),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err(format!("Cannot convert {} to float", self)),
        }
    }

    pub fn to_bool(&self) -> bool {
        self.is_truthy()
    }
}
