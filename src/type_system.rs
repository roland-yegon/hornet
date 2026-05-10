use crate::ast::{Program, Stmt, Expr, Literal, Type};
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

    pub fn analyze(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let val_type = self.check_expr(value)?;
                self.scopes.last_mut().unwrap().insert(name.clone(), val_type);
                Ok(())
            },
            Stmt::Function { name, body, .. } => {
                // Simplified function check
                self.scopes.push(HashMap::new());
                for s in body {
                    self.check_stmt(s)?;
                }
                self.scopes.pop();
                self.scopes.last_mut().unwrap().insert(name.clone(), HornetType::Void);
                Ok(())
            },
            Stmt::If { body, .. } => {
                for s in body {
                    self.check_stmt(s)?;
                }
                Ok(())
            },
            Stmt::StructDef { .. } => Ok(()),
            Stmt::Import(_) => Ok(()),
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<HornetType, String> {
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    Literal::Number(_) => Ok(HornetType::Int),
                    Literal::String(_) => Ok(HornetType::String),
                    Literal::Bool(_) => Ok(HornetType::Bool),
                }
            },
            Expr::Identifier(name) => {
                for scope in self.scopes.iter().rev() {
                    if let Some(t) = scope.get(name) {
                        return Ok(t.clone());
                    }
                }
                Err(format!("Undefined variable: {}", name))
            },
            Expr::BinaryOp { left, right, .. } => {
                let _left_type = self.check_expr(left)?;
                let _right_type = self.check_expr(right)?;
                Ok(HornetType::Int) // Simplified
            },
            _ => Ok(HornetType::Void),
        }
    }
}
