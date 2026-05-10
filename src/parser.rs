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

    fn peek(&self, n: usize) -> &Token {
        if self.pos + n >= self.tokens.len() {
            &self.tokens[self.tokens.len() - 1]
        } else {
            &self.tokens[self.pos + n]
        }
    }

    fn advance(&mut self) -> &Token {
        let token = self.peek(0);
        self.pos += 1;
        token
    }

    fn match_token(&mut self, types: &[TokenType]) -> Option<&Token> {
        for t in types {
            // We use matches! because TokenType might have associated data
            // but for simplicity we can compare discriminant if needed.
            // Here we check if the type matches.
            if std::mem::discriminant(&self.peek(0).token_type) == std::mem::discriminant(t) {
                return Some(self.advance());
            }
        }
        None
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> &Token {
        if std::mem::discriminant(&self.peek(0).token_type) == std::mem::discriminant(&token_type) {
            self.advance()
        } else {
            panic!("{} at line {}", message, self.peek(0).line);
        }
    }

    pub fn parse(&mut self) -> Program {
        let mut statements = Vec::new();
        while !matches!(self.peek(0).token_type, TokenType::Eof) {
            if matches!(self.peek(0).token_type, TokenType::Newline) {
                self.advance();
                continue;
            }
            statements.push(self.parse_statement());
        }
        Program { statements }
    }

    fn parse_statement(&mut self) -> Stmt {
        match &self.peek(0).token_type {
            TokenType::Fn => self.parse_function_def(),
            TokenType::If => self.parse_if_stmt(),
            TokenType::For => self.parse_for_stmt(),
            TokenType::Import => self.parse_import(),
            TokenType::Struct => self.parse_struct_def(),
            TokenType::Identifier(_) if matches!(self.peek(1).token_type, TokenType::Equals) => self.parse_assignment(),
            _ => Stmt::Expr(self.parse_expression()),
        }
    }

    fn parse_import(&mut self) -> Stmt {
        self.advance(); // import
        let name = if let TokenType::Identifier(n) = &self.advance().token_type { n.clone() } else { panic!("Expected name"); };
        Stmt::Import(name)
    }

    fn parse_struct_def(&mut self) -> Stmt {
        self.advance(); // struct
        let name = if let TokenType::Identifier(n) = &self.advance().token_type { n.clone() } else { panic!("Expected name"); };
        self.consume(TokenType::Colon, "Expected ':'");
        self.consume(TokenType::Newline, "Expected newline");
        self.consume(TokenType::Indent(0), "Expected indent");
        let mut fields = Vec::new();
        while !matches!(self.peek(0).token_type, TokenType::Dedent) {
            let field_name = if let TokenType::Identifier(n) = &self.advance().token_type { n.clone() } else { panic!("Expected name"); };
            self.consume(TokenType::Colon, "Expected ':'");
            let field_type = if let TokenType::Identifier(n) = &self.advance().token_type { n.clone() } else { panic!("Expected type"); };
            fields.push((field_name, field_type));
            self.consume(TokenType::Newline, "Expected newline");
        }
        self.consume(TokenType::Dedent, "Expected dedent");
        Stmt::StructDef { name, fields }
    }

    fn parse_function_def(&mut self) -> Stmt {
        self.advance(); // fn
        let name = if let TokenType::Identifier(n) = &self.advance().token_type { n.clone() } else { panic!("Expected name"); };
        self.consume(TokenType::LParen, "Expected '('");
        let mut params = Vec::new();
        if !matches!(self.peek(0).token_type, TokenType::RParen) {
            if let TokenType::Identifier(p) = &self.advance().token_type { params.push(p.clone()); }
            // Grammar HACK: using any separator or colon if needed
            while matches!(self.peek(0).token_type, TokenType::Colon) {
                self.advance();
                if let TokenType::Identifier(p) = &self.advance().token_type { params.push(p.clone()); }
            }
        }
        self.consume(TokenType::RParen, "Expected ')'");
        self.consume(TokenType::Colon, "Expected ':'");
        let body = self.parse_block();
        Stmt::FunctionDef { name, params, body }
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        self.consume(TokenType::Newline, "Expected newline");
        self.consume(TokenType::Indent(0), "Expected indent");
        let mut statements = Vec::new();
        while !matches!(self.peek(0).token_type, TokenType::Dedent | TokenType::Eof) {
            if matches!(self.peek(0).token_type, TokenType::Newline) {
                self.advance();
                continue;
            }
            statements.push(self.parse_statement());
        }
        self.consume(TokenType::Dedent, "Expected dedent");
        statements
    }

    fn parse_assignment(&mut self) -> Stmt {
        let name = if let TokenType::Identifier(n) = &self.advance().token_type { n.clone() } else { panic!("Expected name"); };
        self.consume(TokenType::Equals, "Expected '='");
        let value = self.parse_expression();
        Stmt::Assignment { name, value }
    }

    fn parse_if_stmt(&mut self) -> Stmt {
        self.advance(); // if
        let condition = self.parse_expression();
        self.consume(TokenType::Colon, "Expected ':'");
        let then_branch = self.parse_block();
        
        let mut else_ifs = Vec::new();
        while matches!(self.peek(0).token_type, TokenType::Else) && matches!(self.peek(1).token_type, TokenType::If) {
            self.advance(); // else
            self.advance(); // if
            let cond = self.parse_expression();
            self.consume(TokenType::Colon, "Expected ':'");
            let branch = self.parse_block();
            else_ifs.push((cond, branch));
        }
        
        let mut else_branch = None;
        if matches!(self.peek(0).token_type, TokenType::Else) {
            self.advance(); // else
            self.consume(TokenType::Colon, "Expected ':'");
            else_branch = Some(self.parse_block());
        }
        
        Stmt::If { condition, then_branch, else_ifs, else_branch }
    }

    fn parse_for_stmt(&mut self) -> Stmt {
        self.advance(); // for
        let iterator = if let TokenType::Identifier(n) = &self.advance().token_type { n.clone() } else { panic!("Expected name"); };
        self.consume(TokenType::In, "Expected 'in'");
        let iterable = self.parse_expression(); // Could be range
        self.consume(TokenType::Colon, "Expected ':'");
        let body = self.parse_block();
        Stmt::For { iterator, iterable, body }
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Expr {
        let mut node = self.parse_comparison();
        while let Some(tok) = self.match_token(&[TokenType::EqEq, TokenType::Neq]) {
            let op = match &tok.token_type {
                TokenType::EqEq => "==".to_string(),
                TokenType::Neq => "!=".to_string(),
                _ => unreachable!(),
            };
            node = Expr::BinaryOp {
                left: Box::new(node),
                op,
                right: Box::new(self.parse_comparison()),
            };
        }
        node
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut node = self.parse_addition();
        while let Some(tok) = self.match_token(&[TokenType::Lt, TokenType::Le, TokenType::Gt, TokenType::Ge]) {
            let op = match &tok.token_type {
                TokenType::Lt => "<".to_string(),
                TokenType::Le => "<=".to_string(),
                TokenType::Gt => ">".to_string(),
                TokenType::Ge => ">=".to_string(),
                _ => unreachable!(),
            };
            node = Expr::BinaryOp {
                left: Box::new(node),
                op,
                right: Box::new(self.parse_addition()),
            };
        }
        node
    }

    fn parse_addition(&mut self) -> Expr {
        let mut node = self.parse_multiplication();
        while let Some(tok) = self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let op = match &tok.token_type {
                TokenType::Plus => "+".to_string(),
                TokenType::Minus => "-".to_string(),
                _ => unreachable!(),
            };
            node = Expr::BinaryOp {
                left: Box::new(node),
                op,
                right: Box::new(self.parse_multiplication()),
            };
        }
        node
    }

    fn parse_multiplication(&mut self) -> Expr {
        let mut node = self.parse_factor();
        while let Some(tok) = self.match_token(&[TokenType::Star, TokenType::Slash, TokenType::Percent]) {
            let op = match &tok.token_type {
                TokenType::Star => "*".to_string(),
                TokenType::Slash => "/".to_string(),
                TokenType::Percent => "%".to_string(),
                _ => unreachable!(),
            };
            node = Expr::BinaryOp {
                left: Box::new(node),
                op,
                right: Box::new(self.parse_factor()),
            };
        }
        node
    }

    fn parse_factor(&mut self) -> Expr {
        let mut node = if self.match_token(&[TokenType::LParen]).is_some() {
            let expr = self.parse_expression();
            self.consume(TokenType::RParen, "Expected ')'");
            expr
        } else {
            let tok = self.advance();
            match &tok.token_type {
                TokenType::Number(n) => Expr::Literal(Literal::Number(*n)),
                TokenType::String(s) => Expr::Literal(Literal::String(s.clone())),
                TokenType::Identifier(i) => Expr::Identifier(i.clone()),
                _ => panic!("Unexpected token {:?} at line {}", tok.token_type, tok.line),
            }
        };

        loop {
            if self.match_token(&[TokenType::Dot]).is_some() {
                let member = if let TokenType::Identifier(n) = &self.advance().token_type { n.clone() } else { panic!("Expected name"); };
                node = Expr::MemberAccess { object: Box::new(node), member };
            } else if self.match_token(&[TokenType::LParen]).is_some() {
                let mut args = Vec::new();
                if !matches!(self.peek(0).token_type, TokenType::RParen) {
                    args.push(self.parse_expression());
                    while matches!(self.peek(0).token_type, TokenType::Colon) {
                        self.advance();
                        args.push(self.parse_expression());
                    }
                }
                self.consume(TokenType::RParen, "Expected ')'");
                node = Expr::Call { target: Box::new(node), args };
            } else if self.match_token(&[TokenType::Range, TokenType::RangeExcl]).is_some() {
                let inclusive = matches!(self.peek(-1).token_type, TokenType::Range);
                let end = self.parse_expression();
                node = Expr::Range { start: Box::new(node), end: Box::new(end), inclusive };
            } else {
                break;
            }
        }
        node
    }
}
