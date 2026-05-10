use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    Call {
        target: Box<Expr>,
        args: Vec<Expr>,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Number(i64),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stmt {
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Assignment {
        name: String,
        value: Expr,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_ifs: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
    },
    For {
        iterator: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
