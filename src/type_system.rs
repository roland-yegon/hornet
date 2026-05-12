use crate::ast::{Program, Stmt, Expr, Literal};
use crate::error::{HornetError, HornetDiagnostic};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HornetType {
    Int,
    Float,
    String,
    Bool,
    Unit,
    Array(Box<HornetType>),
    Map(Box<HornetType>, Box<HornetType>),
    Function(Vec<HornetType>, Box<HornetType>),
    Unknown,
    Never,
    Custom(String),
}

impl HornetType {
    fn name(&self) -> &'static str {
        match self {
            HornetType::Int => "Int",
            HornetType::Float => "Float",
            HornetType::String => "String",
            HornetType::Bool => "Bool",
            HornetType::Unit => "Unit",
            HornetType::Array(_) => "Array",
            HornetType::Map(_, _) => "Map",
            HornetType::Function(_, _) => "Function",
            HornetType::Unknown => "Unknown",
            HornetType::Never => "Never",
            HornetType::Custom(_) => "Custom",
        }
    }
}

pub struct TypeSystem {
    scopes: Vec<HashMap<String, HornetType>>,
    // Track inferred function parameter and return types
    function_signatures: HashMap<String, (Vec<HornetType>, HornetType)>,
}

impl TypeSystem {
    pub fn new() -> Self {
        TypeSystem {
            scopes: vec![HashMap::new()],
            function_signatures: HashMap::new(),
        }
    }

    fn make_diagnostic(&self, what: String, why: String, fix: Vec<String>, docs: String) -> HornetError {
        HornetError::Type(HornetDiagnostic {
            what,
            why,
            fix,
            docs,
            line: 0,
            col: 0,
        })
    }

    fn type_error(&self, what: &str, why: &str, fix: Vec<String>, docs: &str) -> HornetError {
        self.make_diagnostic(what.to_string(), why.to_string(), fix, docs.to_string())
    }

    fn type_error_simple(&self, message: String) -> HornetError {
        self.make_diagnostic(
            message.clone(),
            message.clone(),
            vec!["Fix the type error or use an explicit annotation.".to_string()],
            "hornet.dev/errors/type-error".to_string(),
        )
    }
}

impl Default for TypeSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeSystem {
    pub fn analyze(&mut self, program: &Program) -> Result<(), HornetError> {
        // First pass: collect function signatures with return types
        self.collect_function_signatures(&program.statements)?;
        
        // Second pass: check statements with type information
        self.check_stmt_list(&program.statements)
    }

    fn collect_function_signatures(&mut self, statements: &[Stmt]) -> Result<(), HornetError> {
        for stmt in statements {
            if let Stmt::FunctionDef { name, params, return_type, body: _ } = stmt {
                let return_type = return_type.as_ref().map_or(HornetType::Unknown, |rt| {
                    self.parse_type_name(rt)
                });

                let mut param_types = Vec::new();
                for (param_name, param_type) in params {
                    if let Some(type_name) = param_type {
                        param_types.push(self.parse_type_name(type_name));
                    } else {
                        return Err(self.type_error(
                            "Function parameter types are required",
                            &format!("Parameter '{}' in function '{}' is missing a type annotation", param_name, name),
                            vec![format!("Add a type annotation: {}: Int", param_name)],
                            "hornet.dev/errors/type-annotation",
                        ));
                    }
                }

                self.function_signatures.insert(name.clone(), (param_types, return_type));
            }
        }
        Ok(())
    }

    fn parse_type_name(&self, type_str: &str) -> HornetType {
        match type_str {
            "Int" => HornetType::Int,
            "Float" => HornetType::Float,
            "String" => HornetType::String,
            "Bool" => HornetType::Bool,
            "Unit" | "Void" => HornetType::Unit,
            _ => HornetType::Custom(type_str.to_string()),
        }
    }

