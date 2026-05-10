use std::fmt;
use serde_json::Error as SerdeJsonError;

#[derive(Debug)]
pub enum HornetError {
    Io(std::io::Error),
    Lexer(String),
    Parser(String),
    Type(String),
    Other(String),
}

impl fmt::Display for HornetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HornetError::Io(err) => write!(f, "I/O error: {}", err),
            HornetError::Lexer(err) => write!(f, "Lexing error: {}", err),
            HornetError::Parser(err) => write!(f, "Parsing error: {}", err),
            HornetError::Type(err) => write!(f, "Type error: {}", err),
            HornetError::Other(err) => write!(f, "Error: {}", err),
        }
    }
}

impl std::error::Error for HornetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HornetError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for HornetError {
    fn from(err: std::io::Error) -> HornetError {
        HornetError::Io(err)
    }
}

impl From<&str> for HornetError {
    fn from(err: &str) -> HornetError {
        HornetError::Other(err.to_string())
    }
}

impl From<String> for HornetError {
    fn from(err: String) -> HornetError {
        HornetError::Other(err)
    }
}

impl From<SerdeJsonError> for HornetError {
    fn from(err: SerdeJsonError) -> HornetError {
        HornetError::Other(err.to_string())
    }
}
