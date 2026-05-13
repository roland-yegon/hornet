use crate::error::HornetError;
use crate::lexer::{Token, TokenType};
use crate::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self, offset: isize) -> Token {
        let idx = self.pos as isize + offset;
        if idx >= 0 {
            let idx = idx as usize;
            if idx < self.tokens.len() {
                self.tokens[idx].clone()
            } else {
                self.tokens.last().cloned().unwrap_or(Token { token_type: TokenType::Eof, line: 0, column: 0 })
            }
        } else {
            self.tokens.first().cloned().unwrap_or(Token { token_type: TokenType::Eof, line: 0, column: 0 })
        }
    }

    fn advance(&mut self) -> Token {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            token
        } else {
            self.tokens.last().cloned().unwrap_or(Token { token_type: TokenType::Eof, line: 0, column: 0 })
        }
    }

    fn match_token(&mut self, types: &[TokenType]) -> Option<Token> {
        for token_type in types {
            if std::mem::discriminant(&self.peek(0).token_type) == std::mem::discriminant(token_type) {
                return Some(self.advance());
            }
        }
        None
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, HornetError> {
        if std::mem::discriminant(&self.peek(0).token_type) == std::mem::discriminant(&token_type) {
            Ok(self.advance())
        } else {
            Err(HornetError::Parser(format!("{}: expected {:?}, got {:?}", message, token_type, self.peek(0).token_type)))
        }
    }

    fn expect_identifier(&mut self, context: &str) -> Result<String, HornetError> {
        match self.advance().token_type {
            TokenType::Identifier(name) => Ok(name),
            TokenType::From => Ok("from".to_string()),
            TokenType::To => Ok("to".to_string()),
            TokenType::Upto => Ok("upto".to_string()),
            other => Err(HornetError::Parser(format!("Expected identifier for {}, got {:?}", context, other))),
        }
    }

    pub fn parse(&mut self) -> Result<Program, HornetError> {
        let mut statements = Vec::new();
        while !matches!(self.peek(0).token_type, TokenType::Eof) {
            if matches!(self.peek(0).token_type, TokenType::Newline) {
                self.advance();
                continue;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Stmt, HornetError> {
        match &self.peek(0).token_type {
            TokenType::Define => self.parse_function_def(),
            TokenType::If => self.parse_if_stmt(),
            TokenType::For => self.parse_for_stmt(),
            TokenType::While => self.parse_while_stmt(),
            TokenType::Repeat => self.parse_repeat_stmt(),
            TokenType::Check => self.parse_check_stmt(),
            TokenType::Break => {
                self.advance();
                Ok(Stmt::Break)
            }
            TokenType::Continue => {
                self.advance();
                Ok(Stmt::Continue)
            }
            TokenType::Use => self.parse_use_stmt(),
            TokenType::Record => self.parse_record_def(),
            TokenType::Return => self.parse_return(),
            TokenType::Let => self.parse_let_stmt(),
            _ => {
                let expr = self.parse_expression()?;
                if matches!(self.peek(0).token_type, TokenType::Equals) {
                    self.advance(); // =
                    let value = self.parse_expression()?;
                    Ok(Stmt::Assignment { lhs: expr, value })
                } else {
                    Ok(Stmt::Expr(expr))
                }
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, HornetError> {
        self.advance(); // let
        let name = self.expect_identifier("variable name")?;
        self.consume(TokenType::Equals, "Expected '='")?;
        let value = self.parse_expression()?;
        Ok(Stmt::Let { name, value })
    }

    fn parse_repeat_stmt(&mut self) -> Result<Stmt, HornetError> {
        self.advance(); // repeat
        self.consume(TokenType::Colon, "Expected ':'")?;
        let body = self.parse_block()?;
        Ok(Stmt::Loop { body })
    }

    fn parse_use_stmt(&mut self) -> Result<Stmt, HornetError> {
        self.advance(); // use
        let name = self.expect_identifier("module path")?;
        Ok(Stmt::Use(name))
    }

    fn parse_record_def(&mut self) -> Result<Stmt, HornetError> {
        self.advance(); // record
        let name = self.expect_identifier("record name")?;
        self.consume(TokenType::Colon, "Expected ':'")?;
        self.consume(TokenType::Newline, "Expected newline")?;
        self.consume(TokenType::Indent(0), "Expected indent")?;
        let mut fields = Vec::new();
        while !matches!(self.peek(0).token_type, TokenType::Dedent) {
            if matches!(self.peek(0).token_type, TokenType::Newline) {
                self.advance();
                continue;
            }
            let field_name = self.expect_identifier("record field name")?;
            self.consume(TokenType::Colon, "Expected ':'")?;
            let field_type = self.expect_identifier("record field type")?;
            fields.push((field_name, field_type));
            if matches!(self.peek(0).token_type, TokenType::Comma) {
                self.advance();
            }
        }
        self.consume(TokenType::Dedent, "Expected dedent")?;
        Ok(Stmt::RecordDef { name, fields })
    }

    fn parse_function_def(&mut self) -> Result<Stmt, HornetError> {
        self.advance(); // define
        let name = self.expect_identifier("function name")?;
        self.consume(TokenType::LParen, "Expected '('")?;
        let mut params = Vec::new();
        if !matches!(self.peek(0).token_type, TokenType::RParen) {
            params.push(self.parse_function_param()?);
            while matches!(self.peek(0).token_type, TokenType::Comma) {
                self.advance();
                params.push(self.parse_function_param()?);
            }
        }
        self.consume(TokenType::RParen, "Expected ')'")?;

        let return_type = if self.match_token(&[TokenType::Gives]).is_some() {
            Some(self.expect_identifier("return type")?)
        } else {
            None
        };

        self.consume(TokenType::Colon, "Expected ':'")?;
        let body = self.parse_block()?;
        Ok(Stmt::FunctionDef { name, params, return_type, body })
    }

    fn parse_function_param(&mut self) -> Result<(String, Option<String>), HornetError> {
        let name = self.expect_identifier("function parameter")?;
        let param_type = if self.match_token(&[TokenType::Colon]).is_some() {
            Some(self.expect_identifier("parameter type")?)
        } else {
            None
        };
        Ok((name, param_type))
    }
    
    fn parse_return(&mut self) -> Result<Stmt, HornetError> {
        self.advance(); // return
        let value = self.parse_expression()?;
        Ok(Stmt::Return(value))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, HornetError> {
        self.consume(TokenType::Newline, "Expected newline")?;
        self.consume(TokenType::Indent(0), "Expected indent")?;
        let mut statements = Vec::new();
        while !matches!(self.peek(0).token_type, TokenType::Dedent | TokenType::Eof) {
            if matches!(self.peek(0).token_type, TokenType::Newline) {
                self.advance();
                continue;
            }
            statements.push(self.parse_statement()?);
        }
        self.consume(TokenType::Dedent, "Expected dedent")?;
        Ok(statements)
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, HornetError> {
        self.advance(); // if
        let condition = self.parse_expression()?;
        self.consume(TokenType::Colon, "Expected ':'")?;
        let then_branch = self.parse_block()?;
        
        let mut else_ifs = Vec::new();
        while matches!(self.peek(0).token_type, TokenType::Else) && matches!(self.peek(1).token_type, TokenType::If) {
            self.advance(); // else
            self.advance(); // if
            let cond = self.parse_expression()?;
            self.consume(TokenType::Colon, "Expected ':'")?;
            let branch = self.parse_block()?;
            else_ifs.push((cond, branch));
        }
        
        let mut else_branch = None;
        if matches!(self.peek(0).token_type, TokenType::Else) {
            self.advance(); // else
            self.consume(TokenType::Colon, "Expected ':'")?;
            else_branch = Some(self.parse_block()?);
        }
        Ok(Stmt::If { condition, then_branch, else_ifs, else_branch })
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, HornetError> {
        self.advance(); // for
        let iterator = self.expect_identifier("for iterator")?;
        self.consume(TokenType::From, "Expected 'from' after iterator")?;
        let start = self.parse_expression()?;
        let inclusive = match &self.peek(0).token_type {
            TokenType::To => {
                self.advance();
                true
            }
            TokenType::Upto => {
                self.advance();
                false
            }
            other => {
                return Err(HornetError::Parser(format!("Expected 'to' or 'upto' in for loop, got {:?}", other)));
            }
        };
        let end = self.parse_expression()?;
        self.consume(TokenType::Colon, "Expected ':'")?;
        let body = self.parse_block()?;
        Ok(Stmt::For { iterator, iterable: Expr::Range { start: Box::new(start), end: Box::new(end), inclusive }, body })
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, HornetError> {
        self.advance(); // while
        let condition = self.parse_expression()?;
        self.consume(TokenType::Colon, "Expected ':'")?;
        let body = self.parse_block()?;
        Ok(Stmt::While { condition, body })
    }

    fn parse_check_stmt(&mut self) -> Result<Stmt, HornetError> {
        self.advance(); // check
        let value = self.parse_expression()?;
        self.consume(TokenType::Colon, "Expected ':' after check expression")?;
        self.consume(TokenType::Newline, "Expected newline")?;
        self.consume(TokenType::Indent(0), "Expected indent")?;

        let mut arms = Vec::new();
        while matches!(self.peek(0).token_type, TokenType::When) {
            self.advance(); // when
            let pattern = self.parse_expression()?;
            self.consume(TokenType::Colon, "Expected ':' after when pattern")?;
            let body = self.parse_block()?;
            arms.push((pattern, body));
            while matches!(self.peek(0).token_type, TokenType::Newline) {
                self.advance();
            }
        }

        if matches!(self.peek(0).token_type, TokenType::Otherwise) {
            self.advance(); // otherwise
            self.consume(TokenType::Colon, "Expected ':' after otherwise")?;
            let body = self.parse_block()?;
            arms.push((Expr::Identifier("_".to_string()), body));
        }

        self.consume(TokenType::Dedent, "Expected dedent after check block")?;
        Ok(Stmt::Match { value, arms })
    }

    fn parse_expression(&mut self) -> Result<Expr, HornetError> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expr, HornetError> {
        let mut node = self.parse_logical_and()?;
        while self.match_token(&[TokenType::Or]).is_some() {
            node = Expr::BinaryOp {
                left: Box::new(node),
                op: "or".to_string(),
                right: Box::new(self.parse_logical_and()?),
            };
        }
        Ok(node)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, HornetError> {
        let mut node = self.parse_equality()?;
        while self.match_token(&[TokenType::And]).is_some() {
            node = Expr::BinaryOp {
                left: Box::new(node),
                op: "and".to_string(),
                right: Box::new(self.parse_equality()?),
            };
        }
        Ok(node)
    }

    fn parse_equality(&mut self) -> Result<Expr, HornetError> {
        let mut node = self.parse_comparison()?;
        while let Some(tok) = self.match_token(&[TokenType::EqEq, TokenType::Neq]) {
            let op = match tok.token_type {
                TokenType::EqEq => "==".to_string(),
                TokenType::Neq => "!=".to_string(),
                _ => unreachable!(),
            };
            node = Expr::BinaryOp {
                left: Box::new(node),
                op,
                right: Box::new(self.parse_comparison()?),
            };
        }
        Ok(node)
    }

    fn parse_comparison(&mut self) -> Result<Expr, HornetError> {
        let mut node = self.parse_addition()?;
        while let Some(tok) = self.match_token(&[TokenType::Lt, TokenType::Le, TokenType::Gt, TokenType::Ge, TokenType::Is, TokenType::Isnt, TokenType::Above, TokenType::Below, TokenType::Atleast, TokenType::Atmost]) {
            let op = match tok.token_type {
                TokenType::Lt | TokenType::Below => "<".to_string(),
                TokenType::Le | TokenType::Atmost => "<=".to_string(),
                TokenType::Gt | TokenType::Above => ">".to_string(),
                TokenType::Ge | TokenType::Atleast => ">=".to_string(),
                TokenType::Is => "==".to_string(),
                TokenType::Isnt => "!=".to_string(),
                _ => unreachable!(),
            };
            node = Expr::BinaryOp {
                left: Box::new(node),
                op,
                right: Box::new(self.parse_addition()?),
            };
        }
        Ok(node)
    }

    fn parse_addition(&mut self) -> Result<Expr, HornetError> {
        let mut node = self.parse_multiplication()?;
        while let Some(tok) = self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let op = match tok.token_type {
                TokenType::Plus => "+".to_string(),
                TokenType::Minus => "-".to_string(),
                _ => unreachable!(),
            };
            node = Expr::BinaryOp {
                left: Box::new(node),
                op,
                right: Box::new(self.parse_multiplication()?),
            };
        }
        Ok(node)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, HornetError> {
        let mut node = self.parse_factor()?;
        while let Some(tok) = self.match_token(&[TokenType::Star, TokenType::Slash, TokenType::FloorDiv, TokenType::Percent]) {
            let op = match tok.token_type {
                TokenType::Star => "*".to_string(),
                TokenType::Slash => "/".to_string(),
                TokenType::FloorDiv => "//".to_string(),
                TokenType::Percent => "%".to_string(),
                _ => unreachable!(),
            };
            node = Expr::BinaryOp {
                left: Box::new(node),
                op,
                right: Box::new(self.parse_factor()?),
            };
        }
        Ok(node)
    }

    fn parse_factor(&mut self) -> Result<Expr, HornetError> {
        // Handle unary operators
        if let Some(_) = self.match_token(&[TokenType::Minus]) {
            let operand = self.parse_factor()?;
            return Ok(Expr::UnaryOp {
                op: "-".to_string(),
                operand: Box::new(operand),
            });
        }
        if let Some(_) = self.match_token(&[TokenType::Not]) {
            let operand = self.parse_factor()?;
            return Ok(Expr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(operand),
            });
        }

        let mut node = if self.match_token(&[TokenType::LParen]).is_some() {
            let expr = self.parse_expression()?;
            self.consume(TokenType::RParen, "Expected ')'")?;
            expr
        } else {
            let tok = self.advance();
            match tok.token_type {
                TokenType::Int(n) => Expr::Literal(Literal::Int(n)),
                TokenType::Float(f) => Expr::Literal(Literal::Float(f)),
                TokenType::String(s) => Expr::Literal(Literal::String(s)),
                TokenType::True => Expr::Literal(Literal::Bool(true)),
                TokenType::False => Expr::Literal(Literal::Bool(false)),
                TokenType::Identifier(i) => Expr::Identifier(i),
                TokenType::From => Expr::Identifier("from".to_string()),
                TokenType::To => Expr::Identifier("to".to_string()),
                TokenType::Upto => Expr::Identifier("upto".to_string()),
                TokenType::LBracket => {
                    let mut elements = Vec::new();
                    if !matches!(self.peek(0).token_type, TokenType::RBracket) {
                        elements.push(self.parse_expression()?);
                        while matches!(self.peek(0).token_type, TokenType::Comma) {
                            self.advance();
                            if matches!(self.peek(0).token_type, TokenType::RBracket) {
                                break;
                            }
                            elements.push(self.parse_expression()?);
                        }
                    }
                    self.consume(TokenType::RBracket, "Expected ']'")?;
                    Expr::List(elements)
                }
                TokenType::LBrace => {
                    let mut pairs = Vec::new();
                    if !matches!(self.peek(0).token_type, TokenType::RBrace) {
                        let key = self.parse_expression()?;
                        self.consume(TokenType::Colon, "Expected ':'")?;
                        let val = self.parse_expression()?;
                        pairs.push((key, val));
                        while matches!(self.peek(0).token_type, TokenType::Comma) {
                            self.advance();
                            if matches!(self.peek(0).token_type, TokenType::RBrace) {
                                break;
                            }
                            let key = self.parse_expression()?;
                            self.consume(TokenType::Colon, "Expected ':'")?;
                            let val = self.parse_expression()?;
                            pairs.push((key, val));
                        }
                    }
                    self.consume(TokenType::RBrace, "Expected '}'")?;
                    Expr::Map(pairs)
                }
                other => return Err(HornetError::Parser(format!("Unexpected token {:?} at line {}", other, tok.line))),
            }
        };

        loop {
            if self.match_token(&[TokenType::Dot]).is_some() {
                let member = self.expect_identifier("member access")?;
                node = Expr::MemberAccess { object: Box::new(node), member };
            } else if self.match_token(&[TokenType::LParen]).is_some() {
                let mut args = Vec::new();
                if !matches!(self.peek(0).token_type, TokenType::RParen) {
                    args.push(self.parse_arg()?);
                    while matches!(self.peek(0).token_type, TokenType::Comma) {
                        self.advance();
                        if matches!(self.peek(0).token_type, TokenType::RParen) {
                            break;
                        }
                        args.push(self.parse_arg()?);
                    }
                }
                self.consume(TokenType::RParen, "Expected ')'")?;
                node = Expr::Call { target: Box::new(node), args };
            } else if self.match_token(&[TokenType::LBracket]).is_some() {
                let index = self.parse_expression()?;
                self.consume(TokenType::RBracket, "Expected ']'")?;
                node = Expr::IndexAccess { object: Box::new(node), index: Box::new(index) };
            } else if let Some(tok) = self.match_token(&[TokenType::Range, TokenType::RangeExcl]) {
                let inclusive = matches!(tok.token_type, TokenType::Range);
                let end = self.parse_addition()?;
                node = Expr::Range { start: Box::new(node), end: Box::new(end), inclusive };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_arg(&mut self) -> Result<Expr, HornetError> {
        if let TokenType::Identifier(name) = &self.peek(0).token_type {
            if matches!(self.peek(1).token_type, TokenType::Equals) {
                let name = name.clone();
                self.advance(); // name
                self.advance(); // =
                let value = self.parse_expression()?;
                return Ok(Expr::NamedArg { name, value: Box::new(value) });
            }
        }
        self.parse_expression()
    }
}
