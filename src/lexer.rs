use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    // Keywords
    Fn, If, Else, For, While, Match, Import, Const, In,
    Struct, Async, Await, Pub,
    
    // Literals
    Identifier(String),
    Number(i64),
    String(String),
    
    // Operators and Punctuation
    Equals, Plus, Minus, Star, Slash, Percent, Colon, Dot,
    LParen, RParen, LBrace, RBrace,
    Arrow,     // =>
    Range,     // ..
    RangeExcl, // ..<
    
    // Comparisons
    EqEq, Neq, Lt, Le, Gt, Ge,
    
    // Indentation
    Indent(usize),
    Dedent,
    Newline,
    
    Eof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0],
        }
    }

    fn peek(&self, n: usize) -> Option<char> {
        self.source.get(self.pos + n).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let char = self.peek(0)?;
        self.pos += 1;
        if char == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(char)
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.pos < self.source.len() {

            if self.column == 1 {
                self.handle_indentation(&mut tokens);
                if self.pos >= self.source.len() { break; }
            }

            let _char = self.peek(0).unwrap();
            if char.is_whitespace() && char != '\n' {
                self.advance();
                continue;
            }

            if char == '\n' {
                tokens.push(Token { token_type: TokenType::Newline, line: self.line, column: self.column });
                self.advance();
                continue;
            }

            if char == '#' {
                while self.peek(0).is_some() && self.peek(0).unwrap() != '\n' {
                    self.advance();
                }
                continue;
            }

            if char.is_ascii_digit() {
                tokens.push(self.read_number());
            } else if char.is_alphabetic() || char == '_' {
                tokens.push(self.read_identifier());
            } else if char == '"' {
                tokens.push(self.read_string());
            } else {
                if let Some(token) = self.read_operator() {
                    tokens.push(token);
                } else {
                    panic!("Unexpected character: {} at line {}, col {}", char, self.line, self.column);
                }
            }
        }

        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token { token_type: TokenType::Dedent, line: self.line, column: self.column });
        }
        tokens.push(Token { token_type: TokenType::Eof, line: self.line, column: self.column });
        tokens
    }

    fn handle_indentation(&mut self, tokens: &mut Vec<Token>) {
        let mut current_indent = 0;
        while let Some(' ') = self.peek(0) {
            current_indent += 1;
            self.advance();
        }

        if matches!(self.peek(0), Some('\n') | Some('#') | None) {
            return;
        }

        let last_indent = *self.indent_stack.last().unwrap();
        if current_indent > last_indent {
            if (current_indent - last_indent) % 4 != 0 {
                panic!("Indentation error at line {}", self.line);
            }
            self.indent_stack.push(current_indent);
            tokens.push(Token { token_type: TokenType::Indent(current_indent), line: self.line, column: self.column });
        } else if current_indent < last_indent {
            while current_indent < *self.indent_stack.last().unwrap() {
                self.indent_stack.pop();
                tokens.push(Token { token_type: TokenType::Dedent, line: self.line, column: self.column });
            }
            if current_indent != *self.indent_stack.last().unwrap() {
                panic!("Inconsistent indentation at line {}", self.line);
            }
        }
    }

    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();
        let start_col = self.column;
        while let Some(c) = self.peek(0) {
            if c.is_ascii_digit() {
                num_str.push(self.advance().unwrap());
            } else { break; }
        }
        Token { token_type: TokenType::Number(num_str.parse().unwrap()), line: self.line, column: start_col }
    }

    fn read_identifier(&mut self) -> Token {
        let mut ident_str = String::new();
        let start_col = self.column;
        while let Some(c) = self.peek(0) {
            if c.is_alphanumeric() || c == '_' {
                ident_str.push(self.advance().unwrap());
            } else { break; }
        }

        let token_type = match ident_str.as_str() {
            "fn" => TokenType::Fn,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "for" => TokenType::For,
            "while" => TokenType::While,
            "match" => TokenType::Match,
            "import" => TokenType::Import,
            "const" => TokenType::Const,
            "in" => TokenType::In,
            "struct" => TokenType::Struct,
            "async" => TokenType::Async,
            "await" => TokenType::Await,
            "pub" => TokenType::Pub,
            _ => TokenType::Identifier(ident_str),
        };
        Token { token_type, line: self.line, column: start_col }
    }

    fn read_string(&mut self) -> Token {
        self.advance(); // "
        let start_col = self.column - 1;
        let mut string_val = String::new();
        while let Some(c) = self.peek(0) {
            if c == '"' { break; }
            string_val.push(self.advance().unwrap());
        }
        self.advance(); // "
        Token { token_type: TokenType::String(string_val), line: self.line, column: start_col }
    }

    fn read_operator(&mut self) -> Option<Token> {
        let char = self.peek(0)?;
        let start_col = self.column;

        match char {
            '<' => {
                self.advance();
                if self.peek(0) == Some('=') {
                    self.advance();
                    Some(Token { token_type: TokenType::Le, line: self.line, column: start_col })
                } else {
                    Some(Token { token_type: TokenType::Lt, line: self.line, column: start_col })
                }
            }
            '>' => {
                self.advance();
                if self.peek(0) == Some('=') {
                    self.advance();
                    Some(Token { token_type: TokenType::Ge, line: self.line, column: start_col })
                } else {
                    Some(Token { token_type: TokenType::Gt, line: self.line, column: start_col })
                }
            }
            '!' => {
                self.advance();
                if self.peek(0) == Some('=') {
                    self.advance();
                    Some(Token { token_type: TokenType::Neq, line: self.line, column: start_col })
                } else {
                    panic!("Unexpected character: ! at line {}", self.line);
                }
            }
            '=' => {
                self.advance();
                if self.peek(0) == Some('>') {
                    self.advance();
                    Some(Token { token_type: TokenType::Arrow, line: self.line, column: start_col })
                } else if self.peek(0) == Some('=') {
                    self.advance();
                    Some(Token { token_type: TokenType::EqEq, line: self.line, column: start_col })
                } else {
                    Some(Token { token_type: TokenType::Equals, line: self.line, column: start_col })
                }
            }
            '.' => {
                self.advance();
                if self.peek(0) == Some('.') {
                    self.advance();
                    if self.peek(0) == Some('<') {
                        self.advance();
                        Some(Token { token_type: TokenType::RangeExcl, line: self.line, column: start_col })
                    } else {
                        Some(Token { token_type: TokenType::Range, line: self.line, column: start_col })
                    }
                } else {
                    Some(Token { token_type: TokenType::Dot, line: self.line, column: start_col })
                }
            }
            '+' => { self.advance(); Some(Token { token_type: TokenType::Plus, line: self.line, column: start_col }) }
            '-' => { self.advance(); Some(Token { token_type: TokenType::Minus, line: self.line, column: start_col }) }
            '*' => { self.advance(); Some(Token { token_type: TokenType::Star, line: self.line, column: start_col }) }
            '/' => { self.advance(); Some(Token { token_type: TokenType::Slash, line: self.line, column: start_col }) }
            '%' => { self.advance(); Some(Token { token_type: TokenType::Percent, line: self.line, column: start_col }) }
            ':' => { self.advance(); Some(Token { token_type: TokenType::Colon, line: self.line, column: start_col }) }
            '(' => { self.advance(); Some(Token { token_type: TokenType::LParen, line: self.line, column: start_col }) }
            ')' => { self.advance(); Some(Token { token_type: TokenType::RParen, line: self.line, column: start_col }) }
            '{' => { self.advance(); Some(Token { token_type: TokenType::LBrace, line: self.line, column: start_col }) }
            '}' => { self.advance(); Some(Token { token_type: TokenType::RBrace, line: self.line, column: start_col }) }
            _ => None
        }
    }
}
