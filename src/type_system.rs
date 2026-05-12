use crate::ast::{Program, Stmt, Expr, Literal};
use crate::error::HornetError;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HornetType {
    Int,
    Float,
    String,
    Bool,
    Void,
    Array(Box<HornetType>),
    Map(Box<HornetType>, Box<HornetType>),
    Function(Vec<HornetType>, Box<HornetType>),
    Custom(String),
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
                let return_type = return_type.as_ref().map_or(HornetType::Void, |rt| {
                    self.parse_type_name(rt)
                });

                // For now, all parameters are inferred as Void, will be refined during checking
                let param_types = vec![HornetType::Void; params.len()];

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
            "Void" => HornetType::Void,
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
                    if let Some(existing_type) = self.current_scope().get(name) {
                        if *existing_type != value_type {
                            return Err(HornetError::Type(format!(
                                "Type mismatch: variable '{}' has type {:?}, cannot assign type {:?}",
                                name, existing_type, value_type
                            )));
                        }
                    } else {
                        self.current_scope().insert(name.clone(), value_type);
                    }
                    Ok(())
                } else {
                    Err(HornetError::Type("Left-hand side of assignment must be an identifier".into()))
                }
            },
            Stmt::FunctionDef { name, params, body, return_type: _ } => {
                // Get the function signature from the first pass
                let (param_types, return_type) = self.function_signatures.get(name).cloned()
                    .unwrap_or_else(|| {
                        (vec![HornetType::Void; params.len()], HornetType::Void)
                    });

                // Create function type and add to scope
                let func_type = HornetType::Function(param_types.clone(), Box::new(return_type.clone()));
                self.current_scope().insert(name.clone(), func_type);

                // Create new scope for function body
                self.scopes.push(HashMap::new());

                // Add parameters to function scope
                for (i, param) in params.iter().enumerate() {
                    let param_type = if i < param_types.len() {
                        param_types[i].clone()
                    } else {
                        HornetType::Void
                    };
                    self.current_scope().insert(param.clone(), param_type);
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
                    return Err(HornetError::Type("For loop iterable must be a range or collection".into()));
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
            Err(HornetError::Type(format!("{} must be a boolean expression", context)))
        }
    }

    fn check_pattern(&mut self, pattern: &Expr) -> Result<(), HornetError> {
        match pattern {
            Expr::Literal(_) => Ok(()), // Literals are always valid patterns
            Expr::Identifier(name) => {
                // Variable pattern - bind it in the current scope
                self.current_scope().insert(name.clone(), HornetType::Void); // Type will be inferred
                Ok(())
            }
            _ => Err(HornetError::Type(format!("Unsupported pattern: {:?}", pattern))),
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<HornetType, HornetError> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(_) => Ok(HornetType::Int),
                Literal::Float(_) => Ok(HornetType::Float),
                Literal::String(_) => Ok(HornetType::String),
                Literal::Bool(_) => Ok(HornetType::Bool),
                Literal::Unit => Ok(HornetType::Void),
            },
            Expr::Identifier(name) => {
                for scope in self.scopes.iter().rev() {
                    if let Some(t) = scope.get(name) {
                        return Ok(t.clone());
                    }
                }
                Err(HornetError::Type(format!("Undefined variable: {}", name)))
            }
            Expr::BinaryOp { left, op, right } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;
                match op.as_str() {
                    "+" | "-" | "*" | "/" | "%" | "//" => {
                        // Check if operands are numeric or Void (untyped)
                        let left_numeric = matches!(left_type, HornetType::Int | HornetType::Float | HornetType::Void);
                        let right_numeric = matches!(right_type, HornetType::Int | HornetType::Float | HornetType::Void);
                        
                        if left_numeric && right_numeric {
                            // Type inference for untyped operands
                            if left_type == HornetType::Void && right_type == HornetType::Void {
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
                            Err(HornetError::Type("Arithmetic operators require numeric operands".into()))
                        }
                    }
                    "==" | "!=" | "<" | ">" | "<=" | ">=" => Ok(HornetType::Bool),
                    "and" | "or" => {
                        if left_type == HornetType::Bool && right_type == HornetType::Bool {
                            Ok(HornetType::Bool)
                        } else {
                            Err(HornetError::Type("Logical operators require boolean operands".into()))
                        }
                    }
                    _ => Err(HornetError::Type(format!("Unsupported operator: {}", op))),
                }
            }
            Expr::UnaryOp { op, operand } => {
                let operand_type = self.check_expr(operand)?;
                match op.as_str() {
                    "not" => {
                        if operand_type == HornetType::Bool {
                            Ok(HornetType::Bool)
                        } else {
                            Err(HornetError::Type("not operator requires boolean operand".into()))
                        }
                    }
                    "-" => {
                        if operand_type == HornetType::Int || operand_type == HornetType::Float {
                            Ok(operand_type)
                        } else {
                            Err(HornetError::Type("Negation requires numeric operand".into()))
                        }
                    }
                    _ => Err(HornetError::Type(format!("Unknown unary operator: {}", op))),
                }
            }
            Expr::List(elements) => {
                if elements.is_empty() {
                    Ok(HornetType::Array(Box::new(HornetType::Void)))
                } else {
                    let first_type = self.check_expr(&elements[0])?;
                    // Check that all elements have the same type
                    for el in &elements[1..] {
                        let el_type = self.check_expr(el)?;
                        if el_type != first_type {
                            return Err(HornetError::Type("All array elements must have the same type".into()));
                        }
                    }
                    Ok(HornetType::Array(Box::new(first_type)))
                }
            }
            Expr::NamedArg { value, .. } => self.check_expr(value),
            Expr::Map(pairs) => {
                if pairs.is_empty() {
                    Ok(HornetType::Map(Box::new(HornetType::Void), Box::new(HornetType::Void)))
                } else {
                    let (first_key, first_val) = &pairs[0];
                    let key_type = self.check_expr(first_key)?;
                    let val_type = self.check_expr(first_val)?;

                    // Check that all keys and values have the same types
                    for (k, v) in &pairs[1..] {
                        let k_type = self.check_expr(k)?;
                        let v_type = self.check_expr(v)?;
                        if k_type != key_type || v_type != val_type {
                            return Err(HornetError::Type("All map keys and values must have the same types".into()));
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
                    return Err(HornetError::Type("Index must be an integer".into()));
                }

                match obj_type {
                    HornetType::Array(element_type) => Ok(*element_type),
                    HornetType::Map(_, value_type) => Ok(*value_type),
                    HornetType::String => Ok(HornetType::String), // String indexing returns String (single char)
                    _ => Err(HornetError::Type("Cannot index this type".into())),
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
                                Ok(HornetType::Void)
                            }
                            "str" => {
                                if args.len() != 1 {
                                    return Err(HornetError::Type("str() takes exactly 1 argument".into()));
                                }
                                self.check_expr(&args[0])?;
                                Ok(HornetType::String)
                            }
                            "int" => {
                                if args.len() != 1 {
                                    return Err(HornetError::Type("int() takes exactly 1 argument".into()));
                                }
                                self.check_expr(&args[0])?;
                                Ok(HornetType::Int)
                            }
                            "float" => {
                                if args.len() != 1 {
                                    return Err(HornetError::Type("float() takes exactly 1 argument".into()));
                                }
                                self.check_expr(&args[0])?;
                                Ok(HornetType::Float)
                            }
                            "bool" => {
                                if args.len() != 1 {
                                    return Err(HornetError::Type("bool() takes exactly 1 argument".into()));
                                }
                                self.check_expr(&args[0])?;
                                Ok(HornetType::Bool)
                            }
                            "len" => {
                                if args.len() != 1 {
                                    return Err(HornetError::Type("len() takes exactly 1 argument".into()));
                                }
                                self.check_expr(&args[0])?;
                                Ok(HornetType::Int)
                            }
                            "type_of" => {
                                if args.len() != 1 {
                                    return Err(HornetError::Type("type_of() takes exactly 1 argument".into()));
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
                                    return Err(HornetError::Type("input() takes at most 1 argument".into()));
                                }
                                if !args.is_empty() {
                                    self.check_expr(&args[0])?;
                                }
                                Ok(HornetType::String)
                            }
                            "assert" => {
                                if args.len() != 1 {
                                    return Err(HornetError::Type("assert() takes exactly 1 argument".into()));
                                }
                                let arg_type = self.check_expr(&args[0])?;
                                if arg_type != HornetType::Bool {
                                    return Err(HornetError::Type("assert() requires boolean argument".into()));
                                }
                                Ok(HornetType::Void)
                            }
                            _ => {
                                // User-defined function call
                                let func_type = self.check_expr(target)?;
                                if let HornetType::Function(param_types, return_type) = func_type {
                                    if param_types.len() != args.len() {
                                        return Err(HornetError::Type(format!(
                                            "Function expects {} arguments, got {}",
                                            param_types.len(),
                                            args.len()
                                        )));
                                    }
                                    // Check argument types match parameter types
                                    for (i, arg) in args.iter().enumerate() {
                                        let arg_type = self.check_expr(arg)?;
                                        // For now, skip type checking if param type is Void (uninferred)
                                        if param_types[i] != HornetType::Void && arg_type != param_types[i] {
                                            return Err(HornetError::Type(format!(
                                                "Argument {} type mismatch: expected {:?}, got {:?}",
                                                i + 1, param_types[i], arg_type
                                            )));
                                        }
                                    }
                                    Ok(*return_type)
                                } else {
                                    Err(HornetError::Type(format!("'{}' is not a function", name)))
                                }
                            }
                        }
                    }
                    Expr::MemberAccess { object, member } => {
                        // Method call: check the object and return a type based on the method
                        let obj_type = self.check_expr(object)?;
                        match member.as_str() {
                            "str" => Ok(HornetType::String),
                            _ => Err(HornetError::Type(format!("Unknown method '{}' on type {:?}", member, obj_type))),
                        }
                    }
                    _ => Err(HornetError::Type("Unsupported call target".into())),
                }
            }
            Expr::Range { .. } => Ok(HornetType::Custom("Range".into())),
            Expr::MemberAccess { .. } => Ok(HornetType::Custom("Member".into())),
        }
    }
}
