use std::fmt;

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Illegal,
    EOF,
    Ident(String),
    Number(String),
    Punctuator(TokenPunctuator),
    Keyword(TokenKeyword),
    
}
#[derive(Debug, PartialEq)]
pub enum  TokenPunctuator{
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
}

impl TokenPunctuator {
    fn format(&self)->String{
        match &self{
            TokenPunctuator::Assign => String::from("="),
            TokenPunctuator::Plus => String::from("+"),
            TokenPunctuator::Minus => String::from("-"),
            TokenPunctuator::Asterisk => String::from("*"),
            TokenPunctuator::Slash => String::from("/"),
            TokenPunctuator::LParen => String::from("("),
            TokenPunctuator::RParen => String::from(")"),
            TokenPunctuator::Semicolon => String::from(";"),
            TokenPunctuator::Dot => String::from("."),
            TokenPunctuator::Comma => String::from(","),
        }
    }
}

// impl std::fmt::Display for TokenPunctuator {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match &self{
//             TokenPunctuator::Assign => write!(f, "="),
//             TokenPunctuator::Plus => write!(f, "+"),
//             TokenPunctuator::Minus => write!(f, "-"),
//             TokenPunctuator::Asterisk => write!(f, "*"),
//             TokenPunctuator::Slash => write!(f, "/"),
//             TokenPunctuator::LParen => write!(f, "("),
//             TokenPunctuator::RParen => write!(f, ")"),
//             TokenPunctuator::Semicolon => write!(f, ";"),
//             TokenPunctuator::Dot => write!(f, "."),
//             TokenPunctuator::Comma => write!(f, ","),
//         }
//     }
// }
#[derive(Debug, PartialEq)]
pub enum  TokenKeyword {
    Let,    //let
    If,     //if
    Else,   //else
    Return, //return
}
impl std::fmt::Display for TokenKeyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = self.format();
        write!(f, "{}",str)
    }
}

impl TokenKeyword {
    pub fn format(&self)->String{
        match &self {
            TokenKeyword::Let => String::from("let"),
            TokenKeyword::If => String::from("if"),
            TokenKeyword::Else => String::from("else"),
            TokenKeyword::Return => String::from("return"),
        }
    }
}

#[derive(Debug)]
pub struct Token {
    pub typ: TokenType,
    pub line: usize,
    pub column: usize,
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
            TokenType::Ident(t) => write!(f, "<\x1b[31m{}\x1b[39m> ", t),
            TokenType::Number(t) => write!(f, "<\x1b[35m{}\x1b[39m> ", t),
            TokenType::Punctuator(t) =>write!(f, "<\x1b[36m{}\x1b[39m> ",t.format()),
            TokenType::Keyword(t) =>write!(f, "<\x1b[33m{}\x1b[39m> ",t),
            _ => todo!("std::fmt::Display"),
        }
    }
}
