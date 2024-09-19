use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Illegal,
    EOF,
    Literal(String), // 1  "a"
    // TemplateLiteral(String),  //``
    Ident(String), //a
    Punctuator(TokenPunctuator),
    Keyword(TokenKeyword),
}
impl TokenType {
    fn to_raw(&self) -> String {
        match &self {
            TokenType::Illegal => "Illegal".to_string(),
            TokenType::EOF => "EOF".to_string(),
            TokenType::Punctuator(t) => t.to_raw(),
            TokenType::Keyword(t) => t.to_raw(),
            TokenType::Literal(t) => t.to_string(),
            // TokenType::TemplateLiteral(_) => todo!(),
            TokenType::Ident(t) => t.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    /// \>
    GT, // >
    /// \>=
    GTE, // >=
    /// <
    LT, // <
    /// <=
    LTE, // <=
    /// !=
    NE, // !=

    /// 虽然js 还有 <<< >>> ,但是目前并打算加入

    /// <<
    LShift, //<<
    /// \>>
    RShift, //>>
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
    pub fn to_raw(&self) -> String {
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
            TokenPunctuator::GT => String::from(">"),
            TokenPunctuator::GTE => String::from(">="),
            TokenPunctuator::LT => String::from("<"),
            TokenPunctuator::LTE => String::from("<="),
            TokenPunctuator::NE => String::from("!="),
            TokenPunctuator::LShift => String::from("<<"),
            TokenPunctuator::RShift => String::from(">>"),
        }
    }
    pub fn is_precedence(&self) -> bool {
        match &self {
            TokenPunctuator::Plus | TokenPunctuator::Minus => true,
            TokenPunctuator::Multiply | TokenPunctuator::Divide => true,
            TokenPunctuator::Or => true,
            TokenPunctuator::And => true,
            TokenPunctuator::Not => true,
            TokenPunctuator::LShift | TokenPunctuator::RShift => true,
            TokenPunctuator::Equal | TokenPunctuator::NE => true,
            TokenPunctuator::GT
            | TokenPunctuator::GTE
            | TokenPunctuator::LT
            | TokenPunctuator::LTE => true,
            TokenPunctuator::BitOr => true,
            TokenPunctuator::BitXor => true,
            TokenPunctuator::BitAnd => true,
            _ => return false,
        }
    }
    pub fn is_unary(&self) -> bool {
        match &self {
            TokenPunctuator::BitNot
            | TokenPunctuator::Not
            | TokenPunctuator::Plus
            | TokenPunctuator::Minus => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for TokenPunctuator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_raw())
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKeyword {
    None,

    Let,      //let
    Const,    //const
    Var,      //var
    If,       //if
    Else,     //else
    Return,   //return
    For,      //for
    In,       //in
    Of,       //of
    Break,    //break
    Continue, //continue
    Delete,   //delete
    Do,       //do
    Swith,    //switch
}
impl std::fmt::Display for TokenKeyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = self.to_raw();
        write!(f, "{}", str)
    }
}

impl TokenKeyword {
    pub fn to_raw(&self) -> String {
        match &self {
            TokenKeyword::Let => String::from("let"),
            TokenKeyword::If => String::from("if"),
            TokenKeyword::Else => String::from("else"),
            TokenKeyword::Return => String::from("return"),
            TokenKeyword::Break => String::from("break"),
            TokenKeyword::Const => String::from("const"),
            TokenKeyword::Var => String::from("var"),
            TokenKeyword::For => String::from("for"),
            TokenKeyword::Do => String::from("do"),
            TokenKeyword::Swith => String::from("switch"),
            TokenKeyword::In => String::from("in"),
            TokenKeyword::Of => String::from("of"),
            TokenKeyword::Delete => String::from("delete"),
            TokenKeyword::None => String::from(""),
            TokenKeyword::Continue => String::from("continue"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub raw: String,
    pub typ: TokenType, //token类型
    pub line: usize,    //行
    pub column: usize,  //列
}

impl Token {
    pub fn new(typ: TokenType, line: usize, column: usize) -> Token {
        let raw = typ.to_raw();
        Token {
            typ,
            line,
            column,
            raw,
        }
    }
    pub fn desc(&self) -> String {
        String::from(format!(
            "{:?},line:{},column:{}",
            self.typ, self.line, self.column
        ))
    }

    pub fn checked_keyword(&self) -> bool {
        match &self.typ {
            TokenType::Keyword(t) => {
                return true;
            }
            _ => false,
        }
    }
    pub fn is_keyword(&self, key: TokenKeyword) -> bool {
        match &self.typ {
            TokenType::Keyword(t) => {
                return key == *t;
            }
            _ => false,
        }
    }
    pub fn is_precedence(&self) -> bool {
        match &self.typ {
            TokenType::Punctuator(t) => t.is_precedence(),
            _ => false,
        }
    }
    pub fn is_binary(&self) -> bool {
        match &self.typ {
            TokenType::Punctuator(t) => match &t {
                TokenPunctuator::Equal
                | TokenPunctuator::Congruent
                | TokenPunctuator::GT
                | TokenPunctuator::GTE
                | TokenPunctuator::LT
                | TokenPunctuator::LTE
                | TokenPunctuator::NE
                | TokenPunctuator::LShift
                | TokenPunctuator::RShift => true,
                _ => false,
            },
            _ => false,
        }
    }
    pub fn is_logical(&self) -> bool {
        match &self.typ {
            TokenType::Punctuator(t) => match &t {
                TokenPunctuator::And | TokenPunctuator::Or => true,
                _ => false,
            },
            _ => false,
        }
    }
    pub fn is_update(&self) -> bool {
        match &self.typ {
            TokenType::Punctuator(t) => match &t {
                TokenPunctuator::INC | TokenPunctuator::DEC => true,
                _ => false,
            },
            _ => false,
        }
    }
    pub fn is_eof(&self, is_semicolon: bool) -> bool {
        match &self.typ {
            TokenType::EOF => true,
            TokenType::Punctuator(t) => match &t {
                TokenPunctuator::Semicolon => is_semicolon,
                _ => false,
            },
            _ => false,
        }
    }
    pub fn is_ptor(&self, ptor: TokenPunctuator) -> bool {
        match &self.typ {
            TokenType::Punctuator(t) => {
                return ptor == *t;
            }
            _ => false,
        }
    }
    pub fn is_unary(&self) -> bool {
        match &self.typ {
            TokenType::Punctuator(t) => t.is_unary(),
            _ => false,
        }
    }
    pub fn is_ident(&self) -> bool {
        match &self.typ {
            TokenType::Ident(t) => true,
            _ => false,
        }
    }
    pub fn is_literal(&self) -> bool {
        match &self.typ {
            TokenType::Literal(t) => true,
            _ => false,
        }
    }
    pub fn is_ident_num_is_literal_template(&self) -> bool {
        self.is_ident() || self.is_literal() || self.is_literal() || self.is_template_literal()
    }
    pub fn is_template_literal(&self) -> bool {
        match &self.typ {
            //TokenType::TemplateLiteral(_) => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.typ {
            TokenType::Illegal => panic!("Illegal"),
            TokenType::EOF => write!(f, ""),
            TokenType::Ident(t) => write!(f, "<\x1b[31m{}\x1b[39m> ", t),
            TokenType::Punctuator(t) => write!(f, "<\x1b[36m{}\x1b[39m> ", t.to_raw()),
            TokenType::Keyword(t) => write!(f, "<key:\x1b[33m{}\x1b[39m> ", t),
            TokenType::Literal(t) => write!(f, "<\x1b[35m{}\x1b[39m> ", t),
            // TokenType::TemplateLiteral(t) => write!(f, "<temp:\x1b[33m{}\x1b[39m> ", t),
        }
    }
}