    fn check_stmt_list(&mut self, statements: &[Stmt]) -> Result<(), HornetError> {
        for stmt in statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    fn current_scope(&mut self) -> &mut HashMap<String, HornetType> {
        self.scopes.last_mut().expect("scope stack must not be empty")
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), HornetError> {
        match stmt {
            Stmt::Let { name, value } => {
                let value_type = self.check_expr(value)?;
                self.current_scope().insert(name.clone(), value_type);
                Ok(())
            }
            Stmt::Assignment { lhs, value, .. } => {
                let value_type = self.check_expr(value)?;
                if let Expr::Identifier(name) = lhs {
                    // Check if variable is already defined
                    let scope = self.current_scope();
                    if let Some(existing_type) = scope.get(name).cloned() {
                        if existing_type != value_type {
                            return Err(self.type_error_simple(format!(
                                "Type mismatch: variable '{}' has type {:?}, cannot assign type {:?}",
                                name, existing_type, value_type
                            )));
                        }
                    } else {
                        scope.insert(name.clone(), value_type);
                    }
                    Ok(())
                } else {
                    Err(self.type_error_simple("Left-hand side of assignment must be an identifier".into()))
                }
            },
            Stmt::FunctionDef { name, params, body, return_type: _ } => {
                // Get the function signature from the first pass
                let (param_types, return_type) = self.function_signatures.get(name).cloned()
                    .unwrap_or_else(|| {
                        (vec![HornetType::Unknown; params.len()], HornetType::Unknown)
                    });

                // Create function type and add to scope
                let func_type = HornetType::Function(param_types.clone(), Box::new(return_type.clone()));
                self.current_scope().insert(name.clone(), func_type);

                // Create new scope for function body
                self.scopes.push(HashMap::new());

                // Add parameters to function scope
                for (i, (param_name, _)) in params.iter().enumerate() {
                    let param_type = if i < param_types.len() {
                        param_types[i].clone()
                    } else {
                        HornetType::Unknown
                    };
                    self.current_scope().insert(param_name.clone(), param_type);
                }

                // Check function body
                self.check_stmt_list(body)?;

                self.scopes.pop();
                Ok(())
            },
            Stmt::If { condition, then_branch, else_ifs, else_branch } => {
                let cond_type = self.check_expr(condition)?;
                self.assert_bool(cond_type, "if condition")?;
                self.check_stmt_list(then_branch)?;
                for (elif_cond, elif_body) in else_ifs {
                    let cond_type = self.check_expr(elif_cond)?;
                    self.assert_bool(cond_type, "elif condition")?;
                    self.check_stmt_list(elif_body)?;
                }
                if let Some(branch) = else_branch {
                    self.check_stmt_list(branch)?;
                }
                Ok(())
            },
            Stmt::StructDef { .. } => Ok(()),
            Stmt::Import(_) => Ok(()),
            Stmt::Break => Ok(()),
            Stmt::Continue => Ok(()),
            Stmt::Loop { body } => {
                self.check_stmt_list(body)
            },
            Stmt::Match { value, arms } => {
                let _value_type = self.check_expr(value)?;
                for (pattern, body) in arms {
                    // For pattern checking, we need special handling
                    self.check_pattern(pattern)?;
                    self.check_stmt_list(body)?;
                }
                Ok(())
            }
            Stmt::For { iterator, iterable, body } => {
                let iterable_type = self.check_expr(iterable)?;
                if !matches!(iterable_type, HornetType::Int | HornetType::Custom(_)) {
                    return Err(self.type_error_simple("For loop iterable must be a range or collection".into()));
                }
                self.scopes.push(HashMap::new());
                self.current_scope().insert(iterator.clone(), HornetType::Int);
                self.check_stmt_list(body)?;
                self.scopes.pop();
                Ok(())
            },
            Stmt::While { condition, body } => {
                let cond_type = self.check_expr(condition)?;
                self.assert_bool(cond_type, "while condition")?;
                self.check_stmt_list(body)
            },
            Stmt::Expr(expr) => {
                self.check_expr(expr).map(|_| ())
            },
            Stmt::Return(value) => {
                self.check_expr(value).map(|_| ())
            }
        }
    }

    fn assert_bool(&self, value_type: HornetType, context: &str) -> Result<(), HornetError> {
        if matches!(value_type, HornetType::Bool) {
            Ok(())
        } else {
            Err(self.type_error_simple(format!("{} must be a boolean expression", context)))
        }
    }

    fn check_pattern(&mut self, pattern: &Expr) -> Result<(), HornetError> {
        match pattern {
            Expr::Literal(_) => Ok(()), // Literals are always valid patterns
            Expr::Identifier(name) => {
                // Variable pattern - bind it in the current scope
                self.current_scope().insert(name.clone(), HornetType::Unknown); // Type will be inferred
                Ok(())
            }
            _ => Err(self.type_error_simple(format!("Unsupported pattern: {:?}", pattern))),
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<HornetType, HornetError> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(_) => Ok(HornetType::Int),
                Literal::Float(_) => Ok(HornetType::Float),
                Literal::String(_) => Ok(HornetType::String),
                Literal::Bool(_) => Ok(HornetType::Bool),
                Literal::Unit => Ok(HornetType::Unit),
            },
            Expr::Identifier(name) => {
                for scope in self.scopes.iter().rev() {
                    if let Some(t) = scope.get(name) {
                        return Ok(t.clone());
                    }
                }
                Err(self.type_error_simple(format!("Undefined variable: {}", name)))
            }
            Expr::BinaryOp { left, op, right } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;
                match op.as_str() {
                    "+" | "-" | "*" | "/" | "%" | "//" => {
                        // Check if operands are numeric or Void (untyped)
                        let left_numeric = matches!(left_type, HornetType::Int | HornetType::Float | HornetType::Unknown);
                        let right_numeric = matches!(right_type, HornetType::Int | HornetType::Float | HornetType::Unknown);
                        
                        if left_numeric && right_numeric {
                            // Type inference for untyped operands
                            if left_type == HornetType::Unknown && right_type == HornetType::Unknown {
                                Ok(HornetType::Int) // Default to Int for untyped
                            } else if left_type == HornetType::Float || right_type == HornetType::Float {
                                Ok(if op == "//" { HornetType::Int } else { HornetType::Float })
                            } else if op == "/" {
                                Ok(HornetType::Float) // division always returns float
                            } else if op == "//" {
                                Ok(HornetType::Int) // floor division returns int
                            } else {
                                Ok(HornetType::Int)
                            }
                        } else {
                            Err(self.type_error_simple("Arithmetic operators require numeric operands".into()))
                        }
                    }
                    "==" | "!=" | "<" | ">" | "<=" | ">=" => Ok(HornetType::Bool),
                    "and" | "or" => {
                        if left_type == HornetType::Bool && right_type == HornetType::Bool {
                            Ok(HornetType::Bool)
                        } else {
                            Err(self.type_error_simple("Logical operators require boolean operands".into()))
                        }
                    }
                    _ => Err(self.type_error_simple(format!("Unsupported operator: {}", op))),
                }
            }
            Expr::UnaryOp { op, operand } => {
                let operand_type = self.check_expr(operand)?;
                match op.as_str() {
                    "not" => {
                        if operand_type == HornetType::Bool {
                            Ok(HornetType::Bool)
                        } else {
                            Err(self.type_error_simple("not operator requires boolean operand".into()))
                        }
                    }
                    "-" => {
                        if operand_type == HornetType::Int || operand_type == HornetType::Float {
                            Ok(operand_type)
                        } else {
                            Err(self.type_error_simple("Negation requires numeric operand".into()))
                        }
                    }
                    _ => Err(self.type_error_simple(format!("Unknown unary operator: {}", op))),
                }
            }
            Expr::List(elements) => {
                if elements.is_empty() {
                    return Err(self.type_error(
                        "Cannot infer type for empty array",
                        "Empty arrays must include a type annotation so the element type can be inferred.",
                        vec!["Use a typed binding or populate the array with a literal value.".to_string()],
                        "hornet.dev/errors/empty-array-inference",
                    ));
                }
                let first_type = self.check_expr(&elements[0])?;
                // Check that all elements have the same type
                for el in &elements[1..] {
                    let el_type = self.check_expr(el)?;
                    if el_type != first_type {
                        return Err(self.type_error(
                            "All array elements must have the same type",
                            "Array literals must be homogeneous in Hornet.",
                            vec!["Use values of the same type inside the array.".to_string()],
                            "hornet.dev/errors/array-homogeneity",
                        ));
                    }
                }
                Ok(HornetType::Array(Box::new(first_type)))
            }
            Expr::NamedArg { value, .. } => self.check_expr(value),
            Expr::Map(pairs) => {
                if pairs.is_empty() {
                    return Err(self.type_error(
                        "Cannot infer type for empty map",
                        "Empty maps must include a type annotation so the key and value types can be inferred.",
                        vec!["Use a typed binding or add a literal key/value pair.".to_string()],
                        "hornet.dev/errors/empty-map-inference",
                    ));
                } else {
                    let (first_key, first_val) = &pairs[0];
                    let key_type = self.check_expr(first_key)?;
                    let val_type = self.check_expr(first_val)?;

                    // Check that all keys and values have the same types
                    for (k, v) in &pairs[1..] {
                        let k_type = self.check_expr(k)?;
                        let v_type = self.check_expr(v)?;
                        if k_type != key_type || v_type != val_type {
                            return Err(self.type_error_simple("All map keys and values must have the same types".into()));
                        }
                    }
                    Ok(HornetType::Map(Box::new(key_type), Box::new(val_type)))
                }
            }
            Expr::IndexAccess { object, index } => {
                let obj_type = self.check_expr(object)?;
                let index_type = self.check_expr(index)?;

                // Index should be Int
                if index_type != HornetType::Int {
                    return Err(self.type_error_simple("Index must be an integer".into()));
                }

                match obj_type {
                    HornetType::Array(element_type) => Ok(*element_type),
                    HornetType::Map(_, value_type) => Ok(*value_type),
                    HornetType::String => Ok(HornetType::String), // String indexing returns String (single char)
                    _ => Err(self.type_error_simple("Cannot index this type".into())),
                }
            }
            Expr::Call { target, args } => {
                match &**target {
                    Expr::Identifier(name) => {
                        match name.as_str() {
                            "print" | "println" => {
                                for arg in args {
                                    self.check_expr(arg)?;
                                }
                                Ok(HornetType::Unit)
                            }
                            "str" => {
                                if args.len() != 1 {
                                    return Err(self.type_error_simple("str() takes exactly 1 argument".into()));
                                }
                                self.check_expr(&args[0])?;
                                Ok(HornetType::String)
                            }
                            "int" => {
                                if args.len() != 1 {
                                    return Err(self.type_error_simple("int() takes exactly 1 argument".into()));
                                }
                                self.check_expr(&args[0])?;
                                Ok(HornetType::Int)
                            }
                            "float" => {
                                if args.len() != 1 {
                                    return Err(self.type_error_simple("float() takes exactly 1 argument".into()));
                                }
                                self.check_expr(&args[0])?;
                                Ok(HornetType::Float)
                            }
                            "bool" => {
                                if args.len() != 1 {
                                    return Err(self.type_error_simple("bool() takes exactly 1 argument".into()));
                                }
                                self.check_expr(&args[0])?;
                                Ok(HornetType::Bool)
                            }
                            "len" => {
                                if args.len() != 1 {
                                    return Err(self.type_error_simple("len() takes exactly 1 argument".into()));
                                }
                                self.check_expr(&args[0])?;
                                Ok(HornetType::Int)
                            }
                            "type_of" => {
                                if args.len() != 1 {
                                    return Err(self.type_error_simple("type_of() takes exactly 1 argument".into()));
                                }
                                self.check_expr(&args[0])?;
                                Ok(HornetType::String)
                            }
                            "range" => {
                                for arg in args {
                                    self.check_expr(arg)?;
                                }
                                Ok(HornetType::Array(Box::new(HornetType::Int)))
                            }
                            "input" => {
                                if args.len() > 1 {
                                    return Err(self.type_error_simple("input() takes at most 1 argument".into()));
                                }
                                if !args.is_empty() {
                                    self.check_expr(&args[0])?;
                                }
                                Ok(HornetType::String)
                            }
                            "assert" => {
                                if args.len() != 1 {
                                    return Err(self.type_error_simple("assert() takes exactly 1 argument".into()));
                                }
                                let arg_type = self.check_expr(&args[0])?;
                                if arg_type != HornetType::Bool {
                                    return Err(self.type_error_simple("assert() requires boolean argument".into()));
                                }
                                Ok(HornetType::Unit)
                            }
                            _ => {
                                // User-defined function call
                                let func_type = self.check_expr(target)?;
                                if let HornetType::Function(param_types, return_type) = func_type {
                                    if param_types.len() != args.len() {
                                        return Err(self.type_error_simple(format!(
                                            "Function expects {} arguments, got {}",
                                            param_types.len(),
                                            args.len()
                                        )));
                                    }
                                    // Check argument types match parameter types
                                    for (i, arg) in args.iter().enumerate() {
                                        let arg_type = self.check_expr(arg)?;
                                        // For now, skip type checking if param type is Unknown (uninferred)
                                        if param_types[i] != HornetType::Unknown && arg_type != param_types[i] {
                                            return Err(self.type_error_simple(format!(
                                                "Argument {} type mismatch: expected {:?}, got {:?}",
                                                i + 1, param_types[i], arg_type
                                            )));
                                        }
                                    }
                                    Ok(*return_type)
                                } else {
                                    Err(self.type_error_simple(format!("'{}' is not a function", name)))
                                }
                            }
                        }
                    }
                    Expr::MemberAccess { object, member } => {
                        // Method call: check the object and return a type based on the method
                        let obj_type = self.check_expr(object)?;
                        match member.as_str() {
                            "str" => Ok(HornetType::String),
                            _ => Err(self.type_error_simple(format!("Unknown method '{}' on type {:?}", member, obj_type))),
                        }
                    }
                    _ => Err(self.type_error_simple("Unsupported call target".into())),
                }
            }
            Expr::Range { .. } => Ok(HornetType::Custom("Range".into())),
            Expr::MemberAccess { .. } => Ok(HornetType::Custom("Member".into())),
        }
    }
}
