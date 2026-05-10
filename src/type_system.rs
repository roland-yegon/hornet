use crate::ast::{Program, Stmt, Expr};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Void,
    Function { params: Vec<Type>, ret: Box<Type> },
    Unknown,
}

pub struct TypeChecker {
    env: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut env = HashMap::new();
        env.insert("print".to_string(), Type::Function {
            params: vec![Type::Unknown], // Polymorphic print
            ret: Box::new(Type::Void),
        });
        TypeChecker { env }
    }

    pub fn check(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Assignment { name, value } => {
                let val_type = self.infer_expr(value)?;
                self.env.insert(name.clone(), val_type);
                Ok(())
            }
            Stmt::FunctionDef { name, params, body } => {
                // Simplified: assuming unknown param types for now unless annotated
                let mut local_env = self.env.clone();
                for param in params {
                    local_env.insert(param.clone(), Type::Unknown);
                }
                // Check body with local env...
                Ok(())
            }
            Stmt::If { condition, then_branch, else_ifs, else_branch } => {
                let cond_type = self.infer_expr(condition)?;
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    return Err(format!("If condition must be bool, found {:?}", cond_type));
                }
                Ok(())
            }
            Stmt::For { iterator, iterable, body } => {
                Ok(())
            }
            Stmt::Expr(expr) => {
                self.infer_expr(expr)?;
                Ok(())
            }
        }
    }

    fn infer_expr(&self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Number(_) => Ok(Type::Int),
                Literal::String(_) => Ok(Type::String),
            },
            Expr::Identifier(name) => {
                self.env.get(name).cloned().ok_or(format!("Undefined variable: {}", name))
            }
            Expr::BinaryOp { left, op, right } => {
                let _left_t = self.infer_expr(left)?;
                let _right_t = self.infer_expr(right)?;
                // Simplified: assuming numerical ops result in same type
                Ok(Type::Int)
            }
            Expr::Call { target, args } => {
                let _target_t = self.infer_expr(target)?;
                for arg in args {
                    self.infer_expr(arg)?;
                }
                Ok(Type::Unknown)
            }
            Expr::MemberAccess { object: _, member: _ } => {
                Ok(Type::Unknown)
            }
            Expr::Range { .. } => Ok(Type::Unknown),
        }
    }
}
