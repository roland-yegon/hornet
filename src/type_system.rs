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
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), HornetError> {
        match stmt {
            Stmt::Assignment { lhs, value, .. } => {
                let _val_type = self.check_expr(value)?;
                if let Expr::Identifier(name) = lhs {
                    self.scopes.last_mut().unwrap().insert(name.clone(), _val_type);
                }
                Ok(())
            },
            Stmt::FunctionDef { name, body, .. } => {
                self.scopes.push(HashMap::new());
                for s in body {
                    self.check_stmt(s)?;
                }
                self.scopes.pop();
                self.scopes.last_mut().unwrap().insert(name.clone(), HornetType::Void);
                Ok(())
            },
            Stmt::If { then_branch, .. } => {
                for s in then_branch {
                    self.check_stmt(s)?;
                }
                Ok(())
            },
            Stmt::StructDef { .. } => Ok(()),
            Stmt::Import(_) => Ok(()),
            Stmt::For { body, .. } => {
                for s in body {
                    self.check_stmt(s)?;
                }
                Ok(())
            },
            Stmt::While { body, .. } => {
                for s in body {
                    self.check_stmt(s)?;
                }
                Ok(())
            },
            Stmt::Expr(_) => Ok(()),
            Stmt::Return(value) => {
                self.check_expr(value)?;
                Ok(())
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<HornetType, HornetError> {
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    Literal::Number(_) => Ok(HornetType::Int),
                    Literal::String(_) => Ok(HornetType::String),
                }
            },
            Expr::Identifier(name) => {
                for scope in self.scopes.iter().rev() {
                    if let Some(t) = scope.get(name) {
                        return Ok(t.clone());
                    }
                }
                Err(HornetError::Type(format!("Undefined variable: {}", name)))
            },
            Expr::BinaryOp { left, right, .. } => {
                let _left_type = self.check_expr(left)?;
                let _right_type = self.check_expr(right)?;
                Ok(HornetType::Int)
            },
            Expr::List(elements) => {
                for el in elements {
                    self.check_expr(el)?;
                }
                Ok(HornetType::Void) // Should be List type
            },
            Expr::NamedArg { value, .. } => self.check_expr(value),
            Expr::Map(pairs) => {
                for (k, v) in pairs {
                    self.check_expr(k)?;
                    self.check_expr(v)?;
                }
                Ok(HornetType::Void)
            },
            Expr::IndexAccess { object, index } => {
                self.check_expr(object)?;
                self.check_expr(index)?;
                Ok(HornetType::Void)
            },
            _ => Ok(HornetType::Void),
        }
    }
}
