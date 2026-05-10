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
    Custom(String),
}

pub struct TypeSystem {
    scopes: Vec<HashMap<String, HornetType>>,
}

impl TypeSystem {
    pub fn new() -> Self {
        TypeSystem {
            scopes: vec![HashMap::new()],
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
        self.check_stmt_list(&program.statements)
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
            Stmt::Assignment { lhs, value, .. } => {
                let value_type = self.check_expr(value)?;
                if let Expr::Identifier(name) = lhs {
                    self.current_scope().insert(name.clone(), value_type);
                    Ok(())
                } else {
                    Err(HornetError::Type("Left-hand side of assignment must be an identifier".into()))
                }
            },
            Stmt::FunctionDef { name, params, body } => {
                self.current_scope().insert(name.clone(), HornetType::Void);
                self.scopes.push(HashMap::new());
                for param in params {
                    self.current_scope().insert(param.clone(), HornetType::Int);
                }
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

    fn check_expr(&mut self, expr: &Expr) -> Result<HornetType, HornetError> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Number(_) => Ok(HornetType::Int),
                Literal::String(_) => Ok(HornetType::String),
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
                    "+" | "-" | "*" | "/" | "%" => {
                        if left_type == HornetType::Int && right_type == HornetType::Int {
                            Ok(HornetType::Int)
                        } else {
                            Err(HornetError::Type("Arithmetic operators require integer operands".into()))
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
                    "not" => Ok(HornetType::Bool),
                    _ => Err(HornetError::Type(format!("Unsupported operator: {}", op))),
                }
            }
            Expr::List(elements) => {
                for el in elements {
                    self.check_expr(el)?;
                }
                Ok(HornetType::Custom("List".into()))
            }
            Expr::NamedArg { value, .. } => self.check_expr(value),
            Expr::Map(pairs) => {
                for (k, v) in pairs {
                    self.check_expr(k)?;
                    self.check_expr(v)?;
                }
                Ok(HornetType::Custom("Map".into()))
            }
            Expr::IndexAccess { object, index } => {
                self.check_expr(object)?;
                self.check_expr(index)?;
                Ok(HornetType::Int)
            }
            Expr::Call { target, args } => {
                if let Expr::Identifier(name) = &**target {
                    match name.as_str() {
                        "print" => {
                            if args.is_empty() {
                                Err(HornetError::Type("print requires at least one argument".into()))
                            } else {
                                self.check_expr(&args[0])?;
                                Ok(HornetType::Void)
                            }
                        }
                        _ => Ok(HornetType::Void),
                    }
                } else {
                    Err(HornetError::Type("Unsupported call target".into()))
                }
            }
            Expr::Range { .. } => Ok(HornetType::Custom("Range".into())),
            Expr::MemberAccess { .. } => Ok(HornetType::Custom("Member".into())),
        }
    }
}
