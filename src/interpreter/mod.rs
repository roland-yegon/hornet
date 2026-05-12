pub mod value;
pub mod env;
pub use value::Value;
pub use env::Environment;

use crate::ast::{Program, Stmt, Expr, Literal};
use crate::error::HornetError;
use crate::stdlib;

pub struct Interpreter {
    env: Environment,
}

#[derive(Debug, Clone)]
enum ControlFlow {
    None,
    Break,
    Continue,
    Return(Value),
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Interpreter {
            env: Environment::new(),
        };
        interpreter.init_stdlib();
        interpreter
    }

    fn init_stdlib(&mut self) {
        stdlib::register_stdlib(&mut self.env);
    }

    pub fn run(&mut self, program: &Program) -> Result<(), HornetError> {
        for stmt in &program.statements {
            match self.exec_stmt(stmt)? {
                ControlFlow::Return(_) => break, // Top-level return
                _ => {}
            }
        }
        
        // Look for main() function and call it if it exists
        if let Some(Value::Function { params, body, env }) = self.env.get("main").cloned() {
            if !params.is_empty() {
                return Err("main() should take no parameters".into());
            }
            self.env = env;
            self.env.push_scope();
            for stmt in &body {
                match self.exec_stmt(stmt)? {
                    ControlFlow::Return(_) => break,
                    _ => {}
                }
            }
            self.env.pop_scope();
        }

        Ok(())
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<ControlFlow, HornetError> {
        match stmt {
            Stmt::Let { name, value } => {
                let val = self.eval_expr(value)?;
                self.env.define(name, val);
                Ok(ControlFlow::None)
            }
            Stmt::Assignment { lhs, value } => {
                if let Expr::Identifier(name) = lhs {
                    let val = self.eval_expr(value)?;
                    // If variable doesn't exist, create it (implicit let)
                    if !self.env.set(name, val.clone()) {
                        self.env.define(name, val);
                    }
                    Ok(ControlFlow::None)
                } else {
                    Err("Can only assign to identifiers".into())
                }
            }
            Stmt::If { condition, then_branch, else_ifs, else_branch } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.is_truthy() {
                    self.env.push_scope();
                    for s in then_branch {
                        match self.exec_stmt(s)? {
                            ControlFlow::None => {}
                            cf => {
                                self.env.pop_scope();
                                return Ok(cf);
                            }
                        }
                    }
                    self.env.pop_scope();
                } else {
                    let mut executed = false;
                    for (elif_cond, elif_body) in else_ifs {
                        let elif_val = self.eval_expr(elif_cond)?;
                        if elif_val.is_truthy() {
                            self.env.push_scope();
                            for s in elif_body {
                                match self.exec_stmt(s)? {
                                    ControlFlow::None => {}
                                    cf => {
                                        self.env.pop_scope();
                                        return Ok(cf);
                                    }
                                }
                            }
                            self.env.pop_scope();
                            executed = true;
                            break;
                        }
                    }
                    if !executed {
                        if let Some(else_body) = else_branch {
                            self.env.push_scope();
                            for s in else_body {
                                match self.exec_stmt(s)? {
                                    ControlFlow::None => {}
                                    cf => {
                                        self.env.pop_scope();
                                        return Ok(cf);
                                    }
                                }
                            }
                            self.env.pop_scope();
                        }
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::For { iterator, iterable, body } => {
                let iter_val = self.eval_expr(iterable)?;
                match iter_val {
                    Value::Array(arr) => {
                        self.env.push_scope();
                        for elem in arr {
                            self.env.define(iterator, elem);
                            for s in body {
                                match self.exec_stmt(s)? {
                                    ControlFlow::Break => {
                                        self.env.pop_scope();
                                        return Ok(ControlFlow::None);
                                    }
                                    ControlFlow::Continue => break,
                                    ControlFlow::Return(v) => {
                                        self.env.pop_scope();
                                        return Ok(ControlFlow::Return(v));
                                    }
                                    ControlFlow::None => {}
                                }
                            }
                        }
                        self.env.pop_scope();
                        Ok(ControlFlow::None)
                    }
                    _ => Err("For loop iterable must be an array or range".into()),
                }
            }
            Stmt::While { condition, body } => {
                self.env.push_scope();
                loop {
                    let cond_val = self.eval_expr(condition)?;
                    if !cond_val.is_truthy() {
                        break;
                    }
                    for s in body {
                        match self.exec_stmt(s)? {
                            ControlFlow::Break => {
                                self.env.pop_scope();
                                return Ok(ControlFlow::None);
                            }
                            ControlFlow::Continue => break,
                            ControlFlow::Return(v) => {
                                self.env.pop_scope();
                                return Ok(ControlFlow::Return(v));
                            }
                            ControlFlow::None => {}
                        }
                    }
                }
                self.env.pop_scope();
                Ok(ControlFlow::None)
            }
            Stmt::Loop { body } => {
                self.env.push_scope();
                loop {
                    for s in body {
                        match self.exec_stmt(s)? {
                            ControlFlow::Break => {
                                self.env.pop_scope();
                                return Ok(ControlFlow::None);
                            }
                            ControlFlow::Continue => break,
                            ControlFlow::Return(v) => {
                                self.env.pop_scope();
                                return Ok(ControlFlow::Return(v));
                            }
                            ControlFlow::None => {}
                        }
                    }
                }
            }
            Stmt::Match { value, arms } => {
                let match_value = self.eval_expr(value)?;
                for (pattern, body) in arms {
                    self.env.push_scope();
                    if self.pattern_matches(&match_value, pattern)? {
                        for s in body {
                            match self.exec_stmt(s)? {
                                ControlFlow::Return(v) => {
                                    self.env.pop_scope();
                                    return Ok(ControlFlow::Return(v));
                                }
                                cf => {
                                    self.env.pop_scope();
                                    return Ok(cf);
                                }
                            }
                        }
                        self.env.pop_scope();
                        return Ok(ControlFlow::None);
                    }
                    self.env.pop_scope(); // Pop the scope if pattern didn't match
                }
                // No pattern matched - this is an error in Hornet (exhaustive matching required)
                return Err("Match expression did not match any pattern".into());
            }
            Stmt::Break => Ok(ControlFlow::Break),
            Stmt::Continue => Ok(ControlFlow::Continue),
            Stmt::Return(expr) => {
                let val = self.eval_expr(expr)?;
                Ok(ControlFlow::Return(val))
            }
            Stmt::FunctionDef { name, params, return_type: _, body } => {
                let param_names = params.iter().map(|(name, _)| name.clone()).collect();
                let func = Value::Function {
                    params: param_names,
                    body: body.clone(),
                    env: self.env.clone(),
                };
                self.env.define(name, func);
                Ok(ControlFlow::None)
            }
            Stmt::StructDef { .. } => {
                // [[PHASE BLOCKED: struct instantiation not yet implemented]]
                Ok(ControlFlow::None)
            }
            Stmt::Import(_) => {
                // [[PHASE BLOCKED: module system not yet implemented]]
                Ok(ControlFlow::None)
            }
            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(ControlFlow::None)
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, HornetError> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(n) => Ok(Value::Int(*n)),
                Literal::Float(n) => Ok(Value::Float(*n)),
                Literal::String(s) => Ok(Value::Str(s.clone())),
                Literal::Bool(b) => Ok(Value::Bool(*b)),
                Literal::Unit => Ok(Value::Unit),
            },
            Expr::Identifier(name) => {
                self.env
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable: {}", name).into())
            }
            Expr::BinaryOp { left, op, right } => {
                self.eval_binary_op(left, op, right)
            }
            Expr::UnaryOp { op, operand } => {
                self.eval_unary_op(op, operand)
            }
            Expr::Call { target, args } => {
                self.eval_call(target, args)
            }
            Expr::Range { start, end, inclusive } => {
                let start_val = self.eval_expr(start)?.to_int()?;
                let end_val = self.eval_expr(end)?.to_int()?;
                let end = if *inclusive { end_val + 1 } else { end_val };
                let range: Vec<Value> = (start_val..end)
                    .map(Value::Int)
                    .collect();
                Ok(Value::Array(range))
            }
            Expr::List(elements) => {
                let vals: Result<Vec<_>, _> = elements.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::Array(vals?))
            }
            Expr::Map(pairs) => {
                let mut map = Vec::new();
                for (k, v) in pairs {
                    let key = self.eval_expr(k)?;
                    let val = self.eval_expr(v)?;
                    map.push((key, val));
                }
                Ok(Value::Map(map))
            }
            Expr::IndexAccess { object, index } => {
                let obj = self.eval_expr(object)?;
                let idx = self.eval_expr(index)?;
                match obj {
                    Value::Array(arr) => {
                        let i = idx.to_int()? as usize;
                        arr.get(i)
                            .cloned()
                            .ok_or_else(|| "Index out of bounds".into())
                    }
                    Value::Map(map) => {
                        let key = idx;
                        for (entry_key, entry_val) in map {
                            if entry_key == key {
                                return Ok(entry_val.clone());
                            }
                        }
                        Err("Key not found".into())
                    }
                    Value::Str(s) => {
                        let i = idx.to_int()? as usize;
                        s.chars()
                            .nth(i)
                            .map(|c| Value::Str(c.to_string()))
                            .ok_or_else(|| "Index out of bounds".into())
                    }
                    _ => Err("Cannot index this value".into()),
                }
            }
            Expr::MemberAccess { .. } => {
                Err("Member access not yet implemented".into())
            }
            Expr::NamedArg { .. } => {
                Err("Named arguments not yet implemented".into())
            }
        }
    }

    fn eval_binary_op(&mut self, left: &Expr, op: &str, right: &Expr) -> Result<Value, HornetError> {
        // Short-circuit evaluation for logical operators
        if op == "and" {
            let left_val = self.eval_expr(left)?;
            if !left_val.is_truthy() {
                return Ok(Value::Bool(false));
            }
            let right_val = self.eval_expr(right)?;
            return Ok(Value::Bool(right_val.is_truthy()));
        }
        if op == "or" {
            let left_val = self.eval_expr(left)?;
            if left_val.is_truthy() {
                return Ok(Value::Bool(true));
            }
            let right_val = self.eval_expr(right)?;
            return Ok(Value::Bool(right_val.is_truthy()));
        }

        let left_val = self.eval_expr(left)?;
        let right_val = self.eval_expr(right)?;

        match (left_val, right_val) {
            (Value::Int(l), Value::Int(r)) => self.eval_int_op(l, op, r),
            (Value::Float(l), Value::Float(r)) => self.eval_float_op(l, op, r),
            (Value::Int(l), Value::Float(r)) => self.eval_float_op(l as f64, op, r),
            (Value::Float(l), Value::Int(r)) => self.eval_float_op(l, op, r as f64),
            (Value::Str(l), Value::Str(r)) => self.eval_str_op(l, op, r),
            (Value::Bool(l), Value::Bool(r)) => self.eval_bool_op(l, op, r),
            (l, r) => Err(format!(
                "Type mismatch: cannot apply {} to {} and {}",
                op, type_name(&l), type_name(&r)
            ).into()),
        }
    }

    fn eval_unary_op(&mut self, op: &str, operand: &Expr) -> Result<Value, HornetError> {
        let val = self.eval_expr(operand)?;
        match op {
            "not" => Ok(Value::Bool(!val.is_truthy())),
            "-" => match val {
                Value::Int(n) => Ok(Value::Int(-n)),
                Value::Float(n) => Ok(Value::Float(-n)),
                _ => Err("Cannot negate non-numeric value".into()),
            },
            _ => Err(format!("Unknown unary operator: {}", op).into()),
        }
    }

    fn eval_call(&mut self, target: &Expr, args: &[Expr]) -> Result<Value, HornetError> {
        // Handle method calls: obj.method(args...)
        if let Expr::MemberAccess { object, member } = target {
            let obj = self.eval_expr(object)?;
            return self.eval_method_call(obj, member, args);
        }

        if let Expr::Identifier(name) = target {
            // Handle built-in functions
            match name.as_str() {
                "print" => {
                    for arg in args {
                        let val = self.eval_expr(arg)?;
                        print!("{}", val);
                    }
                    return Ok(Value::Unit);
                }
                "println" => {
                    for (i, arg) in args.iter().enumerate() {
                        let val = self.eval_expr(arg)?;
                        print!("{}", val);
                        if i < args.len() - 1 {
                            print!(" ");
                        }
                    }
                    println!();
                    return Ok(Value::Unit);
                }
                "str" => {
                    if args.len() != 1 {
                        return Err("str() takes exactly 1 argument".into());
                    }
                    let val = self.eval_expr(&args[0])?;
                    return Ok(Value::Str(val.to_string_value()));
                }
                "int" => {
                    if args.len() != 1 {
                        return Err("int() takes exactly 1 argument".into());
                    }
                    let val = self.eval_expr(&args[0])?;
                    let i = val.to_int()?;
                    return Ok(Value::Int(i));
                }
                "float" => {
                    if args.len() != 1 {
                        return Err("float() takes exactly 1 argument".into());
                    }
                    let val = self.eval_expr(&args[0])?;
                    let f = val.to_float()?;
                    return Ok(Value::Float(f));
                }
                "bool" => {
                    if args.len() != 1 {
                        return Err("bool() takes exactly 1 argument".into());
                    }
                    let val = self.eval_expr(&args[0])?;
                    return Ok(Value::Bool(val.to_bool()));
                }
                "len" => {
                    if args.len() != 1 {
                        return Err("len() takes exactly 1 argument".into());
                    }
                    let val = self.eval_expr(&args[0])?;
                    match val {
                        Value::Array(a) => return Ok(Value::Int(a.len() as i64)),
                        Value::Str(s) => return Ok(Value::Int(s.len() as i64)),
                        Value::Map(m) => return Ok(Value::Int(m.len() as i64)),
                        _ => return Err("len() requires array, string, or map".into()),
                    }
                }
                "type_of" => {
                    if args.len() != 1 {
                        return Err("type_of() takes exactly 1 argument".into());
                    }
                    let val = self.eval_expr(&args[0])?;
                    return Ok(Value::Str(type_name(&val).to_string()));
                }
                "range" => {
                    if args.is_empty() || args.len() > 2 {
                        return Err("range() takes 1 or 2 arguments".into());
                    }
                    if args.len() == 1 {
                        let n = self.eval_expr(&args[0])?.to_int()?;
                        let arr: Vec<Value> = (0..n).map(Value::Int).collect();
                        return Ok(Value::Array(arr));
                    } else {
                        let start = self.eval_expr(&args[0])?.to_int()?;
                        let end = self.eval_expr(&args[1])?.to_int()?;
                        let arr: Vec<Value> = (start..end).map(Value::Int).collect();
                        return Ok(Value::Array(arr));
                    }
                }
                "input" => {
                    if !args.is_empty() {
                        let prompt = self.eval_expr(&args[0])?;
                        print!("{}", prompt);
                        std::io::Write::flush(&mut std::io::stdout()).ok();
                    }
                    let mut line = String::new();
                    std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut line)
                        .map_err(|e| format!("Input error: {}", e))?;
                    return Ok(Value::Str(line.trim_end().to_string()));
                }
                _ => {}
            }
        }

        // User-defined function call
        let func = self.eval_expr(target)?;
        if let Value::Function { params, body, env } = func {
            if params.len() != args.len() {
                return Err(format!(
                    "Function expects {} arguments, got {}",
                    params.len(),
                    args.len()
                ).into());
            }

            // Evaluate arguments in the CURRENT environment
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(self.eval_expr(arg)?);
            }

            let saved_env = std::mem::replace(&mut self.env, env);
            self.env.push_scope();

            for (param, val) in params.iter().zip(arg_values.iter()) {
                self.env.bind_param(param, val.clone());
            }

            let mut result = Value::Unit;
            for stmt in body {
                match self.exec_stmt(&stmt)? {
                    ControlFlow::Return(val) => {
                        result = val;
                        break;
                    }
                    _ => {}
                }
            }

            self.env.pop_scope();
            self.env = saved_env;
            Ok(result)
        } else {
            Err("Not a function".into())
        }
    }

    fn pattern_matches(&mut self, value: &Value, pattern: &Expr) -> Result<bool, HornetError> {
        match pattern {
            Expr::Literal(lit) => {
                let pattern_val = match lit {
                    Literal::Int(n) => Value::Int(*n),
                    Literal::Float(n) => Value::Float(*n),
                    Literal::String(s) => Value::Str(s.clone()),
                    Literal::Bool(b) => Value::Bool(*b),
                    Literal::Unit => Value::Unit,
                };
                Ok(value == &pattern_val)
            }
            Expr::Identifier(name) => {
                if name == "_" {
                    return Ok(true);
                }
                // Variable binding pattern - always matches and binds the variable
                self.env.define(name, value.clone());
                Ok(true)
            }
            _ => Err(format!("Unsupported pattern: {:?}", pattern).into()),
        }
    }

    fn eval_method_call(&mut self, obj: Value, method: &str, args: &[Expr]) -> Result<Value, HornetError> {
        // Evaluate method arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_expr(arg)?);
        }

        match (obj, method) {
            (Value::Int(n), "str") => {
                if !arg_values.is_empty() {
                    return Err("Int.str() takes no arguments".into());
                }
                Ok(Value::Str(n.to_string()))
            }
            (Value::Float(f), "str") => {
                if !arg_values.is_empty() {
                    return Err("Float.str() takes no arguments".into());
                }
                Ok(Value::Str(f.to_string()))
            }
            (Value::Bool(b), "str") => {
                if !arg_values.is_empty() {
                    return Err("Bool.str() takes no arguments".into());
                }
                Ok(Value::Str(if b { "true" } else { "false" }.to_string()))
            }
            (Value::Str(s), "str") => {
                if !arg_values.is_empty() {
                    return Err("Str.str() takes no arguments".into());
                }
                Ok(Value::Str(s))
            }
            (obj, method) => Err(format!("Method '{}' not found on type {}", method, type_name(&obj)).into()),
        }
    }

    fn eval_int_op(&self, left: i64, op: &str, right: i64) -> Result<Value, HornetError> {
        match op {
            "+" => Ok(Value::Int(left + right)),
            "-" => Ok(Value::Int(left - right)),
            "*" => Ok(Value::Int(left * right)),
            "/" => {
                if right == 0 {
                    return Err("Division by zero".into());
                }
                Ok(Value::Float(left as f64 / right as f64))
            }
            "//" => {
                if right == 0 {
                    return Err("Division by zero".into());
                }
                Ok(Value::Int(left / right))
            }
            "%" => {
                if right == 0 {
                    return Err("Modulo by zero".into());
                }
                Ok(Value::Int(left % right))
            }
            "==" => Ok(Value::Bool(left == right)),
            "!=" => Ok(Value::Bool(left != right)),
            "<" => Ok(Value::Bool(left < right)),
            "<=" => Ok(Value::Bool(left <= right)),
            ">" => Ok(Value::Bool(left > right)),
            ">=" => Ok(Value::Bool(left >= right)),
            _ => Err(format!("Unknown operator: {}", op).into()),
        }
    }

    fn eval_float_op(&self, left: f64, op: &str, right: f64) -> Result<Value, HornetError> {
        match op {
            "+" => Ok(Value::Float(left + right)),
            "-" => Ok(Value::Float(left - right)),
            "*" => Ok(Value::Float(left * right)),
            "/" => {
                if right == 0.0 {
                    return Err("Division by zero".into());
                }
                Ok(Value::Float(left / right))
            }
            "//" => {
                if right == 0.0 {
                    return Err("Division by zero".into());
                }
                Ok(Value::Float((left / right).floor()))
            }
            "%" => Ok(Value::Float(left % right)),
            "==" => Ok(Value::Bool((left - right).abs() < 1e-10)),
            "!=" => Ok(Value::Bool((left - right).abs() >= 1e-10)),
            "<" => Ok(Value::Bool(left < right)),
            "<=" => Ok(Value::Bool(left <= right)),
            ">" => Ok(Value::Bool(left > right)),
            ">=" => Ok(Value::Bool(left >= right)),
            _ => Err(format!("Unknown operator: {}", op).into()),
        }
    }

    fn eval_str_op(&self, left: String, op: &str, right: String) -> Result<Value, HornetError> {
        match op {
            "+" => Ok(Value::Str(left + &right)),
            "==" => Ok(Value::Bool(left == right)),
            "!=" => Ok(Value::Bool(left != right)),
            "<" => Ok(Value::Bool(left < right)),
            "<=" => Ok(Value::Bool(left <= right)),
            ">" => Ok(Value::Bool(left > right)),
            ">=" => Ok(Value::Bool(left >= right)),
            _ => Err(format!("Unknown operator for strings: {}", op).into()),
        }
    }

    fn eval_bool_op(&self, left: bool, op: &str, right: bool) -> Result<Value, HornetError> {
        match op {
            "==" => Ok(Value::Bool(left == right)),
            "!=" => Ok(Value::Bool(left != right)),
            _ => Err(format!("Unknown operator for booleans: {}", op).into()),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

fn type_name(val: &Value) -> &'static str {
    match val {
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::Bool(_) => "Bool",
        Value::Str(_) => "Str",
        Value::Unit => "Unit",
        Value::Array(_) => "Array",
        Value::Map(_) => "Map",
        Value::Function { .. } => "Function",
    }
}
