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
#[derive(Debug, PartialEq, Clone)]
pub enum TokenPunctuator {
    ///=
    MOV, //=
    ///==
    Equal, //==
    ///===
    Congruent, //===
    ///+
    Plus, //+
    ///+=
    Add, //+=
    ///++
    INC, //++
    //-
    Minus, //-
    ///--
    DEC, //--
    ///-=
    SUB, //-=
    ///*
    Multiply, //*
    /// /
    Divide, // /
    ///(
    LParen, // (
    ///)
    RParen, // )
    ///{
    LCParen, //{
    ///}
    RCParen, //}
    ///[
    LSParen, //[
    ///]
    RSParen, //]
    ///;
    Semicolon, //;
    ///.
    Dot, //.
    ///,
    Comma, //,

    ///&
    BitAnd, //&
    ///|
    BitOr, //|
    ///^
    BitXor, //^
    ///~
    BitNot, //~
    ///&&
    And, // &&
    ///||
    Or, // ||
    ///!
    Not, // !
}

impl TokenPunctuator {
    fn format(&self) -> String {
        match &self {
            TokenPunctuator::MOV => String::from("="),
            TokenPunctuator::Equal => String::from("=="),
            TokenPunctuator::Congruent => String::from("==="),
            TokenPunctuator::Plus => String::from("+"),
            TokenPunctuator::Add => String::from("+="),
            TokenPunctuator::INC => String::from("++"),
            TokenPunctuator::Minus => String::from("-"),
            TokenPunctuator::DEC => String::from("--"),
            TokenPunctuator::SUB => String::from("-="),
            TokenPunctuator::Multiply => String::from("*"),
            TokenPunctuator::Divide => String::from("/"),
            TokenPunctuator::LParen => String::from("("),
            TokenPunctuator::RParen => String::from(")"),
            TokenPunctuator::Semicolon => String::from(";"),
            TokenPunctuator::Dot => String::from("."),
            TokenPunctuator::Comma => String::from(","),
            TokenPunctuator::BitAnd => String::from("&"),
            TokenPunctuator::BitOr => String::from("|"),
            TokenPunctuator::BitXor => String::from("^"),
            TokenPunctuator::BitNot => String::from("~"),
            TokenPunctuator::And => String::from("&&"),
            TokenPunctuator::Or => String::from("||"),
            TokenPunctuator::Not => String::from("!"),
            TokenPunctuator::LCParen => String::from("{"),
            TokenPunctuator::RCParen => String::from("}"),
            TokenPunctuator::LSParen => String::from("["),
            TokenPunctuator::RSParen => String::from("]"),
        }
    }
}

impl std::fmt::Display for TokenPunctuator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}
#[derive(Debug, PartialEq)]
pub enum TokenKeyword {
    Let,    //let
    If,     //if
    Else,   //else
    Return, //return
}
impl std::fmt::Display for TokenKeyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = self.format();
        write!(f, "{}", str)
    }
}

impl TokenKeyword {
    pub fn format(&self) -> String {
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
    pub typ: TokenType,//token类型
    pub line: usize,   //行
    pub column: usize, //列
    pub index: usize,  //token的序号
}

static mut TOEKN_INDEX: usize = 0;
impl Token {
    pub fn new(typ: TokenType, line: usize, column: usize) -> Token {
        let mut index = 0;
        unsafe {
            TOEKN_INDEX = TOEKN_INDEX + 1;
            index = TOEKN_INDEX;
        }
        Token {
            typ,
            line,
            column,
            index: index,
        }
    }
    /// 是否是运算符号 + - * /
    pub fn is_operator(&self)->bool{
        match &self.typ{
            TokenType::Punctuator(t) =>{
                match &t{
                    TokenPunctuator::Plus|TokenPunctuator::Minus|TokenPunctuator::Multiply|TokenPunctuator::Divide => true,
                    _=>false
                }
            },
            _=>false
        }
    }
    /// 是否是逻辑符号 && ||
    pub fn is_logical(&self)->bool{
        match &self.typ{
            TokenType::Punctuator(t) =>{
                match &t{
                    TokenPunctuator::And|TokenPunctuator::Or => true,
                    _=>false
                }
            },
            _=>false
        }
    }
    /// 是否是eof 或 ;
    pub fn is_eof_or_semicolon(&self)->bool{
        match &self.typ{
            TokenType::EOF=>true,
            TokenType::Punctuator(t) =>{
                match &t{
                    TokenPunctuator::Semicolon => true,
                    _=>false
                }
            },
            _=>false
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.typ {
            TokenType::Illegal => panic!("Illegal"),
            TokenType::EOF => write!(f, ""),
            TokenType::Ident(t) => write!(f, "<\x1b[31m{}\x1b[39m> ", t),
            TokenType::Number(t) => write!(f, "<\x1b[35m{}\x1b[39m> ", t),
            TokenType::Punctuator(t) => write!(f, "<\x1b[36m{}\x1b[39m> ", t.format()),
            TokenType::Keyword(t) => write!(f, "<key:\x1b[33m{}\x1b[39m> ", t),
        }
    }
}
