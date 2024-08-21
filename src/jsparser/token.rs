use std::fmt;
#[derive(Debug, PartialEq)]
pub enum TokenType {
    Illegal,
    EOF,
    Ident(String),
    Number(String),
    Assign,    //=
    Plus,      //+
    Minus,     //-
    Asterisk,  //*
    Slash,     // /
    LParen,    // (
    RParen,    // )
    Semicolon, //;
    Dot,       //.
    Comma,     //,

    Let,    //let
    If,     //if
    Else,   //else
    Return, //return
}

#[derive(Debug)]
pub struct Token {
    pub typ: TokenType,
    line: usize,
    column: usize,
}

impl Token {
    pub fn new(typ: TokenType, line: usize, column: usize) -> Token {
        Token { typ, line, column }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.typ {
            TokenType::Illegal => panic!("Illegal"),
            TokenType::EOF => write!(f, ""),
            TokenType::Ident(t) => write!(f, " ({}) ", t),
            TokenType::Number(t) => write!(f, " ({}) ", t),
            TokenType::Assign => write!(f, "="),
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Asterisk => write!(f, "*"),
            TokenType::Slash => write!(f, "/"),
            TokenType::LParen => write!(f, "("),
            TokenType::RParen => write!(f, ")"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Dot => write!(f, "."),
            TokenType::Comma => write!(f, ","),
            TokenType::Let => write!(f, "*let"),
            TokenType::If => write!(f, "*if"),
            TokenType::Else => write!(f, "*else"),
            TokenType::Return => write!(f, "*return"),
        }
    }
}
