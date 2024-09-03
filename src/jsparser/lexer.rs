use core::num;

use super::token::{Token, TokenType,TokenPunctuator,TokenKeyword};

pub struct Lexer {
    input: String,
    position: usize,      // 当前字符位置
    read_position: usize, // 下一个字符的位置
    ch: Option<char>,     // 当前字符
    line: usize,          // 当前行号
    column: usize,        // 当前列号
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: None,
            line: 1,   // 初始行号为1
            column: 0, // 初始列号为0
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) -> bool {
        if self.read_position >= self.input.len() {
            self.ch = None;
        } else {
            self.ch = self.input.chars().nth(self.read_position);
        }
        self.position = self.read_position;
        self.read_position += 1;

        if let Some(ch) = self.ch {
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
                return true;
            } else {
                self.column += 1;
            }
        }
        false
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.read_position)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let token = match &self.ch {
            Some('=') =>{
                Token::new(TokenType::Punctuator(TokenPunctuator::Assign), self.line, self.column)
            },
            Some('+') =>Token::new(TokenType::Punctuator(TokenPunctuator::Plus), self.line, self.column),
            Some('-') => {
                return Token::new(TokenType::Punctuator(TokenPunctuator::Minus), self.line, self.column);
            },
            Some('*') => { //  */
                let pc = self.peek_char();
                if pc == Some('/'){
                    todo!()
                }
                else {
                    Token::new(TokenType::Punctuator(TokenPunctuator::Asterisk), self.line, self.column)
                }
            },
            Some('/') => {//注释或者是除号
                let pc = self.peek_char();
                if pc == Some('/') {// //
                    while let Some(ch) = self.ch {
                        if ch == '\n' {
                            self.read_char();
                            break;
                        }
                        self.read_char();
                    }
                    return self.next_token();
                } else if pc == Some('*') {// /*
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
                } else {//除号
                    Token::new(TokenType::Punctuator(TokenPunctuator::Slash), self.line, self.column)
                }
            },
            Some('(') => {
                return Token::new(TokenType::Punctuator(TokenPunctuator::LParen), self.line, self.column);
            },
            Some(')') => {
                return Token::new(TokenType::Punctuator(TokenPunctuator::RParen), self.line, self.column);
            },
            Some(';') => Token::new(TokenType::Punctuator(TokenPunctuator::Semicolon), self.line, self.column),
            Some('.') => {
                return Token::new(TokenType::Punctuator(TokenPunctuator::Dot), self.line, self.column);
            },
            Some(ch) if ch.is_digit(10) => {
                let num = self.read_number();
                return Token::new(TokenType::Number(num), self.line, self.column);
            }
            Some(ch) if ch.is_alphabetic() => {
                let ident = self.read_identifier();
                match ident.as_str() {
                    "let" => {
                        return Token::new(TokenType::Keyword(TokenKeyword::Let), self.line, self.column);
                    },
                    "if" => Token::new(TokenType::Keyword(TokenKeyword::If), self.line, self.column),
                    "else" => Token::new(TokenType::Keyword(TokenKeyword::Else), self.line, self.column),
                    "return" => Token::new(TokenType::Keyword(TokenKeyword::Return), self.line, self.column),
                    _ => {
                        return Token::new(TokenType::Ident(ident), self.line, self.column);
                    },
                }
            }
            None => Token::new(TokenType::EOF, self.line, self.column),
            _ => Token::new(TokenType::Illegal, self.line, self.column),
        };
        //这里会在最后再次读取/过滤下个字符,如果不需要该操作则需要提前return
        self.read_char();
        token
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

    fn read_number(&mut self) -> String {
        let position = self.position;
        while let Some(ch) = self.ch {
            if ch.is_digit(10) {
                self.read_char();
            } else {
                break;
            }
        }
        let number = self.input[position..self.position].to_string();
        // println!("{:?}",number);
        return number;
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;

        while let Some(ch) = self.ch {
            if ch == '$' || ch.is_alphabetic() || ch.is_digit(10){
                self.read_char();
            }
            else {
                break;
            }
        }
        let ident = self.input[position..self.position].to_string();
        // println!("{}",ident);
        ident
    }

    pub fn print(&mut self) {
        println!("/*--------print--------*/");
        let mut p = Lexer::new(String::from(self.input.clone()));
        loop {
            let tok = p.next_token();
            if tok.typ == TokenType::EOF {
                break;
            }
            // print!("{}", tok);
            println!("{} {:?}",tok, tok);
        }
        println!("\n/*-------- end --------/*");
    }
}
