use super::token::{Token, TokenKeyword, TokenPunctuator, TokenType};

pub trait ILexer {
    fn next_token(&mut self) -> Token;
    fn new(input: String) -> Self;
}

pub struct Lexer {
    input: String,
    chars: std::str::Chars<'static>, // 字符迭代器
    position: usize,                 // 当前字符位置
    read_position: usize,            // 下一个字符的位置
    ch: Option<char>,                // 当前字符
    line: usize,                     // 当前行号
    column: usize,                   // 当前列号
}
impl ILexer for Lexer {
    fn new(input: String) -> Self
    {
        let mut lexer = Lexer {
            input: input.clone(),
            chars: "".chars(), // 初始值
            position: 0,
            read_position: 0,
            ch: None,
            line: 1,   // 初始行号为1
            column: 0, // 初始列号为0
        };
        let input_static: &'static str = Box::leak(input.clone().into_boxed_str());
        lexer.chars = input_static.chars();
        lexer.read_char();
        lexer
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let token = match &self.ch {
            Some('=') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    //==
                    self.read_char();
                    let pc2 = self.peek_char();
                    if pc2 == Some('=') {
                        //===
                        self.read_char();
                        self.read_char();
                        return Token::new(
                            TokenType::Punctuator(TokenPunctuator::Congruent),
                            self.line,
                            self.column,
                        );
                    } else {
                        self.read_char();
                        return Token::new(
                            TokenType::Punctuator(TokenPunctuator::Equal),
                            self.line,
                            self.column,
                        );
                    }
                } else {
                    self.read_char();
                    return Token::new(
                        TokenType::Punctuator(TokenPunctuator::MOV),
                        self.line,
                        self.column,
                    );
                }
            }
            Some('+') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    //+=
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::ADD),
                        self.line,
                        self.column,
                    )
                } else if pc == Some('+') {
                    //++
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::INC),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Plus),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('-') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    //-=
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::SUB),
                        self.line,
                        self.column,
                    )
                } else if pc == Some('-') {
                    //--
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::DEC),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Minus),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('*') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::MUL),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Multiply),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('/') => {
                //注释或者是除号
                let pc = self.peek_char();
                if pc == Some('/') {
                    // //
                    while let Some(ch) = self.ch {
                        if ch == '\n' {
                            self.read_char();
                            break;
                        }
                        self.read_char();
                    }
                    return self.next_token();
                } else if pc == Some('=') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::DIV),
                        self.line,
                        self.column,
                    )
                } else if pc == Some('*') {
                    // /*
                    self.read_char();
                    self.read_char();
                    while let Some(ch) = self.ch {
                        let pc = self.peek_char();
                        if ch == '*' && pc == Some('/') {
                            self.read_char();
                            self.read_char();
                            break;
                        }
                        self.read_char();
                    }
                    return self.next_token();
                } else {
                    //除号
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Divide),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('%') => {
                let pc = self.peek_char();
                if pc == Some('/') {
                    // //
                    while let Some(ch) = self.ch {
                        if ch == '\n' {
                            self.read_char();
                            break;
                        }
                        self.read_char();
                    }
                    return self.next_token();
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Modulo),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('(') => Token::new(
                TokenType::Punctuator(TokenPunctuator::LParen),
                self.line,
                self.column,
            ),
            Some(')') => Token::new(
                TokenType::Punctuator(TokenPunctuator::RParen),
                self.line,
                self.column,
            ),
            Some('{') => Token::new(
                TokenType::Punctuator(TokenPunctuator::LCParen),
                self.line,
                self.column,
            ),
            Some('}') => Token::new(
                TokenType::Punctuator(TokenPunctuator::RCParen),
                self.line,
                self.column,
            ),
            Some('[') => Token::new(
                TokenType::Punctuator(TokenPunctuator::LSParen),
                self.line,
                self.column,
            ),
            Some(']') => Token::new(
                TokenType::Punctuator(TokenPunctuator::RSParen),
                self.line,
                self.column,
            ),
            Some(';') => Token::new(
                TokenType::Punctuator(TokenPunctuator::Semicolon),
                self.line,
                self.column,
            ),
            Some(':') => Token::new(
                TokenType::Punctuator(TokenPunctuator::Colon),
                self.line,
                self.column,
            ),
            Some('.') => Token::new(
                TokenType::Punctuator(TokenPunctuator::Dot),
                self.line,
                self.column,
            ),
            Some(',') => Token::new(
                TokenType::Punctuator(TokenPunctuator::Comma),
                self.line,
                self.column,
            ),
            Some('^') => Token::new(
                TokenType::Punctuator(TokenPunctuator::BitXor),
                self.line,
                self.column,
            ),
            Some('~') => Token::new(
                TokenType::Punctuator(TokenPunctuator::BitNot),
                self.line,
                self.column,
            ),
            Some('`') | Some('"') | Some('\'') => {
                let p = self.ch.clone().unwrap();
                let is_template = p == '`';
                self.read_char();
                let mut result = String::new();
                let mut v1: Vec<String> = Vec::new();
                let mut v2: Vec<String> = Vec::new();
                let line = self.line;
                while let Some(ch) = self.ch {
                    if ch == '\\' {
                        let pc = self.peek_char();
                        match pc {
                            Some(t) => {
                                if t == p {
                                    self.read_char();
                                    self.read_char();
                                    result.push(t);
                                    continue;
                                } else if t == '\n' {
                                    self.read_char();
                                    self.read_char();
                                    continue;
                                } else {
                                }
                            }
                            None => return Token::new(TokenType::SyntaxError, line, self.column),
                        }
                    }
                    if is_template {
                        if ch == '$' {
                            let pc = self.peek_char();
                            if pc.is_none() {
                                return Token::new(TokenType::SyntaxError, line, self.column);
                            }
                            if pc == Some('{') {
                                let mut count = 0;
                                self.read_char(); //$
                                v1.push(result.clone());
                                result.clear();
                                loop {
                                    if self.ch == Some('{') {
                                        count += 1;
                                        self.read_char();
                                    } else if self.ch == Some('}') {
                                        count -= 1;
                                        self.read_char();
                                        if count == 0 {
                                            v2.push(result.clone());
                                            result.clear();
                                            break;
                                        }
                                    } else {
                                        result.push(self.ch.unwrap());
                                        self.read_char();
                                    }
                                }
                                continue;
                            } else {
                                break;
                            }
                        }
                    }
                    if self.ch.is_none() {
                        break;
                    }
                    if self.ch.unwrap() == p {
                        self.read_char();
                        break;
                    }
                    result.push(self.ch.unwrap());
                    self.read_char();
                }
                if is_template {
                    if result.len() > 0 {
                        v1.push(result.clone());
                        result.clear();
                    }
                    return Token::new(TokenType::Template(v1, v2), line, self.column);
                } else {
                    return Token::new(
                        TokenType::Literal(format!("{}", result)),
                        line,
                        self.column,
                    );
                }
            }
            Some('&') => {
                let pc = self.peek_char();
                if pc == Some('&') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::And),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::BitAnd),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('|') => {
                let pc = self.peek_char();
                if pc == Some('|') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Or),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::BitOr),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('!') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::NE),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::Not),
                        self.line,
                        self.column,
                    )
                }
            }
            Some(ch) if ch.is_digit(10) => {
                let (num, line) = self.read_number();
                return Token::new(TokenType::Literal(num.to_string()), line, self.column);
            }
            Some(ch) if ch.is_alphabetic() => {
                let (ident, line) = self.read_identifier();
                match ident.as_str() {
                    "let" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Let),
                            line,
                            self.column,
                        );
                    }
                    "const" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Const),
                            line,
                            self.column,
                        );
                    }
                    "var" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Var),
                            line,
                            self.column,
                        );
                    }
                    "if" => {
                        return Token::new(TokenType::Keyword(TokenKeyword::If), line, self.column)
                    }
                    "else" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Else),
                            line,
                            self.column,
                        )
                    }
                    "return" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Return),
                            line,
                            self.column,
                        )
                    }
                    "break" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Break),
                            line,
                            self.column,
                        )
                    }
                    "continue" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Continue),
                            line,
                            self.column,
                        )
                    }
                    "for" => {
                        return Token::new(TokenType::Keyword(TokenKeyword::For), line, self.column)
                    }
                    "in" => {
                        return Token::new(TokenType::Keyword(TokenKeyword::In), line, self.column)
                    }
                    "of" => {
                        return Token::new(TokenType::Keyword(TokenKeyword::Of), line, self.column)
                    }
                    "delete" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Delete),
                            line,
                            self.column,
                        )
                    }
                    "do" => {
                        return Token::new(TokenType::Keyword(TokenKeyword::Do), line, self.column)
                    }
                    "switch" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Swith),
                            line,
                            self.column,
                        )
                    }
                    "case" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Case),
                            line,
                            self.column,
                        )
                    }
                    "default" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Default),
                            line,
                            self.column,
                        )
                    }
                    "function" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::Function),
                            line,
                            self.column,
                        )
                    }
                    "while" => {
                        return Token::new(
                            TokenType::Keyword(TokenKeyword::While),
                            line,
                            self.column,
                        )
                    }
                    _ => {
                        return Token::new(TokenType::Ident(ident), line, self.column);
                    }
                }
            }
            Some('$') | Some('_') => {
                let (ident, line) = self.read_identifier();
                Token::new(TokenType::Ident(ident), self.line, self.column)
            }
            Some('>') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::GTE),
                        self.line,
                        self.column,
                    )
                } else if pc == Some('>') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::RShift),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::GT),
                        self.line,
                        self.column,
                    )
                }
            }
            Some('<') => {
                let pc = self.peek_char();
                if pc == Some('=') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::LTE),
                        self.line,
                        self.column,
                    )
                } else if pc == Some('<') {
                    self.read_char();
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::LShift),
                        self.line,
                        self.column,
                    )
                } else {
                    Token::new(
                        TokenType::Punctuator(TokenPunctuator::LT),
                        self.line,
                        self.column,
                    )
                }
            }
            None => Token::new(TokenType::EOF, self.line, self.column),
            _ => Token::new(TokenType::Illegal, self.line, self.column),
        };
        //这里会在最后再次读取/过滤下个字符,如果不需要该操作则需要提前return
        self.read_char();
        token
    }
}
impl Lexer {
    fn read_char(&mut self) -> bool {
        if let Some(ch) = self.chars.next() {
            self.ch = Some(ch);
            self.position = self.read_position;
            self.read_position += 1;

            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
            true
        } else {
            self.ch = None;
            false
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.clone().next()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.ch {
            if ch.is_whitespace() {
                self.read_char();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> (String, usize) {
        let mut result = String::new();
        let line = self.line;
        while let Some(ch) = self.ch {
            if ch.is_digit(10) || ch == '.' {
                result.push(ch);
                self.read_char();
            } else {
                break;
            }
        }
        (result, line)
    }

    fn read_identifier(&mut self) -> (String, usize) {
        let mut result = String::new();
        let line = self.line;
        while let Some(ch) = self.ch {
            if ch == '$' || ch == '_' || ch.is_alphabetic() || ch.is_digit(10) {
                result.push(ch);
                self.read_char();
            } else {
                break;
            }
        }
        (result, line)
    }

    pub fn print(&mut self) {
        println!("/*--------print--------*/");
        let mut p = Lexer::new(self.input.clone());
        let mut line = 0;
        loop {
            let tok = p.next_token();
            if tok.typ == TokenType::EOF {
                break;
            }
            if line != tok.line && line > 0 {
                println!("");
            }
            line = tok.line;
            print!("{}", tok);
        }
        println!("\n/*-------- end --------*/");
    }
}
