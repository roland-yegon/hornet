use crate::error::HornetError;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    // Keywords
    Define, If, Else, For, While, Check, Use, Const, In, Return, And, Or, Not,
    Record, Async, Await, Pub, Let, Break, Continue, Repeat,
    When, Otherwise, Gives, From, To, Upto,
    Is, Isnt, Above, Below, Atleast, Atmost,
    
    // Deprecated legacy keywords (kept for compatibility only)
    Fn, Match, Import, Struct, Loop,
    
    // Literals
    Identifier(String),
    Int(i64),
    Float(f64),
    String(String),
    True,
    False,
    
    // Operators and Punctuation
    Equals, Plus, Minus, Star, Slash, FloorDiv, Percent, Colon, Dot,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Arrow,     // =>
    RArrow,    // ->
    Range,     // ..
    RangeExcl, // ..<
    
    // Comparisons
    EqEq, Neq, Lt, Le, Gt, Ge,
    
    // Punctuation
    Comma,
    
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

    pub fn tokenize(&mut self) -> Result<Vec<Token>, HornetError> {
        let mut tokens = Vec::new();
        while self.pos < self.source.len() {

            if self.column == 1 {
                self.handle_indentation(&mut tokens)?;
                if self.pos >= self.source.len() { break; }
            }

            let char = self.peek(0).ok_or_else(|| HornetError::Lexer(format!("Unexpected end of source at line {}, col {}", self.line, self.column)))?;
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
                while self.peek(0).is_some() && self.peek(0) != Some('\n') {
                    self.advance();
                }
                continue;
            }

            if char.is_ascii_digit() {
                tokens.push(self.read_number()?);
            } else if char.is_alphabetic() || char == '_' {
                tokens.push(self.read_identifier());
            } else if char == '"' {
                tokens.push(self.read_string()?);
            } else {
                if let Some(token) = self.read_operator()? {
                    tokens.push(token);
                } else {
                    return Err(HornetError::Lexer(format!("Unexpected character: {} at line {}, col {}", char, self.line, self.column)));
                }
            }
        }

        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token { token_type: TokenType::Dedent, line: self.line, column: self.column });
        }
        tokens.push(Token { token_type: TokenType::Eof, line: self.line, column: self.column });
        Ok(tokens)
    }

    fn handle_indentation(&mut self, tokens: &mut Vec<Token>) -> Result<(), HornetError> {
        let mut current_indent = 0;
        while let Some(' ') = self.peek(0) {
            current_indent += 1;
            self.advance();
        }

        if matches!(self.peek(0), Some('\n') | Some('#') | None) {
            return Ok(());
        }

        let last_indent = *self.indent_stack.last().unwrap_or(&0);
        if current_indent > last_indent {
            if !(current_indent - last_indent).is_multiple_of(4) {
                return Err(HornetError::Lexer(format!("Indentation error at line {}", self.line)));
            }
            self.indent_stack.push(current_indent);
            tokens.push(Token { token_type: TokenType::Indent(current_indent), line: self.line, column: self.column });
        } else if current_indent < last_indent {
            while current_indent < *self.indent_stack.last().unwrap_or(&0) {
                self.indent_stack.pop();
                tokens.push(Token { token_type: TokenType::Dedent, line: self.line, column: self.column });
            }
            if current_indent != *self.indent_stack.last().unwrap_or(&0) {
                return Err(HornetError::Lexer(format!("Inconsistent indentation at line {}", self.line)));
            }
        }
        Ok(())
    }

    fn read_number(&mut self) -> Result<Token, HornetError> {
        let mut num_str = String::new();
        let start_col = self.column;
        while let Some(c) = self.peek(0) {
            if c.is_ascii_digit() {
                num_str.push(self.advance().unwrap());
            } else { break; }
        }
        
        // Check for float
        if self.peek(0) == Some('.') && matches!(self.peek(1), Some(c) if c.is_ascii_digit()) {
            num_str.push(self.advance().unwrap()); // consume '.'
            while let Some(c) = self.peek(0) {
                if c.is_ascii_digit() {
                    num_str.push(self.advance().unwrap());
                } else { break; }
            }
            let value = num_str.parse::<f64>().map_err(|_| HornetError::Lexer(format!("Invalid float literal '{}' at line {}", num_str, self.line)))?;
            return Ok(Token { token_type: TokenType::Float(value), line: self.line, column: start_col });
        }

        let value = num_str.parse::<i64>().map_err(|_| HornetError::Lexer(format!("Invalid number literal '{}' at line {}", num_str, self.line)))?;
        Ok(Token { token_type: TokenType::Int(value), line: self.line, column: start_col })
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
            "define" => TokenType::Define,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "for" => TokenType::For,
            "while" => TokenType::While,
            "repeat" => TokenType::Repeat,
            "check" => TokenType::Check,
            "use" => TokenType::Use,
            "when" => TokenType::When,
            "otherwise" => TokenType::Otherwise,
            "gives" => TokenType::Gives,
            "from" => TokenType::From,
            "to" => TokenType::To,
            "upto" => TokenType::Upto,
            "record" => TokenType::Record,
            "is" => TokenType::Is,
            "isnt" => TokenType::Isnt,
            "above" => TokenType::Above,
            "below" => TokenType::Below,
            "atleast" => TokenType::Atleast,
            "atmost" => TokenType::Atmost,
            "loop" => TokenType::Loop, // [[DEPRECATED: use 'repeat']]
            "match" => TokenType::Match, // [[DEPRECATED: use 'check']]
            "import" => TokenType::Import, // [[DEPRECATED: use 'use']]
            "struct" => TokenType::Struct, // [[DEPRECATED: use 'record']]
            "fn" => TokenType::Fn, // [[DEPRECATED: use 'define']]
            "const" => TokenType::Const,
            "let" => TokenType::Let,
            "in" => TokenType::In,
            "async" => TokenType::Async,
            "await" => TokenType::Await,
            "pub" => TokenType::Pub,
            "return" => TokenType::Return,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            "true" => TokenType::True,
            "false" => TokenType::False,
            _ => TokenType::Identifier(ident_str),
        };
        Token { token_type, line: self.line, column: start_col }
    }

    fn read_string(&mut self) -> Result<Token, HornetError> {
        self.advance(); // "
        let start_col = self.column - 1;
        let mut string_val = String::new();
        while let Some(c) = self.peek(0) {
            if c == '"' {
                self.advance();
                return Ok(Token { token_type: TokenType::String(string_val), line: self.line, column: start_col });
            }
            string_val.push(self.advance().unwrap());
        }

        Err(HornetError::Lexer(format!("Unterminated string literal at line {}", self.line)))
    }

    fn read_operator(&mut self) -> Result<Option<Token>, HornetError> {
        let char = self.peek(0).ok_or_else(|| HornetError::Lexer(format!("Unexpected end of source at line {}, col {}", self.line, self.column)))?;
        let start_col = self.column;

        Ok(match char {
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
                    return Err(HornetError::Lexer(format!("Unexpected character: ! at line {}", self.line)));
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
            '-' => {
                self.advance();
                if self.peek(0) == Some('>') {
                    self.advance();
                    Some(Token { token_type: TokenType::RArrow, line: self.line, column: start_col })
                } else {
                    Some(Token { token_type: TokenType::Minus, line: self.line, column: start_col })
                }
            }
            '*' => { self.advance(); Some(Token { token_type: TokenType::Star, line: self.line, column: start_col }) }
            '/' => {
                self.advance();
                if self.peek(0) == Some('/') {
                    self.advance();
                    Some(Token { token_type: TokenType::FloorDiv, line: self.line, column: start_col })
                } else {
                    Some(Token { token_type: TokenType::Slash, line: self.line, column: start_col })
                }
            }
            '%' => { self.advance(); Some(Token { token_type: TokenType::Percent, line: self.line, column: start_col }) }
            ':' => { self.advance(); Some(Token { token_type: TokenType::Colon, line: self.line, column: start_col }) }
            '(' => { self.advance(); Some(Token { token_type: TokenType::LParen, line: self.line, column: start_col }) }
            ')' => { self.advance(); Some(Token { token_type: TokenType::RParen, line: self.line, column: start_col }) }
            '{' => { self.advance(); Some(Token { token_type: TokenType::LBrace, line: self.line, column: start_col }) }
            '}' => { self.advance(); Some(Token { token_type: TokenType::RBrace, line: self.line, column: start_col }) }
            '[' => { self.advance(); Some(Token { token_type: TokenType::LBracket, line: self.line, column: start_col }) }
            ']' => { self.advance(); Some(Token { token_type: TokenType::RBracket, line: self.line, column: start_col }) }
            ',' => { self.advance(); Some(Token { token_type: TokenType::Comma, line: self.line, column: start_col }) }
            _ => None,
        })
    }
}
